use super::handlers;
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, patch},
};
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/tasks",
            get(handlers::list_tasks).post(handlers::create_task),
        )
        .route(
            "/tasks/{id}",
            get(handlers::get_task).delete(handlers::delete_task),
        )
        .route("/tasks/{id}/{action}", patch(handlers::update_task))
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
