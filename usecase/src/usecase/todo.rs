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
        let resp = self
            .repositories
            .todo_repository()
            .get(&id.try_into()?, self.db.0.as_ref())
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
        let status = match &condition.status_code {
            Some(code) => Some(
                self.repositories
                    .todo_status_repository()
                    .get_by_code(code.as_str(), self.db.0.as_ref())
                    .await?,
            ),
            None => None,
        };

        let resp = self
            .repositories
            .todo_repository()
            .find(status, self.db.0.as_ref())
            .await?;
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

    /// create와 update를 단일 트랜잭션에서 실행한다.
    /// update가 실패하면 create도 함께 롤백된다.
    pub async fn create_and_update_todo(
        &self,
        create_source: CreateTodo,
        update_source: UpdateTodoView,
    ) -> anyhow::Result<(TodoView, TodoView)> {
        let mut tx = self.db.0.clone().begin().await?;

        let created = self
            .repositories
            .todo_repository()
            .insert(create_source.try_into()?, &mut tx)
            .await?;

        let status = match &update_source.status_code {
            Some(code) => Some(
                self.repositories
                    .todo_status_repository()
                    .get_by_code(code.as_str(), &mut tx)
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

        let updated = self
            .repositories
            .todo_repository()
            .update(update_todo, &mut tx)
            .await?;

        tx.commit().await?;
        Ok((created.into(), updated.into()))
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
