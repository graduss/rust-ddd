use crate::task::{
    errors::DomainError,
    events::{TaskCreated, TaskEvent, TaskStatusChanged, TaskUpdated},
    value_objects::{TaskDescription, TaskId, TaskStatus, TaskTitle},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    // Identity
    id: TaskId,

    // Attributes
    title: TaskTitle,
    description: TaskDescription,
    status: TaskStatus,

    // Auditing
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,

    #[serde(skip)]
    domain_events: Vec<TaskEvent>,
}

impl Task {
    pub fn create(title: TaskTitle, description: TaskDescription) -> Result<Self, DomainError> {
        let mut task = Self {
            id: TaskId::new(),
            title,
            description,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            domain_events: Vec::new(),
        };

        task.notify(TaskEvent::Created(TaskCreated {
            id: task.id.clone(),
            title: task.title.clone(),
            created_at: task.created_at.clone(),
        }));

        Ok(task)
    }

    pub fn recover(
        id: TaskId,
        title: TaskTitle,
        description: TaskDescription,
        status: TaskStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            status,
            created_at,
            updated_at,
            domain_events: Vec::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), DomainError> {
        match self.status {
            TaskStatus::Cancelled => Err(DomainError::TaskAlreadyCanceled),
            TaskStatus::Completed => Err(DomainError::TaskAlreadyCompleted),
            TaskStatus::InProgress => Ok(()),
            TaskStatus::Todo => {
                self.transition_to(TaskStatus::InProgress);
                Ok(())
            }
        }
    }

    pub fn complete(&mut self) -> Result<(), DomainError> {
        match self.status {
            TaskStatus::Cancelled => Err(DomainError::TaskAlreadyCanceled),
            TaskStatus::Completed => Ok(()),
            TaskStatus::Todo => Err(DomainError::TaskNotStarted),
            TaskStatus::InProgress => {
                self.transition_to(TaskStatus::Completed);
                Ok(())
            }
        }
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        match self.status {
            TaskStatus::Cancelled => Ok(()),
            TaskStatus::Completed => Err(DomainError::TaskAlreadyCompleted),
            TaskStatus::InProgress | TaskStatus::Todo => {
                self.transition_to(TaskStatus::Cancelled);
                Ok(())
            }
        }
    }

    pub fn reopen(&mut self) -> Result<(), DomainError> {
        match self.status {
            TaskStatus::Cancelled => Err(DomainError::CannotReopenCancelledTask),
            TaskStatus::InProgress | TaskStatus::Todo => Ok(()),
            TaskStatus::Completed => {
                self.transition_to(TaskStatus::Todo);
                Ok(())
            }
        }
    }

    pub fn update(
        &mut self,
        title: Option<TaskTitle>,
        description: Option<TaskDescription>,
    ) -> Result<(), DomainError> {
        if self.status.is_completed() {
            return Err(DomainError::TaskAlreadyCompleted);
        }

        match (title, description) {
            (Some(title), Some(description)) => {
                self.title = title.clone();
                self.description = description.clone();
                self.updated_at = Utc::now();
                self.notify(TaskEvent::Updated(TaskUpdated {
                    id: self.id.clone(),
                    title: Some(title),
                    description: Some(description),
                    changed_at: self.updated_at.clone(),
                }));
            }

            (Some(title), None) => {
                self.title = title.clone();
                self.updated_at = Utc::now();
                self.notify(TaskEvent::Updated(TaskUpdated {
                    id: self.id.clone(),
                    title: Some(title),
                    description: None,
                    changed_at: self.updated_at.clone(),
                }));
            }

            (None, Some(description)) => {
                self.description = description.clone();
                self.updated_at = Utc::now();
                self.notify(TaskEvent::Updated(TaskUpdated {
                    id: self.id.clone(),
                    title: None,
                    description: Some(description),
                    changed_at: self.updated_at.clone(),
                }));
            }

            _ => (),
        }

        Ok(())
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn title(&self) -> &TaskTitle {
        &self.title
    }

    pub fn description(&self) -> &TaskDescription {
        &self.description
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn pending_events(&self) -> &[TaskEvent] {
        &self.domain_events
    }

    pub fn extract_events(&mut self) -> Vec<TaskEvent> {
        std::mem::take(&mut self.domain_events)
    }

    fn transition_to(&mut self, status: TaskStatus) {
        let previous_status = self.status.clone();
        self.status = status;
        self.updated_at = Utc::now();

        self.notify(TaskEvent::StatusChanged(TaskStatusChanged {
            id: self.id.clone(),
            new_status: self.status.clone(),
            previous_status,
            changed_at: self.updated_at.clone(),
        }));
    }

    fn notify(&mut self, event: TaskEvent) {
        self.domain_events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task() -> Task {
        Task::create(
            TaskTitle::try_from("Fix login bug").unwrap(),
            TaskDescription::try_from("").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn create_task_has_correct_defaults() {
        let task = make_task();
        assert_eq!(task.title().to_string(), "Fix login bug");
        assert_eq!(task.status(), &TaskStatus::Todo);
    }

    #[test]
    fn create_task_generates_created_event() {
        let task = make_task();
        assert_eq!(task.pending_events().len(), 1);
        assert!(matches!(task.pending_events()[0], TaskEvent::Created(_)));
    }

    // ── Переходы состояний ──

    #[test]
    fn start_task_transitions_to_in_progress() {
        let mut task = make_task();
        task.start().unwrap();
        assert_eq!(task.status(), &TaskStatus::InProgress);
    }

    #[test]
    fn complete_task_from_todo() {
        let mut task = make_task();
        assert_eq!(task.complete(), Err(DomainError::TaskNotStarted));
    }

    #[test]
    fn complete_task_from_in_progress() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();
        assert_eq!(task.status(), &TaskStatus::Completed);
    }

    #[test]
    fn cancel_task() {
        let mut task = make_task();
        task.cancel().unwrap();
        assert_eq!(task.status(), &TaskStatus::Cancelled);
    }

    // ── Инварианты (бизнес-правила) ──

    #[test]
    fn cannot_complete_cancelled_task() {
        let mut task = make_task();
        task.cancel().unwrap();
        let err = task.complete().unwrap_err();
        assert_eq!(err, DomainError::TaskAlreadyCanceled);
    }

    #[test]
    fn cannot_start_completed_task() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();
        let err = task.start().unwrap_err();
        assert_eq!(err, DomainError::TaskAlreadyCompleted);
    }

    #[test]
    fn cannot_start_cancelled_task() {
        let mut task = make_task();
        task.cancel().unwrap();
        let err = task.start().unwrap_err();
        assert_eq!(err, DomainError::TaskAlreadyCanceled);
    }

    #[test]
    fn cannot_cancel_completed_task() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();
        assert_eq!(task.cancel(), Err(DomainError::TaskAlreadyCompleted));
    }

    #[test]
    fn cannot_reopen_cancelled_task() {
        let mut task = make_task();
        task.cancel().unwrap();
        let err = task.reopen().unwrap_err();
        assert_eq!(err, DomainError::CannotReopenCancelledTask);
    }

    #[test]
    fn can_reopen_completed_task() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();
        task.reopen().unwrap();
        assert_eq!(task.status(), &TaskStatus::Todo);
    }

    // ── Domain Events ──

    #[test]
    fn state_transitions_generate_events() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();

        // Created + StatusChanged(Todo→InProgress) + StatusChanged(InProgress→Done)
        assert_eq!(task.pending_events().len(), 3);
    }

    // ── Обновление полей ──

    #[test]
    fn update_title_changes_title_and_generates_event() {
        let mut task = make_task();

        let new_title = TaskTitle::try_from("Updated title").unwrap();
        task.update(Some(new_title.clone()), None).unwrap();

        assert_eq!(task.title(), &new_title);
        assert_eq!(task.pending_events().len(), 2);
    }

    #[test]
    fn cannot_update_title_of_completed_task() {
        let mut task = make_task();
        task.start().unwrap();
        task.complete().unwrap();

        let err = task
            .update(Some(TaskTitle::try_from("New title").unwrap()), None)
            .unwrap_err();
        assert_eq!(err, DomainError::TaskAlreadyCompleted);
    }

    // ── Идемпотентность ──

    #[test]
    fn start_already_in_progress_task_is_idempotent() {
        let mut task = make_task();
        task.start().unwrap();
        task.start().unwrap(); // второй вызов — без ошибки
        assert_eq!(task.status(), &TaskStatus::InProgress);
    }

    // ── Reconstitute ──

    #[test]
    fn reconstitute_does_not_generate_events() {
        let id = TaskId::new();
        let now = Utc::now();
        let task = Task::recover(
            id,
            TaskTitle::try_from("Restored task").unwrap(),
            TaskDescription::try_from("").unwrap(),
            TaskStatus::InProgress,
            now,
            now,
        );
        assert_eq!(task.pending_events().len(), 0);
        assert_eq!(task.status(), &TaskStatus::InProgress);
    }
}
