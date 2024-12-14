use crate::model::user::{CreateUser, SearchUserCondition, UserView};
use anyhow::anyhow;
use std::sync::Arc;
use tracing::log::{error, info};
use domain::model::user::User;
use domain::repository::user::UserRepository;
use infra::modules::RepositoriesModuleExt;

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
        let username = source.username.clone();
        match self.repositories
            .user_repository()
            .get_user_by_username(username.as_str())
            .await
        {
            Ok(Some(_)) => {
                error!("username {} already exists", username);
                return Err(anyhow!("username {} already exists", username));
            }
            Err(e) => {
                error!("failed to get user by username: {:?}", e);
                return Err(anyhow!("username is empty".to_string()));
            }
            _ => {}
        }
        // hash password
        let hashed_password = bcrypt::hash(source.password.clone(), 12)?;
        if hashed_password.is_empty() {
            return Err(anyhow!("hashed password is empty"));
        }
        // let hashed_password = bcrypt::hash(source.password.clone(), 12).map_err(|_| {
        //     error!("Failed to hash password");
        //     None
        // });
        let user = CreateUser::new(source.username, hashed_password);
        let user_view = self
            .repositories
            .user_repository()
            .insert(user.try_into()?)
            .await?;

        Ok(user_view.into())
    }

    pub async fn login_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        let username = source.username.clone();
        let user_view: User = match self.repositories.user_repository().get_user_by_username(username.as_str()).await {
            Ok(Some(user_view)) => user_view,
            _ => {
                error!("username {} is not registered.", username);
                return Err(anyhow!("username {} is not registered", username));
            }
        };
        let login_result = bcrypt::verify(source.password.clone(), user_view.password.as_str())?;
        // let login_result =
        //     bcrypt::verify(source.password.clone(), user_view.password.as_str()).map_err(|_| {
        //         error!("failed to hash password");
        //         return Err(anyhow!("failed to hash password."));
        //     });
        match login_result {
            true => {
                info!("login succeeded!");
                Ok(user_view.into())
            }
            false => {
                error!("bad password.");
                Err(anyhow!("bad password."))
            }
        }
    }
}
