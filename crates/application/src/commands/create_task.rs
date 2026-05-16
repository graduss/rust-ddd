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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::mocks::{MockEventPublisher, MockTaskRepository, seed_task};

    #[tokio::test]
    async fn create_task_and_returns_id() {
        let repo = Arc::new(MockTaskRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());
        let handler = CreateTaskHandler::new(repo.clone(), publisher.clone());

        let command = CreateTaskCommand {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
        };

        let id = handler.execute(command).await.unwrap();
        let task = repo.find_by_id(&id).await.unwrap();
        assert_eq!(task.title(), &TaskTitle::try_from("Test Task").unwrap());

        let mut events = publisher.events().await;
        assert_eq!(events.len(), 1);
        assert!(events.pop().is_some());
    }

    #[tokio::test]
    async fn create_task_rejects_invalid_title() {
        let result = seed_task(Arc::new(MockTaskRepository::new()), "").await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Domain(domain::DomainError::EmptyTitle))
        ));
    }
}
