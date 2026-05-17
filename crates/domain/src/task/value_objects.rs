use std::ops::Deref;

use super::errors::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TITLE_LENGTH: usize = 255;
const MAX_DESCRIPTION_LENGTH: usize = 2000;

// --- TaskId -----

/// Newtype wrapper around [`Uuid`] that prevents accidental mixing with other IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct TaskId(Uuid);

impl TaskId {
    /// Generates a new random v4 UUID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for TaskId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<&Uuid> for TaskId {
    fn from(uuid: &Uuid) -> Self {
        uuid.clone().into()
    }
}

impl Deref for TaskId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- TaskTitle -----

/// A non-empty, trimmed task title of at most 255 characters.
///
/// Constructed via `TryFrom<&str>`; invariants are enforced at the boundary so callers
/// can trust that any `TaskTitle` value is valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTitle(String);

impl TryFrom<&str> for TaskTitle {
    type Error = DomainError;

    fn try_from(title: &str) -> Result<Self, Self::Error> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        if title.len() > MAX_TITLE_LENGTH {
            return Err(DomainError::TitleTooLong(title.len()));
        }
        Ok(Self(title.to_owned()))
    }
}

impl Deref for TaskTitle {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TaskTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- TaskDescription -----

/// An optional task description of at most 2000 characters (empty string is allowed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDescription(String);

impl TryFrom<&str> for TaskDescription {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let description = value;
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(DomainError::DescriptionTooLong(description.len()));
        }
        Ok(Self(description.to_owned()))
    }
}

impl Deref for TaskDescription {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TaskDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- TaskStatus -----

/// Lifecycle state of a task.
///
/// ```text
/// Todo ──start()──► InProgress ──complete()──► Completed ──reopen()──► Todo
///  │                    │
///  └───cancel()────────►┘──► Cancelled
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    /// Returns `true` for terminal states where edits are no longer allowed.
    ///
    /// Both `Completed` and `Cancelled` are terminal — the name is slightly misleading
    /// but intentional: it signals "done in some final sense."
    pub fn is_completed(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Cancelled)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "todo"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- TaskID ---
    #[test]
    fn test_task_id_uniqueness() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_id_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id: TaskId = uuid.into();
        assert_eq!(&id as &Uuid, &uuid);
    }

    #[test]
    fn test_task_id_display() {
        const EXPECTED: &str = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = Uuid::parse_str(EXPECTED).unwrap();
        let id: TaskId = uuid.into();
        assert_eq!(format!("{}", id), EXPECTED);
    }

    // --- TaskTitle ---
    #[test]
    fn test_task_title_value() {
        let title: TaskTitle = "Test Title".try_into().unwrap();
        assert_eq!(&title as &str, "Test Title");
        assert_eq!(title.to_string(), "Test Title");
    }

    #[test]
    fn title_trims_whitespace() {
        let t: TaskTitle = "  spaces around  ".try_into().unwrap();
        assert_eq!(&t as &str, "spaces around");
    }

    #[test]
    fn title_rejects_empty() {
        assert_eq!(TaskTitle::try_from(""), Err(DomainError::EmptyTitle));
        assert_eq!(TaskTitle::try_from("   "), Err(DomainError::EmptyTitle));
    }

    #[test]
    fn title_rejects_too_long() {
        let long = "a".repeat(256);
        assert_eq!(
            TaskTitle::try_from(&long as &str),
            Err(DomainError::TitleTooLong(256))
        );
    }

    #[test]
    fn title_accepts_exactly_255_chars() {
        let max = "a".repeat(255);
        assert!(TaskTitle::try_from(&max as &str).is_ok());
    }

    #[test]
    fn title_equality_by_value() {
        let t1 = TaskTitle::try_from("Same title").unwrap();
        let t2 = TaskTitle::try_from("Same title").unwrap();
        assert_eq!(t1, t2, "Titles with same value must be equal");
    }

    // ── Description ──

    #[test]
    fn description_valid() {
        let d = TaskDescription::try_from("Some description").unwrap();
        assert_eq!(&d as &str, "Some description");
        assert_eq!(d.to_string(), "Some description");
    }

    #[test]
    fn description_rejects_too_long() {
        let long = "x".repeat(2001);
        assert_eq!(
            TaskDescription::try_from(&long as &str),
            Err(DomainError::DescriptionTooLong(2001))
        );
    }

    // ── TaskStatus ──

    #[test]
    fn status_terminal_states() {
        assert!(TaskStatus::Completed.is_completed());
        assert!(TaskStatus::Cancelled.is_completed());
        assert!(!TaskStatus::Todo.is_completed());
        assert!(!TaskStatus::InProgress.is_completed());
    }

    #[test]
    fn status_display() {
        assert_eq!(TaskStatus::InProgress.to_string(), "in_progress");
        assert_eq!(TaskStatus::Todo.to_string(), "todo");
        assert_eq!(TaskStatus::Completed.to_string(), "completed");
        assert_eq!(TaskStatus::Cancelled.to_string(), "cancelled");
    }
}
