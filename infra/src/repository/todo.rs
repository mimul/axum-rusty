pub mod status;

use crate::db::IDatabasePool;
use crate::model::todo::{InsertTodo, StoredTodo, UpdateStoredTodo, UpsertStoredTodo};
use crate::repository::PgTx;
use anyhow::Context;
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use domain::model::Id;
use shaku::Component;
use sqlx::{query, query_as};
use std::sync::Arc;

/// Todo 레포지토리 인터페이스.
#[async_trait]
pub trait ITodoRepository: shaku::Interface {
    async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
    async fn find(&self, status: Option<TodoStatus>) -> anyhow::Result<Vec<Todo>>;
    async fn get_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
    async fn find_tx(&self, tx: &mut PgTx, status: Option<TodoStatus>)
        -> anyhow::Result<Vec<Todo>>;
    async fn insert_tx(&self, tx: &mut PgTx, todo: NewTodo) -> anyhow::Result<Todo>;
    async fn update_tx(&self, tx: &mut PgTx, todo: UpdateTodo) -> anyhow::Result<Todo>;
    async fn upsert_tx(&self, tx: &mut PgTx, todo: UpsertTodo) -> anyhow::Result<Todo>;
    async fn delete_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
}

/// PostgreSQL Todo 레포지토리 구현체.
#[derive(Component)]
#[shaku(interface = ITodoRepository)]
pub struct TodoRepository {
    #[shaku(inject)]
    db: Arc<dyn IDatabasePool>,
}

#[async_trait]
impl ITodoRepository for TodoRepository {
    async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>> {
        let sql = r#"
            SELECT t.id, t.title, t.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_statuses ts ON ts.id = t.status_id
            WHERE t.id = $1
        "#;
        let result = query_as::<_, StoredTodo>(sql)
            .bind(id.value.to_string())
            .fetch_optional(self.db.pool())
            .await?;
        match result {
            Some(st) => Ok(Some(st.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find(&self, status: Option<TodoStatus>) -> anyhow::Result<Vec<Todo>> {
        let stored: Vec<StoredTodo> = match status {
            Some(s) => {
                let sql = r#"
                    SELECT t.id, t.title, t.description,
                           ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                           t.created_at, t.updated_at
                    FROM todos t
                    INNER JOIN todo_statuses ts ON ts.id = t.status_id
                    WHERE t.status_id = $1
                    ORDER BY t.created_at ASC
                "#;
                query_as::<_, StoredTodo>(sql)
                    .bind(s.id.value.to_string())
                    .fetch_all(self.db.pool())
                    .await?
            }
            None => {
                let sql = r#"
                    SELECT t.id, t.title, t.description,
                           ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                           t.created_at, t.updated_at
                    FROM todos t
                    INNER JOIN todo_statuses ts ON ts.id = t.status_id
                    ORDER BY t.created_at ASC
                "#;
                query_as::<_, StoredTodo>(sql)
                    .fetch_all(self.db.pool())
                    .await?
            }
        };
        stored
            .into_iter()
            .map(|st| st.try_into())
            .collect::<anyhow::Result<Vec<Todo>>>()
    }

    async fn get_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>> {
        let sql = r#"
            SELECT t.id, t.title, t.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_statuses ts ON ts.id = t.status_id
            WHERE t.id = $1
        "#;
        let result = query_as::<_, StoredTodo>(sql)
            .bind(id.value.to_string())
            .fetch_optional(&mut **tx)
            .await?;
        match result {
            Some(st) => Ok(Some(st.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_tx(
        &self,
        tx: &mut PgTx,
        status: Option<TodoStatus>,
    ) -> anyhow::Result<Vec<Todo>> {
        let stored: Vec<StoredTodo> = match status {
            Some(s) => {
                let sql = r#"
                    SELECT t.id, t.title, t.description,
                           ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                           t.created_at, t.updated_at
                    FROM todos t
                    INNER JOIN todo_statuses ts ON ts.id = t.status_id
                    WHERE t.status_id = $1
                    ORDER BY t.created_at ASC
                "#;
                query_as::<_, StoredTodo>(sql)
                    .bind(s.id.value.to_string())
                    .fetch_all(&mut **tx)
                    .await?
            }
            None => {
                let sql = r#"
                    SELECT t.id, t.title, t.description,
                           ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                           t.created_at, t.updated_at
                    FROM todos t
                    INNER JOIN todo_statuses ts ON ts.id = t.status_id
                    ORDER BY t.created_at ASC
                "#;
                query_as::<_, StoredTodo>(sql).fetch_all(&mut **tx).await?
            }
        };
        stored
            .into_iter()
            .map(|st| st.try_into())
            .collect::<anyhow::Result<Vec<Todo>>>()
    }

    async fn insert_tx(&self, tx: &mut PgTx, source: NewTodo) -> anyhow::Result<Todo> {
        let todo: InsertTodo = source.into();
        let id = todo.id.clone();

        query("INSERT INTO todos (id, title, description) VALUES ($1, $2, $3)")
            .bind(&todo.id)
            .bind(&todo.title)
            .bind(&todo.description)
            .execute(&mut **tx)
            .await?;

        let sql = r#"
            SELECT t.id, t.title, t.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_statuses ts ON ts.id = t.status_id
            WHERE t.id = $1
        "#;
        let stored = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut **tx)
            .await?;
        stored.try_into()
    }

    async fn update_tx(&self, tx: &mut PgTx, source: UpdateTodo) -> anyhow::Result<Todo> {
        let todo: UpdateStoredTodo = source.into();
        let id = todo.id.clone();

        let update_sql = r#"
            UPDATE todos AS target SET
                title       = CASE WHEN $2 IS NOT NULL THEN $2 ELSE current_todo.title END,
                description = CASE WHEN $3 IS NOT NULL THEN $3 ELSE current_todo.description END,
                status_id   = CASE WHEN $4 IS NOT NULL THEN $4 ELSE current_todo.status_id END,
                updated_at  = current_timestamp
            FROM (SELECT * FROM todos WHERE id = $1) AS current_todo
            WHERE target.id = $1
        "#;
        query(update_sql)
            .bind(&todo.id)
            .bind(todo.title)
            .bind(todo.description)
            .bind(todo.status_id)
            .execute(&mut **tx)
            .await?;

        let sql = r#"
            SELECT t.id, t.title, t.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_statuses ts ON ts.id = t.status_id
            WHERE t.id = $1
        "#;
        let stored = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut **tx)
            .await?;
        stored.try_into()
    }

    async fn upsert_tx(&self, tx: &mut PgTx, source: UpsertTodo) -> anyhow::Result<Todo> {
        let todo: UpsertStoredTodo = source.into();
        let id = todo.id.clone();

        let upsert_sql = r#"
            INSERT INTO todos (id, title, description, status_id) VALUES ($1, $2, $3, $4)
            ON CONFLICT ON CONSTRAINT pk_todos_id
            DO UPDATE SET title = $2, description = $3, status_id = $4, updated_at = current_timestamp
        "#;
        query(upsert_sql)
            .bind(&todo.id)
            .bind(todo.title)
            .bind(todo.description)
            .bind(todo.status_id)
            .execute(&mut **tx)
            .await
            .context(format!(r#"failed to upsert "{}" into todos"#, todo.id))?;

        let sql = r#"
            SELECT t.id, t.title, t.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_statuses ts ON ts.id = t.status_id
            WHERE t.id = $1
        "#;
        let stored = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut **tx)
            .await?;
        stored.try_into()
    }

    async fn delete_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>> {
        let sql = r#"
            WITH deleted AS (
                DELETE FROM todos WHERE id = $1
                RETURNING id, title, description, status_id, created_at, updated_at
            )
            SELECT d.id, d.title, d.description,
                   ts.id AS status_id, ts.code AS status_code, ts.name AS status_name,
                   d.created_at, d.updated_at
            FROM deleted d
            INNER JOIN todo_statuses ts ON ts.id = d.status_id
        "#;
        let result = query_as::<_, StoredTodo>(sql)
            .bind(id.value.to_string())
            .fetch_optional(&mut **tx)
            .await?;
        match result {
            Some(st) => Ok(Some(st.try_into()?)),
            None => Ok(None),
        }
    }
}
