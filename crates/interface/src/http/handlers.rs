use super::dto::{CreateTaskRequest, CreateTaskResponse, ListTasksRequest};
use crate::{http::error::ApiError, state::AppState};
use application::{
    commands::{
        change_status::{ChangeStatusCommand, StatusAction},
        create_task::CreateTaskCommand,
        delete_task::DeleteTaskCommand,
    },
    queries::{dto::TaskDto, get_task::GetTaskByIdQuery, list_tasks::ListTaskQuery},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

// --- POST /tasks ---
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<CreateTaskResponse>), ApiError> {
    let task_id = state
        .create_task
        .execute(CreateTaskCommand {
            title: payload.title,
            description: payload.description,
        })
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateTaskResponse {
            id: task_id.to_string(),
        }),
    ))
}

// --- GET /tasks ---
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListTasksRequest>,
) -> Result<(StatusCode, Json<Vec<TaskDto>>), ApiError> {
    let tasks = state
        .get_all_tasks
        .execute(ListTaskQuery {
            status_filter: params.status,
        })
        .await?;
    Ok((StatusCode::OK, Json(tasks)))
}

// --- GET /tasks/:id ---
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<TaskDto>), ApiError> {
    let task = state
        .get_task
        .execute(GetTaskByIdQuery { id: id.into() })
        .await?;
    Ok((StatusCode::OK, Json(task)))
}

// --- PATCH /tasks/:id/:action ---
pub async fn update_task(
    State(state): State<AppState>,
    Path((id, action)): Path<(Uuid, StatusAction)>,
) -> Result<StatusCode, ApiError> {
    state
        .chenge_status
        .execute(ChangeStatusCommand {
            task_id: id.into(),
            action,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- DELETE /tasks/:id ---
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .delete_task
        .execute(DeleteTaskCommand { task_id: id.into() })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
