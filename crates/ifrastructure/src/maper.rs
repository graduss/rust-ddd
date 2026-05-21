use chrono::{DateTime, Utc};
use domain::{DomainError, Task, TaskDescription, TaskStatus, TaskTitle};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum TaskStatusSqlx {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

impl Into<TaskStatus> for TaskStatusSqlx {
    fn into(self) -> TaskStatus {
        match self {
            TaskStatusSqlx::Todo => TaskStatus::Todo,
            TaskStatusSqlx::InProgress => TaskStatus::InProgress,
            TaskStatusSqlx::Completed => TaskStatus::Completed,
            TaskStatusSqlx::Cancelled => TaskStatus::Cancelled,
        }
    }
}

impl From<TaskStatus> for TaskStatusSqlx {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Todo => TaskStatusSqlx::Todo,
            TaskStatus::InProgress => TaskStatusSqlx::InProgress,
            TaskStatus::Completed => TaskStatusSqlx::Completed,
            TaskStatus::Cancelled => TaskStatusSqlx::Cancelled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskSqlx {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatusSqlx,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryInto<Task> for TaskSqlx {
    type Error = DomainError;

    fn try_into(self) -> Result<Task, Self::Error> {
        Ok(Task::recover(
            self.id.into(),
            TaskTitle::try_from(self.title.as_str())?,
            TaskDescription::try_from(self.description.as_str())?,
            self.status.into(),
            self.created_at,
            self.updated_at,
        ))
    }
}
