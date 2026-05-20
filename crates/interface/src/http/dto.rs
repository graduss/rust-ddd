use domain::TaskStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksRequest {
    pub status: Option<TaskStatus>,
}
