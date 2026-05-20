use application::{
    commands::{
        change_status::{ChangeStatusCommand, StatusAction},
        create_task::CreateTaskCommand,
        delete_task::DeleteTaskCommand,
    },
    queries::{get_task::GetTaskByIdQuery, list_tasks::ListTaskQuery},
};
use clap::{Parser, Subcommand, ValueEnum};
use domain::TaskStatus;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Parser, Debug)]
#[command(name = "task", version, about = "Task management CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Create {
        #[arg(short, long)]
        title: String,
        #[arg(short, long, default_value = "")]
        description: String,
    },
    List {
        status: Option<TaskStatusFilter>,
    },
    Show {
        id: Uuid,
    },
    Update {
        id: Uuid,
        status: StatusActionCli,
    },
    Delete {
        id: Uuid,
    },
}

pub async fn run_command(cli: Cli, state: AppState) -> anyhow::Result<String> {
    match cli.command {
        Command::Create { title, description } => {
            let id = state
                .create_task
                .execute(CreateTaskCommand { title, description })
                .await?;
            Ok(format!("Task created: {id}"))
        }
        Command::List { status } => {
            let list = state
                .get_all_tasks
                .execute(ListTaskQuery {
                    status_filter: status.map(|s| s.into()),
                })
                .await?;
            if list.is_empty() {
                Ok("No tasks found".to_string())
            } else {
                Ok(list
                    .into_iter()
                    .map(|t| format!("[{}] - {} - {}", t.status, t.id, t.title))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
        Command::Show { id } => {
            let task = state
                .get_task
                .execute(GetTaskByIdQuery { id: id.into() })
                .await?;

            Ok(format!("Task: {} - {}", task.id, task.title))
        }
        Command::Update { id, status } => {
            state
                .chenge_status
                .execute(ChangeStatusCommand {
                    task_id: id.into(),
                    action: status.into(),
                })
                .await?;

            Ok(format!("Task updated: {id}"))
        }
        Command::Delete { id } => {
            state
                .delete_task
                .execute(DeleteTaskCommand { task_id: id.into() })
                .await?;

            Ok(format!("Task deleted: {id}"))
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
#[value(rename_all = "kebab-case")]
pub enum StatusActionCli {
    Start,
    Complete,
    Cancel,
    Reopen,
}

impl Into<StatusAction> for StatusActionCli {
    fn into(self) -> StatusAction {
        match self {
            StatusActionCli::Start => StatusAction::Start,
            StatusActionCli::Complete => StatusAction::Complete,
            StatusActionCli::Cancel => StatusAction::Cancel,
            StatusActionCli::Reopen => StatusAction::Reopen,
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
#[value(rename_all = "kebab-case")]
pub enum TaskStatusFilter {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

impl Into<TaskStatus> for TaskStatusFilter {
    fn into(self) -> TaskStatus {
        match self {
            TaskStatusFilter::Todo => TaskStatus::Todo,
            TaskStatusFilter::InProgress => TaskStatus::InProgress,
            TaskStatusFilter::Completed => TaskStatus::Completed,
            TaskStatusFilter::Cancelled => TaskStatus::Cancelled,
        }
    }
}
