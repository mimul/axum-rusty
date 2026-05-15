use crate::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use async_trait::async_trait;

/// Todo 유스케이스 인터페이스.
#[async_trait]
pub trait ITodoUseCase: shaku::Interface {
    async fn get_todo(&self, id: String) -> anyhow::Result<Option<TodoView>>;
    async fn find_todo(&self, condition: SearchTodoCondition) -> anyhow::Result<Vec<TodoView>>;
    async fn create_todo(&self, source: CreateTodo) -> anyhow::Result<TodoView>;
    async fn update_todo(&self, source: UpdateTodoView) -> anyhow::Result<TodoView>;
    async fn upsert_todo(&self, source: UpsertTodoView) -> anyhow::Result<TodoView>;
    async fn create_and_update_todo(
        &self,
        create_source: CreateTodo,
        update_source: UpdateTodoView,
    ) -> anyhow::Result<(TodoView, TodoView)>;
    async fn delete_todo(&self, id: String) -> anyhow::Result<Option<TodoView>>;
}
