mod common;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{body_to_json, build_test_state};
use domain::{TaskRepository, TaskStatus};
use interface::http::router::router;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

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
