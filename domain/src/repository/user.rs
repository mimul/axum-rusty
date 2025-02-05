use crate::model::user::{NewUser, User};
use crate::model::Id;
use async_trait::async_trait;
use crate::transaction::PostgresAcquire;

#[async_trait]
pub trait UserRepository {
    async fn get_user(&self, id: &Id<User>, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Option<User>>;
    async fn insert(&self, source: NewUser, executor: impl PostgresAcquire<'_>) -> anyhow::Result<User>;
}
