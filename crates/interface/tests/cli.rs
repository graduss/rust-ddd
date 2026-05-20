mod common;
use application::mocks::seed_task;
use common::build_test_state;
use domain::TaskRepository;
use interface::cli::{Cli, Command, StatusActionCli, TaskStatusFilter, run_command};
use uuid::Uuid;

// ── create ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cli_create_task() {
    let (state, repo, publisher) = build_test_state();

    let output = run_command(
        Cli {
            command: Command::Create {
                title: "Buy milk".to_string(),
                description: "".to_string(),
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.starts_with("Task created:"));
    assert_eq!(publisher.events().await.len(), 1);
    assert_eq!(repo.find_all().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_cli_create_task_empty_title_returns_error() {
    let (state, _repo, _publisher) = build_test_state();

    let result = run_command(
        Cli {
            command: Command::Create {
                title: "".to_string(),
                description: "".to_string(),
            },
        },
        state,
    )
    .await;

    assert!(result.is_err());
}

// ── list ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cli_list_empty() {
    let (state, _repo, _publisher) = build_test_state();

    let output = run_command(Cli { command: Command::List { status: None } }, state)
        .await
        .unwrap();

    assert_eq!(output, "No tasks found");
}

#[tokio::test]
async fn test_cli_list_returns_all_tasks() {
    let (state, repo, _publisher) = build_test_state();
    seed_task(repo.clone(), "Task A").await.unwrap();
    seed_task(repo.clone(), "Task B").await.unwrap();

    let output = run_command(Cli { command: Command::List { status: None } }, state)
        .await
        .unwrap();

    assert!(output.contains("Task A"));
    assert!(output.contains("Task B"));
}

#[tokio::test]
async fn test_cli_list_filter_by_status() {
    let (state, repo, _publisher) = build_test_state();
    seed_task(repo.clone(), "Todo task").await.unwrap();

    let id = seed_task(repo.clone(), "In-progress task").await.unwrap();
    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let output = run_command(
        Cli {
            command: Command::List {
                status: Some(TaskStatusFilter::InProgress),
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("In-progress task"));
    assert!(!output.contains("Todo task"));
}

// ── show ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cli_show_task() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "My task").await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Show { id: *id },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("My task"));
    assert!(output.contains(&id.to_string()));
}

#[tokio::test]
async fn test_cli_show_task_not_found_returns_error() {
    let (state, _repo, _publisher) = build_test_state();

    let result = run_command(
        Cli {
            command: Command::Show { id: Uuid::new_v4() },
        },
        state,
    )
    .await;

    assert!(result.is_err());
}

// ── update ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cli_update_start() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Update {
                id: *id,
                status: StatusActionCli::Start,
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("Task updated"));
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &domain::TaskStatus::InProgress
    );
}

#[tokio::test]
async fn test_cli_update_complete() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Update {
                id: *id,
                status: StatusActionCli::Complete,
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("Task updated"));
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &domain::TaskStatus::Completed
    );
}

#[tokio::test]
async fn test_cli_update_cancel() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Update {
                id: *id,
                status: StatusActionCli::Cancel,
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("Task updated"));
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &domain::TaskStatus::Cancelled
    );
}

#[tokio::test]
async fn test_cli_update_reopen() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    task.complete().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Update {
                id: *id,
                status: StatusActionCli::Reopen,
            },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("Task updated"));
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &domain::TaskStatus::Todo
    );
}

#[tokio::test]
async fn test_cli_update_not_found_returns_error() {
    let (state, _repo, _publisher) = build_test_state();

    let result = run_command(
        Cli {
            command: Command::Update {
                id: Uuid::new_v4(),
                status: StatusActionCli::Start,
            },
        },
        state,
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_cli_update_conflict_complete_todo_returns_error() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let result = run_command(
        Cli {
            command: Command::Update {
                id: *id,
                status: StatusActionCli::Complete,
            },
        },
        state,
    )
    .await;

    assert!(result.is_err());
}

// ── delete ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cli_delete_task() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task to delete").await.unwrap();

    let output = run_command(
        Cli {
            command: Command::Delete { id: *id },
        },
        state,
    )
    .await
    .unwrap();

    assert!(output.contains("Task deleted"));
    assert!(repo.find_by_id(&id).await.is_none());
}

#[tokio::test]
async fn test_cli_delete_not_found_returns_error() {
    let (state, _repo, _publisher) = build_test_state();

    let result = run_command(
        Cli {
            command: Command::Delete { id: Uuid::new_v4() },
        },
        state,
    )
    .await;

    assert!(result.is_err());
}
