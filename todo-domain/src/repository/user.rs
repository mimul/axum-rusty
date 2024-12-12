use async_trait::async_trait;
use crate::model::Id;
use crate::model::user::{NewUser, User};

#[async_trait]
pub trait UserRepository {
    async fn get_user(&self, id: &Id<User>) -> anyhow::Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
    async fn insert(&self, source: NewUser) -> anyhow::Result<User>;
}