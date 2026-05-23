use anyhow::{Context, Result};
use application::{
    commands::{
        change_status::ChangeStatusHandler, create_task::CreateTaskHandler,
        delete_task::DeleteTaskHandler,
    },
    queries::{get_task::GetTaskByIdHandler, list_tasks::ListTaskQueryHandler},
};
use clap::{Parser, Subcommand};
use infrastructure::{
    pg_task_repository::{PgPoolOptions, PgTaskRepository},
    redis_publisher::RedisPublisher,
};
use interface::{
    AppState,
    cli::{Cli as CliCommand, run_command},
    http,
};
use std::sync::Arc;
use tokio::signal;
use tracing::info;

#[derive(Debug, Parser)]
#[command(name = "task-manager", version, about = "DDD/CQRS Task Manager")]
struct AppCli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Debug, Subcommand)]
enum Mode {
    Serve {
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
    },

    Cli {
        #[command(subcommand)]
        command: interface::cli::Command,
    },
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();
}

async fn build_app_state() -> Result<AppState> {
    let db_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/tasks)")?;
    let redis_url = std::env::var("REDIS_URL")
        .context("REDIS_URL must be set (e.g. redis://localhost:6379)")?;
    let channel =
        std::env::var("EVENTS_CHANNEL").context("EVENTS_CHANNEL must be set (e.g. task.events)")?;

    info!("connecting to postgres...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .context("postgres connect")?;

    // Trait-объекты для портов
    let repo = Arc::new(PgTaskRepository::new(pool));
    let publisher = Arc::new(
        RedisPublisher::open(&redis_url, channel)
            .await
            .context("Redis open chanel error")?,
    );

    // Сборка use case'ов
    let state = AppState {
        create_task: Arc::new(CreateTaskHandler::new(repo.clone(), publisher.clone())),
        chenge_status: Arc::new(ChangeStatusHandler::new(repo.clone(), publisher.clone())),
        delete_task: Arc::new(DeleteTaskHandler::new(repo.clone())),
        get_task: Arc::new(GetTaskByIdHandler::new(repo.clone())),
        get_all_tasks: Arc::new(ListTaskQueryHandler::new(repo.clone())),
    };

    Ok(state)
}

async fn serve_http(state: AppState, addr: &str) -> Result<()> {
    let app = http::router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context(format!("bind {addr}"))?;

    info!(addr, "HTTP server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server")?;

    info!("server stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c    => info!("received Ctrl+C"),
        _ = terminate => info!("received SIGTERM"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    init_tracing();

    let app = AppCli::parse();
    let state = build_app_state().await?;

    match app.mode {
        Mode::Serve { addr } => serve_http(state, &addr).await,
        Mode::Cli { command } => {
            let output = run_command(CliCommand { command }, state)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{output}");
            Ok(())
        }
    }
}
