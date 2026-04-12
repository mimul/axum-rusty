use crate::model::user::{InsertUser, StoredUser};
use crate::repository::todo::PgTx;
use domain::model::user::{NewUser, User};
use domain::model::Id;
use sqlx::{query, query_as, PgPool};

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // -----------------------------------------------------------------------
    // 읽기
    // -----------------------------------------------------------------------

    pub async fn get_user(&self, id: &Id<User>) -> anyhow::Result<Option<User>> {
        find_user_by_id(&self.pool, id).await
    }

    pub async fn get_user_tx(&self, tx: &mut PgTx, id: &Id<User>) -> anyhow::Result<Option<User>> {
        find_user_by_id(&mut **tx, id).await
    }

    pub async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        find_user_by_username(&self.pool, username).await
    }

    pub async fn get_user_by_username_tx(
        &self,
        tx: &mut PgTx,
        username: &str,
    ) -> anyhow::Result<Option<User>> {
        find_user_by_username(&mut **tx, username).await
    }

    // -----------------------------------------------------------------------
    // 쓰기
    // -----------------------------------------------------------------------

    pub async fn insert_tx(&self, tx: &mut PgTx, source: NewUser) -> anyhow::Result<User> {
        let user: InsertUser = source.into();

        query(
            "INSERT INTO users (id, username, email, password, fullname) VALUES ($1, $2, $3, $4, $5)",
        )
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
        result.try_into()
    }
}

// -----------------------------------------------------------------------
// Private helpers — generic executor로 pool / tx 모두 처리
// -----------------------------------------------------------------------

async fn find_user_by_id<'e, E>(executor: E, id: &Id<User>) -> anyhow::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let sql = r#"
        SELECT id, username, email, password, fullname
        FROM users
        WHERE id = $1
    "#;
    query_as::<_, StoredUser>(sql)
        .bind(id.value.to_string())
        .fetch_optional(executor)
        .await?
        .map(|su| su.try_into())
        .transpose()
}

async fn find_user_by_username<'e, E>(executor: E, username: &str) -> anyhow::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let sql = r#"
        SELECT id, username, email, password, fullname
        FROM users
        WHERE username = $1
    "#;
    query_as::<_, StoredUser>(sql)
        .bind(username)
        .fetch_optional(executor)
        .await?
        .map(|su| su.try_into())
        .transpose()
}
