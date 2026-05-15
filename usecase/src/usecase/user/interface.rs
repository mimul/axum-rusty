use crate::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use async_trait::async_trait;

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
