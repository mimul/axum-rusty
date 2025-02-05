use sqlx::{Acquire, Postgres};

pub trait PostgresAcquire<'c>: Acquire<'c, Database = Postgres> + Send {}
impl<'c, T> PostgresAcquire<'c> for T
where
    T: Acquire<'c, Database = Postgres> + Send {}