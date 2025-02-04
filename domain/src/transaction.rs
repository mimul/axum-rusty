use sqlx::{Acquire, Postgres};
// https://qiita.com/FuJino/items/08b4c3298918191eab65
pub trait PostgresAcquire<'c>: Acquire<'c, Database = Postgres> + Send {}
impl<'c, T> PostgresAcquire<'c> for T
where
    T: Acquire<'c, Database = Postgres> + Send,
{}