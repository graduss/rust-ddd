use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::{TaskDescription, TaskId, TaskStatus, TaskTitle};

/// Emitted by [`Task::create`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreated {
    pub id: TaskId,
    pub title: TaskTitle,
    pub created_at: DateTime<Utc>,
}

/// Emitted by every successful state-machine transition (`start`, `complete`, `cancel`, `reopen`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusChanged {
    pub id: TaskId,
    pub new_status: TaskStatus,
    pub previous_status: TaskStatus,
    pub changed_at: DateTime<Utc>,
}

/// Emitted by [`Task::update`]; fields are `None` when they were not changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdated {
    pub id: TaskId,
    pub title: Option<TaskTitle>,
    pub description: Option<TaskDescription>,
    pub changed_at: DateTime<Utc>,
}

/// Emitted externally (e.g. by the delete command handler) before the task is removed from the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDeleted {
    pub id: TaskId,
    pub changed_at: DateTime<Utc>,
}

/// Discriminated union of all events the task aggregate can produce.
///
/// Events are accumulated inside [`Task`] and drained via [`Task::extract_events`].
/// The application layer is responsible for dispatching them after the use case completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    Created(TaskCreated),
    StatusChanged(TaskStatusChanged),
    Updated(TaskUpdated),
    Deleted(TaskDeleted),
}
