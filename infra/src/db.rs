use anyhow::Context;
use common::config::ApplicationConfig;
use log::LevelFilter;
use shaku::Component;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Pool, Postgres};
use std::str::FromStr;
use std::time::Duration;

const MAX_POOL_CONNECTIONS: u32 = 10;

pub type PgPool = Pool<Postgres>;

/// 데이터베이스 커넥션 풀 인터페이스.
///
/// 레포지토리 / 유스케이스가 Pool에 직접 의존하지 않고
/// 이 트레이트를 통해 주입받는다.
pub trait IDatabasePool: shaku::Interface {
    fn pool(&self) -> &PgPool;
}

/// PostgreSQL 커넥션 풀 컴포넌트.
///
/// `pool` 은 shaku 파라미터로 제공된다.
/// `AppModule::builder().with_component_parameters::<Db>(DbParameters { pool })` 로 초기화한다.
#[derive(Component)]
#[shaku(interface = IDatabasePool)]
pub struct Db {
    pool: PgPool,
}

impl IDatabasePool for Db {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// ApplicationConfig 로부터 PgPool 을 생성한다.
///
/// bootstrap 에서 한 번 호출하여 AppModule 에 파라미터로 전달한다.
pub async fn create_pool(config: &ApplicationConfig) -> anyhow::Result<PgPool> {
    let pg_options = PgConnectOptions::from_str(config.database_url.as_str())
        .with_context(|| format!("DB URL 파싱 실패: {}", config.database_url))?;
    let pg_options = pg_options
        .log_statements(LevelFilter::Trace)
        .log_slow_statements(LevelFilter::Info, Duration::from_millis(250))
        .clone();
    PgPoolOptions::new()
        .max_connections(MAX_POOL_CONNECTIONS)
        .connect_with(pg_options)
        .await
        .context("DB 연결 실패. 설정을 확인해 주세요.")
}
