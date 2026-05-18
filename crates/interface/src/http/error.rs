use application::error::ApplicationError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::DomainError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

pub struct ApiError {
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::NotFound(message) => Self {
                status: StatusCode::NOT_FOUND,
                body: ApiErrorBody {
                    code: "NOT_FOUND",
                    message,
                },
            },
            ApplicationError::Domain(error) => error.into(),
            ApplicationError::Repository(message) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiErrorBody {
                    code: "REPOSITORY_ERROR",
                    message,
                },
            },
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::EmptyTitle
            | DomainError::TitleTooLong(_)
            | DomainError::DescriptionTooLong(_) => Self {
                status: StatusCode::BAD_REQUEST,
                body: ApiErrorBody {
                    code: "EMPTY_TITLE",
                    message: error.to_string(),
                },
            },

            DomainError::TaskAlreadyCanceled
            | DomainError::TaskNotStarted
            | DomainError::TaskAlreadyCompleted
            | DomainError::CannotReopenCancelledTask => Self {
                status: StatusCode::CONFLICT,
                body: ApiErrorBody {
                    code: "BUSINESS_RULE_ERROR",
                    message: error.to_string(),
                },
            },

            DomainError::TaskNotFound(_) => Self {
                status: StatusCode::NOT_FOUND,
                body: ApiErrorBody {
                    code: "NOT_FOUND",
                    message: error.to_string(),
                },
            },
        }
    }
}
