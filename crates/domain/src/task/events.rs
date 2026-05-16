use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::{TaskDescription, TaskId, TaskStatus, TaskTitle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreated {
    pub id: TaskId,
    pub title: TaskTitle,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusChanged {
    pub id: TaskId,
    pub new_status: TaskStatus,
    pub previous_status: TaskStatus,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdated {
    pub id: TaskId,
    pub title: Option<TaskTitle>,
    pub description: Option<TaskDescription>,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDeleted {
    pub id: TaskId,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    Created(TaskCreated),
    StatusChanged(TaskStatusChanged),
    Updated(TaskUpdated),
    Deleted(TaskDeleted),
}
