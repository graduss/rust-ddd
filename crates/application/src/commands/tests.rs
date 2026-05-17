use std::sync::Arc;

use crate::{
    error::ApplicationError,
    mocks::{MockTaskRepository, init_di, seed_task},
    use_case::UseCase,
};

use super::{
    change_status::{ChangeStatusCommand, ChangeStatusHandler, StatusAction},
    create_task::{CreateTaskCommand, CreateTaskHandler},
    delete_task::{DeleteTaskCommand, DeleteTaskHandler},
};
use domain::{DomainError, TaskId, TaskRepository, TaskStatus, TaskTitle};
use tokio::test;

//--- ChangeStatusHandler ----
#[test]
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

#[test]
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

#[test]
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

// --- CreateTaskHandler ---
#[test]
async fn create_task_and_returns_id() {
    let (repo, publisher) = init_di();
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

#[test]
async fn create_task_rejects_invalid_title() {
    let result = seed_task(Arc::new(MockTaskRepository::new()), "").await;
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(ApplicationError::Domain(domain::DomainError::EmptyTitle))
    ));
}

// --- DeleteTaskHandler ---
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
