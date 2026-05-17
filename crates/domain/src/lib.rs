//! Pure domain layer for the task-manager workspace.
//!
//! Contains the `Task` aggregate, value objects, domain events, errors, and the
//! `TaskRepository` port. No I/O or infrastructure dependencies are allowed here.

mod task;

pub use task::{
    aggregate::Task, errors::DomainError, events::TaskEvent, repository::TaskRepository,
    value_objects::*,
};
