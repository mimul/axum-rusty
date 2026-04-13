use crate::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use async_trait::async_trait;
use domain::model::todo::{UpdateTodo, UpsertTodo};
use infra::db::IDatabasePool;
use infra::repository::todo::status::ITodoStatusRepository;
use infra::repository::todo::ITodoRepository;
use shaku::Component;
use std::sync::Arc;

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

/// Todo 유스케이스 구현체.
#[derive(Component)]
#[shaku(interface = ITodoUseCase)]
pub struct TodoUseCase {
    #[shaku(inject)]
    db: Arc<dyn IDatabasePool>,
    #[shaku(inject)]
    todo_repo: Arc<dyn ITodoRepository>,
    #[shaku(inject)]
    todo_status_repo: Arc<dyn ITodoStatusRepository>,
}

#[async_trait]
impl ITodoUseCase for TodoUseCase {
    async fn get_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let resp = self.todo_repo.get(&id.try_into()?).await?;
        Ok(resp.map(Into::into))
    }

    async fn find_todo(&self, condition: SearchTodoCondition) -> anyhow::Result<Vec<TodoView>> {
        let status = match &condition.status_code {
            Some(code) => Some(self.todo_status_repo.get_by_code(code.as_str()).await?),
            None => None,
        };
        let todos = self.todo_repo.find(status).await?;
        Ok(todos.into_iter().map(Into::into).collect())
    }

    async fn create_todo(&self, source: CreateTodo) -> anyhow::Result<TodoView> {
        let mut tx = self.db.pool().begin().await?;
        let todo = self
            .todo_repo
            .insert_tx(&mut tx, source.try_into()?)
            .await?;
        tx.commit().await?;
        Ok(todo.into())
    }

    async fn update_todo(&self, source: UpdateTodoView) -> anyhow::Result<TodoView> {
        let mut tx = self.db.pool().begin().await?;
        let status = match &source.status_code {
            Some(code) => Some(
                self.todo_status_repo
                    .get_by_code_tx(&mut tx, code.as_str())
                    .await?,
            ),
            None => None,
        };
        let update_todo = UpdateTodo::new(
            source.id.try_into()?,
            source.title,
            source.description,
            status,
        );
        let todo = self.todo_repo.update_tx(&mut tx, update_todo).await?;
        tx.commit().await?;
        Ok(todo.into())
    }

    async fn upsert_todo(&self, source: UpsertTodoView) -> anyhow::Result<TodoView> {
        let mut tx = self.db.pool().begin().await?;
        let status = self
            .todo_status_repo
            .get_by_code_tx(&mut tx, &source.status_code)
            .await?;
        let upsert_todo = UpsertTodo::new(
            source.id.try_into()?,
            source.title,
            source.description,
            status,
        );
        let todo = self.todo_repo.upsert_tx(&mut tx, upsert_todo).await?;
        tx.commit().await?;
        Ok(todo.into())
    }

    async fn create_and_update_todo(
        &self,
        create_source: CreateTodo,
        update_source: UpdateTodoView,
    ) -> anyhow::Result<(TodoView, TodoView)> {
        let mut tx = self.db.pool().begin().await?;

        let created = self
            .todo_repo
            .insert_tx(&mut tx, create_source.try_into()?)
            .await?;

        let status = match &update_source.status_code {
            Some(code) => Some(
                self.todo_status_repo
                    .get_by_code_tx(&mut tx, code.as_str())
                    .await?,
            ),
            None => None,
        };
        let update_todo = UpdateTodo::new(
            update_source.id.try_into()?,
            update_source.title,
            update_source.description,
            status,
        );
        let updated = self.todo_repo.update_tx(&mut tx, update_todo).await?;
        tx.commit().await?;
        Ok((created.into(), updated.into()))
    }

    async fn delete_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let mut tx = self.db.pool().begin().await?;
        let resp = self.todo_repo.delete_tx(&mut tx, &id.try_into()?).await?;
        tx.commit().await?;
        Ok(resp.map(Into::into))
    }
}
