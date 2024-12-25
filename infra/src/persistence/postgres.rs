use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Pool, Postgres};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::log::LevelFilter;
use crate::persistence::config::Config;

#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Postgres>>);

impl Db {
    pub async fn new(config: Config) -> Db {
        let pg_options = PgConnectOptions::from_str(config.database_url.as_str()).unwrap_or_else(|_| panic!("Error connecting to {}", config.database_url.as_str()))
            .log_statements(LevelFilter::Trace)
            .log_slow_statements(LevelFilter::Info, Duration::from_millis(250))
            .clone();
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_with(pg_options)
            //.connect(config.database_url.as_str())
            .await
            .unwrap_or_else(|_| {
                panic!("Cannot connect to the database. Please check your configuration.")
            });
        Db(Arc::new(pool))
    }
}
