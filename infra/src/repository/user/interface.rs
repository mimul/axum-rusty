use crate::repository::PgTx;
use async_trait::async_trait;
use domain::model::user::{NewUser, User};
use domain::model::Id;

/// User 레포지토리 인터페이스.
#[async_trait]
pub trait IUserRepository: shaku::Interface {
    async fn get_user(&self, id: &Id<User>) -> anyhow::Result<Option<User>>;
    async fn get_user_tx(&self, tx: &mut PgTx, id: &Id<User>) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username_tx(
        &self,
        tx: &mut PgTx,
        username: &str,
    ) -> anyhow::Result<Option<User>>;
    async fn insert_tx(&self, tx: &mut PgTx, source: NewUser) -> anyhow::Result<User>;
}
