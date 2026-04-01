use crate::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use domain::model::todo::{UpdateTodo, UpsertTodo};
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use std::sync::Arc;
use infra::module::repo_module::RepositoriesModuleExt;
use infra::persistence::postgres::Db;

pub struct TodoUseCase<R: RepositoriesModuleExt> {
    db: Db,
    repositories: Arc<R>,
}

impl<R: RepositoriesModuleExt> TodoUseCase<R> {
    pub fn new(db: Db, repositories: Arc<R>) -> Self {
        Self { db, repositories }
    }

    pub async fn get_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let mut tx = self.db.0.clone().begin().await?;
        let resp = self
            .repositories
            .todo_repository()
            .get(&id.try_into()?, &mut tx)
            .await?;

        match resp {
            Some(todo) => Ok(Some(todo.into())),
            None => Ok(None),
        }
    }

    pub async fn find_todo(
        &self,
        condition: SearchTodoCondition,
    ) -> anyhow::Result<Option<Vec<TodoView>>> {
        let mut tx = self.db.0.clone().begin().await?;
        let status = match &condition.status_code {
            Some(code) => Some(
                self.repositories
                    .todo_status_repository()
                    .get_by_code(code.as_str(), &mut tx)
                    .await?,
            ),
            None => None,
        };

        let resp = self.repositories.todo_repository().find(status, &mut tx).await?;
        match resp {
            Some(todos) => {
                let tv_list = todos.into_iter().map(|t| t.into()).collect();
                Ok(Some(tv_list))
            }
            None => Ok(None),
        }
    }

    pub async fn create_todo(&self, source: CreateTodo) -> anyhow::Result<TodoView> {
        let mut tx = self.db.0.clone().begin().await?;
        let todo_view = self
            .repositories
            .todo_repository()
            .insert(source.try_into()?, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(todo_view.into())
    }

    pub async fn update_todo(&self, source: UpdateTodoView) -> anyhow::Result<TodoView> {
        let mut tx = self.db.0.clone().begin().await?;
        let status = match &source.status_code {
            Some(code) => Some(
                self.repositories
                    .todo_status_repository()
                    .get_by_code(code.as_str(), &mut tx)
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

        let todo_view = self
            .repositories
            .todo_repository()
            .update(update_todo, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(todo_view.into())
    }

    pub async fn upsert_todo(&self, source: UpsertTodoView) -> anyhow::Result<TodoView> {
        let mut tx = self.db.0.clone().begin().await?;
        let status = self
            .repositories
            .todo_status_repository()
            .get_by_code(&source.status_code, &mut tx)
            .await?;

        let upsert_todo = UpsertTodo::new(
            source.id.try_into()?,
            source.title,
            source.description,
            status,
        );

        let todo_view = self
            .repositories
            .todo_repository()
            .upsert(upsert_todo, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(todo_view.into())
    }

    pub async fn delete_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let mut tx = self.db.0.clone().begin().await?;
        let resp = self
            .repositories
            .todo_repository()
            .delete(&id.try_into()?, &mut tx)
            .await?;
        tx.commit().await?;
        match resp {
            Some(t) => Ok(Some(t.into())),
            None => Ok(None),
        }
    }
}
