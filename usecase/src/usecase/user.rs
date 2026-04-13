use crate::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use anyhow::anyhow;
use async_trait::async_trait;
use domain::model::user::User;
use infra::db::IDatabasePool;
use infra::repository::user::IUserRepository;
use log::{error, info};
use shaku::Component;
use std::sync::Arc;

/// bcrypt 해시 cost factor (OWASP 권고: 10 이상)
const BCRYPT_COST: u32 = 12;

/// User 유스케이스 인터페이스.
#[async_trait]
pub trait IUserUseCase: shaku::Interface {
    async fn get_user(&self, id: String) -> anyhow::Result<Option<UserView>>;
    async fn get_user_by_username(
        &self,
        condition: SearchUserCondition,
    ) -> anyhow::Result<Option<UserView>>;
    async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView>;
    async fn login_user(&self, source: LoginUser) -> anyhow::Result<UserView>;
}

/// User 유스케이스 구현체.
#[derive(Component)]
#[shaku(interface = IUserUseCase)]
pub struct UserUseCase {
    #[shaku(inject)]
    db: Arc<dyn IDatabasePool>,
    #[shaku(inject)]
    user_repo: Arc<dyn IUserRepository>,
}

#[async_trait]
impl IUserUseCase for UserUseCase {
    async fn get_user(&self, id: String) -> anyhow::Result<Option<UserView>> {
        let resp = self.user_repo.get_user(&id.try_into()?).await?;
        Ok(resp.map(Into::into))
    }

    async fn get_user_by_username(
        &self,
        condition: SearchUserCondition,
    ) -> anyhow::Result<Option<UserView>> {
        let username = condition
            .username
            .ok_or_else(|| anyhow!("username is empty"))?;
        let resp = self.user_repo.get_user_by_username(&username).await?;
        Ok(resp.map(Into::into))
    }

    async fn create_user(&self, source: CreateUser) -> anyhow::Result<UserView> {
        // CPU-heavy 작업은 트랜잭션 시작 전에 완료
        let hashed_password = bcrypt::hash(source.password.clone(), BCRYPT_COST)?;

        let mut tx = self.db.pool().begin().await?;

        // 읽기: username 중복 확인
        if self
            .user_repo
            .get_user_by_username_tx(&mut tx, &source.username)
            .await?
            .is_some()
        {
            error!("username {} already exists", source.username);
            return Err(anyhow!("username {} already exists", source.username));
        }

        // 쓰기: insert
        let user = CreateUser::new(source.username, hashed_password, source.fullname);
        let user_view = self.user_repo.insert_tx(&mut tx, user.try_into()?).await?;
        tx.commit().await?;
        Ok(user_view.into())
    }

    async fn login_user(&self, source: LoginUser) -> anyhow::Result<UserView> {
        let user: User = self
            .user_repo
            .get_user_by_username(&source.username)
            .await?
            .ok_or_else(|| {
                error!("username {} is not registered.", source.username);
                anyhow!("username {} is not registered", source.username)
            })?;

        // CPU-heavy 검증
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
