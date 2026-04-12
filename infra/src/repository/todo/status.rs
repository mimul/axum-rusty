use crate::model::todo::status::StoredTodoStatus;
use crate::repository::todo::PgTx;
use anyhow::anyhow;
use domain::model::todo::status::TodoStatus;
use sqlx::{query_as, PgPool};

pub struct TodoStatusRepository {
    pool: PgPool,
}

impl TodoStatusRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_code(&self, code: &str) -> anyhow::Result<TodoStatus> {
        Self::get_by_code_impl(code, &self.pool).await
    }

    pub async fn get_by_code_tx(&self, tx: &mut PgTx, code: &str) -> anyhow::Result<TodoStatus> {
        Self::get_by_code_impl(code, &mut **tx).await
    }

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
