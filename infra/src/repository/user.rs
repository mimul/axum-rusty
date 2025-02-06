use anyhow::Context;
use async_trait::async_trait;
use crate::model::user::{InsertUser, StoredUser};
use crate::repository::DatabaseRepositoryImpl;
use domain::model::user::{NewUser, User};
use domain::model::Id;
use domain::repository::user::UserRepository;
use sqlx::{query, query_as};
use domain::transaction::PgAcquire;

#[async_trait]
impl UserRepository for DatabaseRepositoryImpl<User> {
    async fn get_user(&self, id: &Id<User>, executor: impl PgAcquire<'_>) -> anyhow::Result<Option<User>> {
        let mut conn = executor.acquire().await.context("failed to acquire postgres connection")?;
        let sql = r#"
            select u.id, u.username, u.email, u.password, u.fullname from users as u where u.id = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(id.value.to_string())
            .fetch_one(&mut *conn)
            .await
            .ok();

        match result {
            Some(su) => Ok(Some(su.try_into()?)),
            None => Ok(None),
        }
    }

    async fn get_user_by_username(&self, username: &str, executor: impl PgAcquire<'_>) -> anyhow::Result<Option<User>> {
        let mut conn = executor.acquire().await.context("failed to acquire postgres connection")?;
        let sql = r#"
            select u.id, u.username, u.email, u.password, u.fullname from users as u where u.username = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(username.to_string())
            .fetch_one(&mut *conn)
            .await
            .ok();

        match result {
            Some(su) => Ok(Some(su.try_into()?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, source: NewUser, executor: impl PgAcquire<'_>) -> anyhow::Result<User> {
        let mut conn = executor.acquire().await.context("failed to acquire postgres connection")?;
        let user: InsertUser = source.into();
        let id = user.id.clone();
        let username = user.username.clone();

        let _ = query("insert into users (id, username, email, password, fullname) values ($1, $2, $3, $4, $5)")
            .bind(user.id)
            .bind(user.username)
            .bind(username)
            .bind(user.password)
            .bind(user.fullname)
            .execute(&mut *conn)
            .await?;

        let sql = r#"
            select u.id, u.username, u.email, u.password, u.fullname
            from  users as u
            where u.id = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(result.try_into()?)
    }
}
