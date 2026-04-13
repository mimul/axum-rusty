pub mod health_check;
pub mod todo;
pub mod user;

/// 레포지토리 쓰기 메서드에서 공유하는 트랜잭션 타입.
pub type PgTx = sqlx::Transaction<'static, sqlx::Postgres>;
