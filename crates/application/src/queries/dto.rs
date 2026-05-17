use chrono::{DateTime, Utc};
use domain::{Task, TaskDescription, TaskId, TaskStatus, TaskTitle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: TaskId,
    pub title: TaskTitle,
    pub description: TaskDescription,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Task> for TaskDto {
    fn from(task: &Task) -> Self {
        TaskDto {
            id: task.id().clone(),
            title: task.title().clone(),
            description: task.description().clone(),
            status: task.status().clone(),
            created_at: task.created_at().clone(),
            updated_at: task.updated_at().clone(),
        }
    }
}
