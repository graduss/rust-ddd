use domain::{DomainError, Task, TaskId, TaskRepository};
use sqlx::postgres::PgPool;

use crate::maper::{TaskSqlx, TaskStatusSqlx};

pub struct PgTaskRepository {
    pool: PgPool,
}

impl PgTaskRepository {
    pub async fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TaskRepository for PgTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(*task.id().clone())
        .bind(task.title().to_string())
        .bind(task.description().to_string())
        .bind(TaskStatusSqlx::from(task.status().clone()))
        .bind(task.created_at().clone())
        .bind(task.updated_at().clone())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: &TaskId) -> Option<Task> {
        let task: Option<TaskSqlx> = sqlx::query_as(
            r#"
            SELECT * FROM tasks WHERE id = $1
        "#,
        )
        .bind(*id.clone())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))
        .ok()
        .flatten();

        task.map(|t| t.try_into().ok()).flatten()
    }

    async fn find_all(&self) -> Result<Vec<Task>, DomainError> {
        let list: Vec<TaskSqlx> = sqlx::query_as(
            r#"
            SELECT * FROM tasks
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        list.into_iter()
            .map(|t| t.try_into())
            .collect::<Result<Vec<Task>, DomainError>>()
    }

    async fn delete_by_id(&self, id: &TaskId) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            DELETE FROM tasks WHERE id = $1
        "#,
        )
        .bind(*id.clone())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn update(&self, task: &Task) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE tasks SET title = $2, description = $3, status = $4, updated_at = $5 WHERE id = $1
        "#,
        )
        .bind(*task.id().clone())
        .bind(task.title().to_string())
        .bind(task.description().to_string())
        .bind(TaskStatusSqlx::from(task.status().clone()))
        .bind(task.updated_at().clone())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, id: &TaskId) -> Result<bool, DomainError> {
        sqlx::query_as::<_, (bool,)>(
            r#"
            SELECT EXISTS(SELECT 1 FROM tasks WHERE id = $1)
        "#,
        )
        .bind(*id.clone())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))
        .map(|exists| exists.0)
    }
}
