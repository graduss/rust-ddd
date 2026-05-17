use super::get_task::{GetTaskByIdHandler, GetTaskByIdQuery};
use super::list_tasks::{ListTaskQuery, ListTaskQueryHandler};
use crate::{
    error::ApplicationError,
    mocks::{MockTaskRepository, seed_task},
    use_case::UseCase,
};
use domain::{TaskId, TaskRepository, TaskStatus};
use std::sync::Arc;
use tokio::test;

//--- GetTaskByIdHandler
#[test]
async fn test_get_task_by_id() -> Result<(), ApplicationError> {
    let repo = Arc::new(MockTaskRepository::new());
    let id = seed_task(repo.clone(), "Test task").await?;
    let handler = GetTaskByIdHandler::new(repo);
    let dto = handler.execute(GetTaskByIdQuery { id: id.clone() }).await?;

    assert_eq!(dto.id, id);
    assert_eq!(dto.title.to_string(), "Test task");
    assert_eq!(dto.status, TaskStatus::Todo);

    Ok(())
}

#[test]
async fn test_get_task_by_id_not_found() -> Result<(), ApplicationError> {
    let repo = Arc::new(MockTaskRepository::new());
    let handler = GetTaskByIdHandler::new(repo);
    let result = handler
        .execute(GetTaskByIdQuery { id: TaskId::new() })
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(ApplicationError::NotFound(_))));

    Ok(())
}

// --- ListTaskQueryHandler
#[tokio::test]
async fn test_list_tasks() {
    let repo = Arc::new(MockTaskRepository::new());
    let handler = ListTaskQueryHandler::new(repo.clone());
    seed_task(repo.clone(), "Test Task1").await.unwrap();
    seed_task(repo.clone(), "Test Task2").await.unwrap();

    let result = handler
        .execute(ListTaskQuery {
            status_filter: None,
        })
        .await;
    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].title.to_string(), "Test Task1");
    assert_eq!(tasks[1].title.to_string(), "Test Task2");
}

#[tokio::test]
async fn test_list_tasks_with_filter() -> Result<(), ApplicationError> {
    let repo = Arc::new(MockTaskRepository::new());
    let handler = ListTaskQueryHandler::new(repo.clone());
    seed_task(repo.clone(), "Test Task1").await?;
    let task2_id = seed_task(repo.clone(), "Test Task2").await?;
    let mut task2 = repo
        .find_by_id(&task2_id)
        .await
        .ok_or(ApplicationError::NotFound("Task not found".into()))?;
    task2.start()?;
    repo.update(&task2).await?;

    let result = handler
        .execute(ListTaskQuery {
            status_filter: Some(TaskStatus::InProgress),
        })
        .await;
    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title.to_string(), "Test Task2");
    Ok(())
}

#[tokio::test]
async fn test_empty_list() -> Result<(), ApplicationError> {
    let repo = Arc::new(MockTaskRepository::new());
    let handler = ListTaskQueryHandler::new(repo.clone());
    let result = handler
        .execute(ListTaskQuery {
            status_filter: None,
        })
        .await;
    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 0);
    Ok(())
}
