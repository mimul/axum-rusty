use crate::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use anyhow::anyhow;
use domain::model::user::User;
use domain::repository::user::UserRepository;
use infra::modules::RepositoriesModuleExt;
use std::sync::Arc;
use log::{error, info};
use infra::persistence::postgres::Db;

pub struct UserUseCase<R: RepositoriesModuleExt> {
    db: Db,
    repositories: Arc<R>,
}

impl<R: RepositoriesModuleExt> UserUseCase<R> {
    pub fn new(db: Db, repositories: Arc<R>) -> Self {
        Self { db, repositories }
    }

    pub async fn get_user(&self, id: String) -> anyhow::Result<Option<UserView>> {
        let mut tx = self.db.0.clone().begin().await?;
        let resp = self
            .repositories
            .user_repository()
            .get_user(&id.clone().try_into()?, &mut tx)
            .await?;

        match resp {
            Some(user) => {
                Ok(Some(user.into()))
            },
            None => Ok(None),
        }
    }

    pub async fn get_user_by_username(
        &self,
        condition: SearchUserCondition,
    ) -> anyhow::Result<Option<UserView>> {
        let mut tx = self.db.0.clone().begin().await?;
        let username = if let Some(u) = &condition.username {
            u.as_str()
        } else {
            return Err(anyhow!("username is empty".to_string()));
        };
        let resp = self
            .repositories
            .user_repository()
            .get_user_by_username(username, &mut tx)
            .await?;

        match resp {
            Some(user) => Ok(Some(user.into())),
            None => Ok(None),
        }
    }

    pub async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        let username = source.username.clone();
        let mut tx = self.db.0.clone().begin().await?;
        match self
            .repositories
            .user_repository()
            .get_user_by_username(username.as_str(), &mut tx)
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
        let user = CreateUser::new(source.username, hashed_password, source.fullname);
        let user_view = self
            .repositories
            .user_repository()
            .insert(user.try_into()?, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(user_view.into())
    }

    pub async fn login_user(&self, source: LoginUser) -> anyhow::Result<UserView> {
        let username = source.username.clone();
        let mut tx = self.db.0.clone().begin().await?;
        let user_view: User = match self
            .repositories
            .user_repository()
            .get_user_by_username(username.as_str(), &mut tx)
            .await
        {
            Ok(Some(user_view)) => user_view,
            _ => {
                error!("username {} is not registered.", username);
                return Err(anyhow!("username {} is not registered", username));
            }
        };
        let login_result = bcrypt::verify(source.password.clone(), user_view.password.as_str())?;
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
