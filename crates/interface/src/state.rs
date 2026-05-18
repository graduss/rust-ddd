use std::sync::Arc;

use application::{
    commands::{
        change_status::ChangeStatusCommand, create_task::CreateTaskCommand,
        delete_task::DeleteTaskCommand,
    },
    queries::{dto::TaskDto, get_task::GetTaskByIdQuery, list_tasks::ListTaskQuery},
    use_case::UseCase,
};
use domain::TaskId;

pub type DynCreateTask = Arc<dyn UseCase<Input = CreateTaskCommand, Output = TaskId>>;
pub type DynChangeStatus = Arc<dyn UseCase<Input = ChangeStatusCommand, Output = ()>>;
pub type DynDeleteTask = Arc<dyn UseCase<Input = DeleteTaskCommand, Output = ()>>;
pub type DynGetTask = Arc<dyn UseCase<Input = GetTaskByIdQuery, Output = TaskDto>>;
pub type DynGetAllTasks = Arc<dyn UseCase<Input = ListTaskQuery, Output = Vec<TaskDto>>>;

#[derive(Clone)]
pub struct AppState {
    pub create_task: DynCreateTask,
    pub chenge_status: DynChangeStatus,
    pub delete_task: DynDeleteTask,
    pub get_task: DynGetTask,
    pub get_all_tasks: DynGetAllTasks,
}
