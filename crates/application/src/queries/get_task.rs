use super::dto::TaskDto;
use crate::{error::ApplicationError, use_case::UseCase};
use domain::{TaskId, TaskRepository};
use std::sync::Arc;

pub struct GetTaskByIdQuery {
    pub id: TaskId,
}

pub struct GetTaskByIdHandler<R: TaskRepository> {
    repo: Arc<R>,
}

impl<R: TaskRepository> GetTaskByIdHandler<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl<R: TaskRepository> UseCase for GetTaskByIdHandler<R> {
    type Input = GetTaskByIdQuery;
    type Output = TaskDto;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ApplicationError> {
        let task = self
            .repo
            .find_by_id(&input.id)
            .await
            .ok_or(ApplicationError::NotFound("Task not found".into()))?;

        Ok(TaskDto::from(&task))
    }
}
