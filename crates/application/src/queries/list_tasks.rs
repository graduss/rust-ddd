use super::dto::TaskDto;
use crate::{error::ApplicationError, use_case::UseCase};
use domain::{TaskRepository, TaskStatus};
use std::sync::Arc;

pub struct ListTaskQuery {
    pub status_filter: Option<TaskStatus>,
}

pub struct ListTaskQueryHandler<R: TaskRepository> {
    repo: Arc<R>,
}

impl<R: TaskRepository> ListTaskQueryHandler<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl<R: TaskRepository> UseCase for ListTaskQueryHandler<R> {
    type Input = ListTaskQuery;
    type Output = Vec<TaskDto>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ApplicationError> {
        let tasks = self.repo.find_all().await?;
        let filtered_tasks = tasks
            .iter()
            .filter(|task| {
                input
                    .status_filter
                    .as_ref()
                    .map_or(true, |s| s == task.status())
            })
            .map(|task| TaskDto::from(task))
            .collect::<Vec<_>>();
        Ok(filtered_tasks)
    }
}
