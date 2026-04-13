use crate::db::IDatabasePool;
use crate::model::todo::status::StoredTodoStatus;
use crate::repository::PgTx;
use anyhow::anyhow;
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use shaku::Component;
use sqlx::query_as;
use std::sync::Arc;

/// TodoStatus 레포지토리 인터페이스.
#[async_trait]
pub trait ITodoStatusRepository: shaku::Interface {
    async fn get_by_code(&self, code: &str) -> anyhow::Result<TodoStatus>;
    async fn get_by_code_tx(&self, tx: &mut PgTx, code: &str) -> anyhow::Result<TodoStatus>;
}

/// PostgreSQL TodoStatus 레포지토리 구현체.
#[derive(Component)]
#[shaku(interface = ITodoStatusRepository)]
pub struct TodoStatusRepository {
    #[shaku(inject)]
    db: Arc<dyn IDatabasePool>,
}

#[async_trait]
impl ITodoStatusRepository for TodoStatusRepository {
    async fn get_by_code(&self, code: &str) -> anyhow::Result<TodoStatus> {
        Self::get_by_code_impl(code, self.db.pool()).await
    }

    async fn get_by_code_tx(&self, tx: &mut PgTx, code: &str) -> anyhow::Result<TodoStatus> {
        Self::get_by_code_impl(code, &mut **tx).await
    }
}

impl TodoStatusRepository {
    async fn get_by_code_impl<'e>(
        code: &str,
        executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
    ) -> anyhow::Result<TodoStatus> {
        let sql = r#"
            SELECT id, code, name
            FROM todo_statuses
            WHERE code = $1
        "#;
        let result = query_as::<_, StoredTodoStatus>(sql)
            .bind(code)
            .fetch_optional(executor)
            .await?;
        match result {
            Some(st) => Ok(st.try_into()?),
            None => Err(anyhow!("`statusCode` '{}' is invalid.", code)),
        }
    }
}
