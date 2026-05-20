use application::{
    commands::{
        change_status::ChangeStatusHandler, create_task::CreateTaskHandler,
        delete_task::DeleteTaskHandler,
    },
    mocks::{MockEventPublisher, MockTaskRepository, init_di},
    queries::{get_task::GetTaskByIdHandler, list_tasks::ListTaskQueryHandler},
};
use axum::body::{Body, to_bytes};
use interface::AppState;
use std::sync::Arc;

pub fn build_test_state() -> (AppState, Arc<MockTaskRepository>, Arc<MockEventPublisher>) {
    let (repo, publisher) = init_di();

    let state = AppState {
        create_task: Arc::new(CreateTaskHandler::new(repo.clone(), publisher.clone())),
        chenge_status: Arc::new(ChangeStatusHandler::new(repo.clone(), publisher.clone())),
        delete_task: Arc::new(DeleteTaskHandler::new(repo.clone())),
        get_task: Arc::new(GetTaskByIdHandler::new(repo.clone())),
        get_all_tasks: Arc::new(ListTaskQueryHandler::new(repo.clone())),
    };

    (state, repo, publisher)
}

#[allow(dead_code)]
pub async fn body_to_json(body: Body) -> serde_json::Value {
    let body = to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
