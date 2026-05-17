use async_trait::async_trait;

use super::{aggregate::Task, errors::DomainError, value_objects::TaskId};

/// Port (output port in hexagonal architecture) that infrastructure must implement.
///
/// Defined in the domain so that the domain never depends on any concrete storage.
/// The application layer depends on this trait; infrastructure crates provide the impl.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Persists a newly created task.
    async fn save(&self, task: &Task) -> Result<(), DomainError>;

    /// Returns `None` when the task does not exist rather than an error.
    async fn find_by_id(&self, id: &TaskId) -> Option<Task>;

    async fn find_all(&self) -> Result<Vec<Task>, DomainError>;

    async fn delete_by_id(&self, id: &TaskId) -> Result<(), DomainError>;

    /// Overwrites an existing task record (the task must already be saved).
    async fn update(&self, task: &Task) -> Result<(), DomainError>;

    async fn exists(&self, id: &TaskId) -> Result<bool, DomainError>;
}
