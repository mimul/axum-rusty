use crate::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use crate::module::uow::TodoUnitOfWorkFactory;
use domain::model::todo::{UpdateTodo, UpsertTodo};
use std::sync::Arc;

pub struct TodoUseCase {
    uow_factory: Arc<dyn TodoUnitOfWorkFactory>,
}

impl TodoUseCase {
    pub fn new(uow_factory: Arc<dyn TodoUnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }

    pub async fn get_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let uow = self.uow_factory.begin().await?;
        let resp = uow.todo_repo().get(&id.try_into()?).await?;
        Ok(resp.map(Into::into))
    }

    pub async fn find_todo(&self, condition: SearchTodoCondition) -> anyhow::Result<Vec<TodoView>> {
        let uow = self.uow_factory.begin().await?;
        let status = match &condition.status_code {
            Some(code) => Some(uow.todo_status_repo().get_by_code(code.as_str()).await?),
            None => None,
        };
        let todos = uow.todo_repo().find(status).await?;
        Ok(todos.into_iter().map(Into::into).collect())
    }

    pub async fn create_todo(&self, source: CreateTodo) -> anyhow::Result<TodoView> {
        let mut uow = self.uow_factory.begin().await?;
        let todo = uow.todo_repo().insert(source.try_into()?).await?;
        uow.commit().await?;
        Ok(todo.into())
    }

    pub async fn update_todo(&self, source: UpdateTodoView) -> anyhow::Result<TodoView> {
        let mut uow = self.uow_factory.begin().await?;
        let status = match &source.status_code {
            Some(code) => Some(uow.todo_status_repo().get_by_code(code.as_str()).await?),
            None => None,
        };
        let update_todo = UpdateTodo::new(source.id.try_into()?, source.title, source.description, status);
        let todo = uow.todo_repo().update(update_todo).await?;
        uow.commit().await?;
        Ok(todo.into())
    }

    pub async fn upsert_todo(&self, source: UpsertTodoView) -> anyhow::Result<TodoView> {
        let mut uow = self.uow_factory.begin().await?;
        let status = uow.todo_status_repo().get_by_code(&source.status_code).await?;
        let upsert_todo = UpsertTodo::new(source.id.try_into()?, source.title, source.description, status);
        let todo = uow.todo_repo().upsert(upsert_todo).await?;
        uow.commit().await?;
        Ok(todo.into())
    }

    /// create와 update를 단일 트랜잭션에서 실행한다.
    /// update가 실패하면 create도 함께 롤백된다.
    pub async fn create_and_update_todo(
        &self,
        create_source: CreateTodo,
        update_source: UpdateTodoView,
    ) -> anyhow::Result<(TodoView, TodoView)> {
        let mut uow = self.uow_factory.begin().await?;

        let created = uow.todo_repo().insert(create_source.try_into()?).await?;

        let status = match &update_source.status_code {
            Some(code) => Some(uow.todo_status_repo().get_by_code(code.as_str()).await?),
            None => None,
        };
        let update_todo = UpdateTodo::new(
            update_source.id.try_into()?,
            update_source.title,
            update_source.description,
            status,
        );
        let updated = uow.todo_repo().update(update_todo).await?;
        uow.commit().await?;
        Ok((created.into(), updated.into()))
    }

    pub async fn delete_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let mut uow = self.uow_factory.begin().await?;
        let resp = uow.todo_repo().delete(&id.try_into()?).await?;
        uow.commit().await?;
        Ok(resp.map(Into::into))
    }
}
