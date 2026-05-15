use crate::repository::PgTx;
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use domain::model::Id;

/// Todo 레포지토리 인터페이스.
#[async_trait]
pub trait ITodoRepository: shaku::Interface {
    async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
    async fn find(&self, status: Option<TodoStatus>) -> anyhow::Result<Vec<Todo>>;
    async fn get_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
    async fn find_tx(
        &self,
        tx: &mut PgTx,
        status: Option<TodoStatus>,
    ) -> anyhow::Result<Vec<Todo>>;
    async fn insert_tx(&self, tx: &mut PgTx, todo: NewTodo) -> anyhow::Result<Todo>;
    async fn update_tx(&self, tx: &mut PgTx, todo: UpdateTodo) -> anyhow::Result<Todo>;
    async fn upsert_tx(&self, tx: &mut PgTx, todo: UpsertTodo) -> anyhow::Result<Todo>;
    async fn delete_tx(&self, tx: &mut PgTx, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
}
