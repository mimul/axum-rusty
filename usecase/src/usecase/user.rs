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
/// cost=10: ~80~150ms / cost=12: ~400~1000ms (2의 지수 증가)
const BCRYPT_COST: u32 = 10;

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
        // bcrypt::hash는 CPU-blocking → spawn_blocking으로 tokio worker thread 분리
        let password = source.password.clone();
        let hashed_password: String =
            tokio::task::spawn_blocking(move || bcrypt::hash(&password, BCRYPT_COST))
                .await
                .map_err(|e| anyhow!("bcrypt hash task panicked: {e}"))??;

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

        // bcrypt::verify는 CPU-blocking → spawn_blocking으로 tokio worker thread 분리
        let input_password = source.password.clone();
        let stored_hash = user.password.clone();
        let login_result: bool =
            tokio::task::spawn_blocking(move || bcrypt::verify(&input_password, &stored_hash))
                .await
                .map_err(|e| anyhow!("bcrypt verify task panicked: {e}"))??;
        if login_result {
            info!("login succeeded!");
            Ok(user.into())
        } else {
            error!("bad password.");
            Err(anyhow!("bad password."))
        }
    }
}
