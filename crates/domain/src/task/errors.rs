use thiserror::Error;

/// All domain-rule violations that can occur within the task bounded context.
#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Title can not be empty")]
    EmptyTitle,

    #[error("Title is too long: {0} chars (max: 255)")]
    TitleTooLong(usize),

    #[error("Description is too long: {0} chars (max: 2000)")]
    DescriptionTooLong(usize),

    /// Returned when any mutating operation is attempted on a completed task.
    #[error("Task is already completed")]
    TaskAlreadyCompleted,

    /// Returned when any mutating operation is attempted on a cancelled task.
    #[error("Task is already canceled")]
    TaskAlreadyCanceled,

    /// `cancel()` is the only terminal transition for `Cancelled`; reopening is not allowed.
    #[error("Cannot reopen a cancelled task")]
    CannotReopenCancelledTask,

    /// `complete()` requires the task to be `InProgress` first.
    #[error("Task not started")]
    TaskNotStarted,

    #[error("Task not found: {0}")]
    TaskNotFound(String),
}
