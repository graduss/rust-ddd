mod common;
use application::mocks::seed_task;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_json, build_test_state};
use domain::{TaskRepository, TaskStatus};
use interface::http::router;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

// ── POST /tasks ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_task() {
    let (state, repo, publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Fix login",
                        "description": "The login page is broken",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(publisher.events().await.len(), 1);

    let body: serde_json::Value = body_to_json(response.into_body()).await;
    let uuid: Uuid = body["id"].as_str().unwrap().try_into().unwrap();
    let task = repo.find_by_id(&uuid.into()).await.unwrap();

    assert_eq!(task.status(), &TaskStatus::Todo);
}

#[tokio::test]
async fn test_create_task_empty_title_returns_400() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "title": "" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ── GET /health ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── GET /tasks ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_tasks_empty() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response.into_body()).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_tasks_returns_all() {
    let (state, repo, _publisher) = build_test_state();
    seed_task(repo.clone(), "Task A").await.unwrap();
    seed_task(repo.clone(), "Task B").await.unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response.into_body()).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_tasks_filter_by_status() {
    let (state, repo, _publisher) = build_test_state();
    seed_task(repo.clone(), "Todo task").await.unwrap();

    // seed a second task and advance it to InProgress via the repo directly
    let id = seed_task(repo.clone(), "In-progress task").await.unwrap();
    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks?status=todo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response.into_body()).await;
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["status"], "todo");
}

// ── GET /tasks/:id ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_task() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "My task").await.unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/tasks/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = body_to_json(response.into_body()).await;
    assert_eq!(body["title"], "My task");
    assert_eq!(body["status"], "todo");
}

#[tokio::test]
async fn test_get_task_not_found() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/tasks/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PATCH /tasks/:id/:action ──────────────────────────────────────────────────

#[tokio::test]
async fn test_update_task_start() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Start"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &TaskStatus::InProgress
    );
}

#[tokio::test]
async fn test_update_task_complete() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Complete"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &TaskStatus::Completed
    );
}

#[tokio::test]
async fn test_update_task_cancel() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &TaskStatus::Cancelled
    );
}

#[tokio::test]
async fn test_update_task_reopen() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let mut task = repo.find_by_id(&id).await.unwrap();
    task.start().unwrap();
    task.complete().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Reopen"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        repo.find_by_id(&id).await.unwrap().status(),
        &TaskStatus::Todo
    );
}

#[tokio::test]
async fn test_update_task_not_found() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{}/Start", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_task_conflict_complete_todo_returns_409() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();
    let app = router(state);

    // Complete a Todo task (must be InProgress first) → 409
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Complete"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_task_conflict_reopen_cancelled_returns_409() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task").await.unwrap();

    let mut task = repo.find_by_id(&id).await.unwrap();
    task.cancel().unwrap();
    let _ = task.extract_events();
    repo.update(&task).await.unwrap();

    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/tasks/{id}/Reopen"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── DELETE /tasks/:id ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_task() {
    let (state, repo, _publisher) = build_test_state();
    let id = seed_task(repo.clone(), "Task to delete").await.unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(repo.find_by_id(&id).await.is_none());
}

#[tokio::test]
async fn test_delete_task_not_found() {
    let (state, _repo, _publisher) = build_test_state();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
