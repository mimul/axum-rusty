use crate::model::user::{CreateUser, SearchUserCondition, UserView};
use anyhow::anyhow;
use std::sync::Arc;
use todo_domain::repository::user::UserRepository;
use todo_infra::modules::RepositoriesModuleExt;

pub struct UserUseCase<R: RepositoriesModuleExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesModuleExt> crate::usecase::user::UserUseCase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }

    pub async fn get_user(&self, id: String) -> anyhow::Result<Option<UserView>> {
        let resp = self
            .repositories
            .user_repository()
            .get_user(&id.try_into()?)
            .await?;

        match resp {
            Some(user) => Ok(Some(user.into())),
            None => Ok(None),
        }
    }

    pub async fn get_user_by_username(
        &self,
        condition: SearchUserCondition,
    ) -> anyhow::Result<Option<UserView>> {
        let username = if let Some(u) = &condition.username {
            u.as_str()
        } else {
            return Err(anyhow!("username is empty".to_string()));
        };
        let resp = self
            .repositories
            .user_repository()
            .get_user_by_username(username)
            .await?;

        match resp {
            Some(user) => Ok(Some(user.into())),
            None => Ok(None),
        }
    }

    pub async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        // hash password
        let hashed_password = bcrypt::hash(source.password.clone(), 12).unwrap();
        if hashed_password.is_empty() {
            return Err(anyhow!("hashed password is empty"));
        }
        let user = CreateUser::new(source.username, hashed_password);
        let user_view = self
            .repositories
            .user_repository()
            .insert(user.try_into()?)
            .await?;

        Ok(user_view.into())
    }
}
