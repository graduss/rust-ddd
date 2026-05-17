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
