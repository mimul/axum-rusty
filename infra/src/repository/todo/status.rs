use crate::model::todo::status::StoredTodoStatus;
use crate::module::uow::SharedTx;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use domain::repository::todo::status::TodoStatusRepository;
use sqlx::query_as;

pub struct PgTodoStatusRepo {
    tx: SharedTx,
}

impl PgTodoStatusRepo {
    pub fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl TodoStatusRepository for PgTodoStatusRepo {
    async fn get_by_code(&self, code: &str) -> anyhow::Result<TodoStatus> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().context("transaction not active")?;
        let sql = r#"
            SELECT id, code, name
            FROM todo_statuses
            WHERE code = $1
        "#;
        let result = query_as::<_, StoredTodoStatus>(sql)
            .bind(code)
            .fetch_optional(&mut **tx)
            .await?;
        match result {
            Some(st) => Ok(st.try_into()?),
            None => Err(anyhow!("`statusCode` '{}' is invalid.", code)),
        }
    }
}
