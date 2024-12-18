use crate::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use anyhow::anyhow;
use domain::model::todo::{UpdateTodo, UpsertTodo};
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use infra::modules::RepositoriesModuleExt;
use std::sync::Arc;

pub struct TodoUseCase<R: RepositoriesModuleExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesModuleExt> TodoUseCase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }

    pub async fn get_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let resp = self
            .repositories
            .todo_repository()
            .get(&id.try_into()?)
            .await?;

        match resp {
            Some(t) => Ok(Some(t.into())),
            None => Ok(None),
        }
    }

    pub async fn find_todo(
        &self,
        condition: SearchTodoCondition,
    ) -> anyhow::Result<Option<Vec<TodoView>>> {
        let status = if let Some(code) = &condition.status_code {
            match self
                .repositories
                .todo_status_repository()
                .get_by_code(code.as_str())
                .await
            {
                Ok(status) => Some(status),
                Err(err) => {
                    return Err(anyhow!(err));
                }
            }
        } else {
            None
        };

        let resp = self.repositories.todo_repository().find(status).await?;
        match resp {
            Some(todos) => {
                let tv_list = todos.into_iter().map(|t| t.into()).collect();
                Ok(Some(tv_list))
            }
            None => Ok(None),
        }
    }

    pub async fn create_todo(&self, source: CreateTodo) -> anyhow::Result<TodoView> {
        let todo_view = self
            .repositories
            .todo_repository()
            .insert(source.try_into()?)
            .await?;

        Ok(todo_view.into())
    }

    pub async fn update_todo(&self, source: UpdateTodoView) -> anyhow::Result<TodoView> {
        let status = if let Some(code) = &source.status_code {
            match self
                .repositories
                .todo_status_repository()
                .get_by_code(code.as_str())
                .await
            {
                Ok(status) => Some(status),
                Err(err) => {
                    return Err(anyhow!(err));
                }
            }
        } else {
            None
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
            .update(update_todo)
            .await?;

        Ok(todo_view.into())
    }

    pub async fn upsert_todo(&self, source: UpsertTodoView) -> anyhow::Result<TodoView> {
        let status = match self
            .repositories
            .todo_status_repository()
            .get_by_code(&source.status_code)
            .await
        {
            Ok(status) => status,
            Err(err) => {
                return Err(anyhow!(err));
            }
        };

        let upsert_todo = UpsertTodo::new(
            source.id.try_into()?,
            source.title,
            source.description,
            status,
        );

        let todo_view = self
            .repositories
            .todo_repository()
            .upsert(upsert_todo)
            .await?;

        Ok(todo_view.into())
    }

    pub async fn delete_todo(&self, id: String) -> anyhow::Result<Option<TodoView>> {
        let resp = self
            .repositories
            .todo_repository()
            .delete(&id.try_into()?)
            .await?;

        match resp {
            Some(t) => Ok(Some(t.into())),
            None => Ok(None),
        }
    }
}
