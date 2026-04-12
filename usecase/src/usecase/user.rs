use crate::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use crate::module::uow::UserUnitOfWorkFactory;

/// bcrypt 해시 cost factor (OWASP 권고: 10 이상)
const BCRYPT_COST: u32 = 12;
use anyhow::anyhow;
use domain::model::user::User;
use log::{error, info};
use std::sync::Arc;

pub struct UserUseCase {
    uow_factory: Arc<dyn UserUnitOfWorkFactory>,
}

impl UserUseCase {
    pub fn new(uow_factory: Arc<dyn UserUnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }

    pub async fn get_user(&self, id: String) -> anyhow::Result<Option<UserView>> {
        let uow = self.uow_factory.begin().await?;
        let resp = uow.user_repo().get_user(&id.try_into()?).await?;
        Ok(resp.map(Into::into))
    }

    pub async fn get_user_by_username(
        &self,
        condition: SearchUserCondition,
    ) -> anyhow::Result<Option<UserView>> {
        let username = condition.username.ok_or_else(|| anyhow!("username is empty"))?;
        let uow = self.uow_factory.begin().await?;
        let resp = uow.user_repo().get_user_by_username(&username).await?;
        Ok(resp.map(Into::into))
    }

    pub async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        // CPU-heavy 작업은 트랜잭션 시작 전에 완료
        let hashed_password = bcrypt::hash(source.password.clone(), BCRYPT_COST)?;
        if hashed_password.is_empty() {
            return Err(anyhow!("hashed password is empty"));
        }

        let mut uow = self.uow_factory.begin().await?;

        // 읽기: username 중복 확인
        if uow.user_repo().get_user_by_username(&source.username).await?.is_some() {
            error!("username {} already exists", source.username);
            return Err(anyhow!("username {} already exists", source.username));
        }

        // 쓰기: insert
        let user = CreateUser::new(source.username, hashed_password, source.fullname);
        let user_view = uow.user_repo().insert(user.try_into()?).await?;
        uow.commit().await?;
        Ok(user_view.into())
    }

    pub async fn login_user(&self, source: LoginUser) -> anyhow::Result<UserView> {
        let uow = self.uow_factory.begin().await?;
        let user: User = uow
            .user_repo()
            .get_user_by_username(&source.username)
            .await?
            .ok_or_else(|| {
                error!("username {} is not registered.", source.username);
                anyhow!("username {} is not registered", source.username)
            })?;

        // CPU-heavy 검증은 DB 커넥션 반납 후 수행
        drop(uow);
        let login_result = bcrypt::verify(source.password.clone(), user.password.as_str())?;
        if login_result {
            info!("login succeeded!");
            Ok(user.into())
        } else {
            error!("bad password.");
            Err(anyhow!("bad password."))
        }
    }
}
