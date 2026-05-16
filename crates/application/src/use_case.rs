use super::error::ApplicationError;
use async_trait::async_trait;

#[async_trait]
pub trait UseCase: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ApplicationError>;
}
