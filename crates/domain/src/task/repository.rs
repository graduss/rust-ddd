use async_trait::async_trait;

use super::{aggregate::Task, errors::DomainError, value_objects::TaskId};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn save(&self, task: &Task) -> Result<(), DomainError>;

    async fn find_by_id(&self, id: &TaskId) -> Option<Task>;

    async fn find_all(&self) -> Result<Vec<Task>, DomainError>;

    async fn delete_by_id(&self, id: &TaskId) -> Result<(), DomainError>;

    async fn update(&self, task: &Task) -> Result<(), DomainError>;

    async fn exists(&self, id: &TaskId) -> Result<bool, DomainError>;
}
