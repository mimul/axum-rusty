use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::PgPool;
use usecase::usecase::health_check::HealthCheckPort;

pub struct HealthCheckRepository {
    pool: PgPool,
}

impl HealthCheckRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthCheckPort for HealthCheckRepository {
    async fn check_connection(&self) -> anyhow::Result<()> {
        self.pool
            .try_acquire()
            .map(|_| ())
            .ok_or_else(|| anyhow!("Failed to connect database `postgres`."))
    }
}
