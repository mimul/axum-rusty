use sqlx::{Acquire, Postgres};

pub trait PgAcquire<'c>: Acquire<'c, Database = Postgres> + Send {}
impl<'c, T> PgAcquire<'c> for T
where
    T: Acquire<'c, Database = Postgres> + Send {}