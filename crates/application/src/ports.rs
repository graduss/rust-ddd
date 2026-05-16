use super::error::ApplicationError;
use async_trait::async_trait;
use domain::TaskEvent;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: TaskEvent) -> Result<(), ApplicationError>;
}
