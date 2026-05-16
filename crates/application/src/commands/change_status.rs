use std::sync::Arc;

use domain::{TaskId, TaskRepository};
use serde::{Deserialize, Serialize};

use crate::{error::ApplicationError, ports::EventPublisher, use_case::UseCase};

#[derive(Debug, Deserialize, Serialize)]
pub enum StatusAction {
    Start,
    Complete,
    Cancel,
    Reopen,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangeStatusCommand {
    pub task_id: TaskId,
    pub action: StatusAction,
}

pub struct ChangeStatusHandler<R: TaskRepository, P: EventPublisher> {
    repo: Arc<R>,
    publisher: Arc<P>,
}

impl<R: TaskRepository, P: EventPublisher> ChangeStatusHandler<R, P> {
    pub fn new(repo: Arc<R>, publisher: Arc<P>) -> Self {
        Self { repo, publisher }
    }
}

#[async_trait::async_trait]
impl<R: TaskRepository, P: EventPublisher> UseCase for ChangeStatusHandler<R, P> {
    type Input = ChangeStatusCommand;
    type Output = ();

    async fn execute(&self, input: ChangeStatusCommand) -> Result<(), ApplicationError> {
        let mut task =
            self.repo
                .find_by_id(&input.task_id)
                .await
                .ok_or(ApplicationError::NotFound(format!(
                    "Task not found: {}",
                    &input.task_id
                )))?;

        match input.action {
            StatusAction::Start => task.start(),
            StatusAction::Complete => task.complete(),
            StatusAction::Cancel => task.cancel(),
            StatusAction::Reopen => task.reopen(),
        }?;

        self.repo.update(&task).await?;

        for event in task.extract_events() {
            self.publisher.publish(event).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::mocks::{init_di, seed_task};
    use domain::{DomainError, TaskId, TaskStatus};

    #[tokio::test]
    async fn test_change_status() -> Result<(), ApplicationError> {
        let (repo, publisher) = init_di();
        let handler = ChangeStatusHandler::new(repo.clone(), publisher.clone());
        let id = seed_task(repo.clone(), "test").await?;
        let cmd = ChangeStatusCommand {
            task_id: id.clone(),
            action: StatusAction::Start,
        };
        handler.execute(cmd).await?;
        let task = repo.find_by_id(&id).await.unwrap();
        assert_eq!(task.status(), &TaskStatus::InProgress);

        let events = publisher.events().await;
        assert_eq!(events.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_change_status_error() -> Result<(), ApplicationError> {
        let (repo, publisher) = init_di();
        let handler = ChangeStatusHandler::new(repo.clone(), publisher.clone());
        let task_id = seed_task(repo, "test_change_status_error").await?;
        let cmd = ChangeStatusCommand {
            task_id: task_id.clone(),
            action: StatusAction::Complete,
        };

        let res = handler.execute(cmd).await;
        assert!(res.is_err());
        assert!(matches!(
            res,
            Err(ApplicationError::Domain(DomainError::TaskNotStarted))
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_not_found() -> Result<(), ApplicationError> {
        let (repo, publisher) = init_di();
        let handler = ChangeStatusHandler::new(repo.clone(), publisher.clone());
        let cmd = ChangeStatusCommand {
            task_id: TaskId::new(),
            action: StatusAction::Start,
        };

        let res = handler.execute(cmd).await;
        assert!(res.is_err());
        assert!(matches!(res, Err(ApplicationError::NotFound(_))));

        Ok(())
    }
}
