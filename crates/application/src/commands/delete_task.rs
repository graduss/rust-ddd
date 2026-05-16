use std::sync::Arc;

use domain::{TaskId, TaskRepository};
use serde::{Deserialize, Serialize};

use crate::{error::ApplicationError, use_case::UseCase};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTaskCommand {
    pub task_id: TaskId,
}

#[derive(Debug)]
pub struct DeleteTaskHandler<R: TaskRepository> {
    repo: Arc<R>,
}

impl<R: TaskRepository> DeleteTaskHandler<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl<R: TaskRepository> UseCase for DeleteTaskHandler<R> {
    type Input = DeleteTaskCommand;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ApplicationError> {
        if !self.repo.exists(&input.task_id).await.unwrap_or(false) {
            return Err(ApplicationError::NotFound("Task not found".to_string()));
        }

        self.repo.delete_by_id(&input.task_id).await?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::mocks::{MockTaskRepository, seed_task};

    #[tokio::test]
    async fn delete_existing_task() -> Result<(), ApplicationError> {
        let repo = Arc::new(MockTaskRepository::new());
        let id = seed_task(repo.clone(), "X").await?;

        let handler = DeleteTaskHandler::new(repo.clone());
        handler
            .execute(DeleteTaskCommand {
                task_id: id.clone(),
            })
            .await
            .unwrap();

        assert!(repo.find_by_id(&id).await.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_not_found() -> Result<(), ApplicationError> {
        let repo = Arc::new(MockTaskRepository::new());
        let handler = DeleteTaskHandler::new(repo);

        let res = handler
            .execute(DeleteTaskCommand {
                task_id: TaskId::new(),
            })
            .await;

        assert!(matches!(res, Err(ApplicationError::NotFound(_))));

        Ok(())
    }
}
