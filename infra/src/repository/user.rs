use crate::model::user::{InsertUser, StoredUser};
use crate::module::uow::SharedTx;
use anyhow::Context;
use async_trait::async_trait;
use domain::model::user::{NewUser, User};
use domain::model::Id;
use domain::repository::user::UserRepository;
use sqlx::{query, query_as};

pub struct PgUserRepo {
    tx: SharedTx,
}

impl PgUserRepo {
    pub fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl UserRepository for PgUserRepo {
    async fn get_user(&self, id: &Id<User>) -> anyhow::Result<Option<User>> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().context("transaction not active")?;
        let sql = r#"
            SELECT id, username, email, password, fullname
            FROM users
            WHERE id = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(id.value.to_string())
            .fetch_optional(&mut **tx)
            .await?;
        match result {
            Some(su) => Ok(Some(su.try_into()?)),
            None => Ok(None),
        }
    }

    async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().context("transaction not active")?;
        let sql = r#"
            SELECT id, username, email, password, fullname
            FROM users
            WHERE username = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(username)
            .fetch_optional(&mut **tx)
            .await?;
        match result {
            Some(su) => Ok(Some(su.try_into()?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, source: NewUser) -> anyhow::Result<User> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().context("transaction not active")?;
        let user: InsertUser = source.into();

        query("INSERT INTO users (id, username, email, password, fullname) VALUES ($1, $2, $3, $4, $5)")
            .bind(&user.id)
            .bind(&user.username)
            .bind(&user.username) // email = username (현재 스키마 유지)
            .bind(&user.password)
            .bind(&user.fullname)
            .execute(&mut **tx)
            .await?;

        let sql = r#"
            SELECT id, username, email, password, fullname
            FROM users
            WHERE id = $1
        "#;
        let result = query_as::<_, StoredUser>(sql)
            .bind(&user.id)
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.try_into()?)
    }
}
