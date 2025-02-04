use crate::model::todo::status::StoredTodoStatus;
use crate::repository::DatabaseRepositoryImpl;
use anyhow::anyhow;
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use domain::repository::todo::status::TodoStatusRepository;
use sqlx::query_as;
use domain::transaction::PostgresAcquire;

#[async_trait]
impl TodoStatusRepository for DatabaseRepositoryImpl<TodoStatus> {
    async fn get_by_code(&self, code: &str, executor: impl PostgresAcquire<'_>) -> anyhow::Result<TodoStatus> {
        let mut conn = executor.acquire().await?;
        let sql = r#"
            select id, code, name
            from todo_statuses
            where code = $1
        "#;

        let stored_todo_status = query_as::<_, StoredTodoStatus>(sql)
            .bind(code.to_string())
            .fetch_one(&mut *conn)
            .await
            .ok();

        match stored_todo_status {
            Some(todo_status) => Ok(todo_status.try_into()?),
            None => Err(anyhow!("`statusCode` is invalid.")),
        }
    }
}
