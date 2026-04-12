use anyhow::anyhow;
use sqlx::PgPool;

pub struct HealthCheckRepository {
    pool: PgPool,
}

impl HealthCheckRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check_connection(&self) -> anyhow::Result<()> {
        self.pool
            .try_acquire()
            .map(|_| ())
            .ok_or_else(|| anyhow!("Failed to connect database `postgres`."))
    }
}
