use crate::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};

/// bcrypt 해시 cost factor (OWASP 권고: 10 이상)
const BCRYPT_COST: u32 = 12;
use anyhow::anyhow;
use domain::model::user::User;
use domain::repository::user::UserRepository;
use std::sync::Arc;
use log::{error, info};
use infra::module::repo_module::RepositoriesModuleExt;
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
        let resp = self
            .repositories
            .user_repository()
            .get_user(&id.clone().try_into()?, self.db.0.as_ref())
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
            return Err(anyhow!("username is empty"));
        };
        let resp = self
            .repositories
            .user_repository()
            .get_user_by_username(username, self.db.0.as_ref())
            .await?;

        match resp {
            Some(user) => Ok(Some(user.into())),
            None => Ok(None),
        }
    }

    pub async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        let username = source.username.clone();

        // 읽기는 트랜잭션 불필요 — pool 직접 사용
        match self
            .repositories
            .user_repository()
            .get_user_by_username(username.as_str(), self.db.0.as_ref())
            .await
        {
            Ok(Some(_)) => {
                error!("username {} already exists", username);
                return Err(anyhow!("username {} already exists", username));
            }
            Err(e) => {
                error!("failed to get user by username: {:?}", e);
                return Err(anyhow!("username is empty"));
            }
            _ => {}
        }

        // CPU-heavy 작업은 트랜잭션 시작 전에 완료
        let hashed_password = bcrypt::hash(source.password.clone(), BCRYPT_COST)?;
        if hashed_password.is_empty() {
            return Err(anyhow!("hashed password is empty"));
        }

        // 트랜잭션은 INSERT만 담당
        let mut tx = self.db.0.clone().begin().await?;
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

        // 읽기는 트랜잭션 불필요 — pool 직접 사용
        let user_view: User = match self
            .repositories
            .user_repository()
            .get_user_by_username(username.as_str(), self.db.0.as_ref())
            .await
        {
            Ok(Some(user_view)) => user_view,
            _ => {
                error!("username {} is not registered.", username);
                return Err(anyhow!("username {} is not registered", username));
            }
        };

        // CPU-heavy 검증은 DB 커넥션 반납 후 수행
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
