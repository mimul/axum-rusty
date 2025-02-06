use crate::model::todo::status::TodoStatus;
use crate::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use crate::model::Id;
use async_trait::async_trait;
use crate::transaction::PgAcquire;

pub mod status;

#[async_trait]
pub trait TodoRepository {
    async fn get(&self, id: &Id<Todo>, executor: impl PgAcquire<'_>) -> anyhow::Result<Option<Todo>>;
    async fn find(&self, status: Option<TodoStatus>, executor: impl PgAcquire<'_>) -> anyhow::Result<Option<Vec<Todo>>>;
    async fn insert(&self, source: NewTodo, executor: impl PgAcquire<'_>) -> anyhow::Result<Todo>;
    async fn update(&self, source: UpdateTodo, executor: impl PgAcquire<'_>) -> anyhow::Result<Todo>;
    async fn upsert(&self, source: UpsertTodo, executor: impl PgAcquire<'_>) -> anyhow::Result<Todo>;
    async fn delete(&self, id: &Id<Todo>, executor: impl PgAcquire<'_>) -> anyhow::Result<Option<Todo>>;
}
