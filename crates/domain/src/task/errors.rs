use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Title can not be empty")]
    EmptyTitle,

    #[error("Title is too long: {0} chars (max: 255)")]
    TitleTooLong(usize),

    #[error("Description is too long: {0} chars (max: 2000)")]
    DescriptionTooLong(usize),

    #[error("Task is already completed")]
    TaskAlreadyCompleted,

    #[error("Task is already canceled")]
    TaskAlreadyCanceled,

    #[error("Cannot reopen a cancelled task")]
    CannotReopenCancelledTask,

    #[error("Task not started")]
    TaskNotStarted,

    #[error("Task not found: {0}")]
    TaskNotFound(String),
}
