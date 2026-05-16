use crate::error::ApplicationError;
use std::sync::Arc;

use super::ports::EventPublisher;
use async_trait::async_trait;
use domain::{DomainError, Task, TaskDescription, TaskEvent, TaskId, TaskRepository, TaskTitle};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct MockTaskRepository {
    inner: Mutex<Vec<Task>>,
}

impl MockTaskRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn count(&self) -> usize {
        self.inner.lock().await.len()
    }
}

#[async_trait]
impl TaskRepository for MockTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), DomainError> {
        self.inner.lock().await.push(task.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &TaskId) -> Option<Task> {
        let tasks = self.inner.lock().await.clone();
        tasks.into_iter().find(|t| t.id() == id)
    }

    async fn find_all(&self) -> Result<Vec<Task>, DomainError> {
        let tasks = self.inner.lock().await.clone();
        Ok(tasks)
    }

    async fn delete_by_id(&self, id: &TaskId) -> Result<(), DomainError> {
        let mut tasks = self.inner.lock().await;
        tasks.retain(|t| t.id() != id);
        Ok(())
    }

    async fn update(&self, task: &Task) -> Result<(), DomainError> {
        let mut guard = self.inner.lock().await;
        if let Some(existing) = guard.iter_mut().find(|t| t.id() == task.id()) {
            *existing = task.clone();
            Ok(())
        } else {
            Err(DomainError::TaskNotFound(task.id().to_string()))
        }
    }

    async fn exists(&self, id: &TaskId) -> Result<bool, DomainError> {
        Ok(self.inner.lock().await.iter().any(|t| t.id() == id))
    }
}

#[derive(Default)]
pub struct MockEventPublisher {
    inner: Mutex<Vec<TaskEvent>>,
}

impl MockEventPublisher {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn events(&self) -> Vec<TaskEvent> {
        self.inner.lock().await.clone()
    }
}

#[async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, event: TaskEvent) -> Result<(), ApplicationError> {
        self.inner.lock().await.push(event);
        Ok(())
    }
}

pub async fn seed_tesk(
    repo: Arc<MockTaskRepository>,
    title: &str,
) -> Result<TaskId, ApplicationError> {
    let mut task = Task::create(TaskTitle::try_from(title)?, TaskDescription::try_from("")?)?;
    let id = task.id().clone();
    let _ = task.extract_events();
    repo.save(&task).await?;
    Ok(id)
}
