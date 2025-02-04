use crate::model::todo::status::TodoStatus;
use async_trait::async_trait;
use crate::transaction::PostgresAcquire;

#[async_trait]
pub trait TodoStatusRepository {
    async fn get_by_code(&self, code: &str, executor: impl PostgresAcquire<'_>) -> anyhow::Result<TodoStatus>;
}
