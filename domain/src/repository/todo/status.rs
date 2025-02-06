use crate::model::todo::status::TodoStatus;
use async_trait::async_trait;
use crate::transaction::PgAcquire;

#[async_trait]
pub trait TodoStatusRepository {
    async fn get_by_code(&self, code: &str, executor: impl PgAcquire<'_>) -> anyhow::Result<TodoStatus>;
}
