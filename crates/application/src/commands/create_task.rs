use crate::{error::ApplicationError, ports::EventPublisher, use_case::UseCase};
use async_trait::async_trait;
use domain::{Task, TaskDescription, TaskId, TaskRepository, TaskTitle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// --- Command DTO ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommand {
    pub title: String,
    pub description: String,
}

// --- Command Handler ---

pub struct CreateTaskHandler<R: TaskRepository, P: EventPublisher> {
    repo: Arc<R>,
    publisher: Arc<P>,
}

impl<R: TaskRepository, P: EventPublisher> CreateTaskHandler<R, P> {
    pub fn new(repo: Arc<R>, publisher: Arc<P>) -> Self {
        Self { repo, publisher }
    }
}

#[async_trait]
impl<R: TaskRepository, P: EventPublisher> UseCase for CreateTaskHandler<R, P> {
    type Input = CreateTaskCommand;
    type Output = TaskId;

    async fn execute(&self, input: CreateTaskCommand) -> Result<TaskId, ApplicationError> {
        let title = TaskTitle::try_from(input.title.as_str())?;
        let description = TaskDescription::try_from(input.description.as_str())?;

        let mut task = Task::create(title, description)?;
        let id = task.id().clone();

        self.repo.save(&task).await?;
        for event in task.extract_events() {
            self.publisher.publish(event).await?;
        }

        Ok(id)
    }
}
