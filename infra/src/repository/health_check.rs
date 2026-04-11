use crate::persistence::postgres::Db;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use usecase::usecase::health_check::HealthCheckPort;

pub struct HealthCheckRepository {
    db: Arc<Db>,
}

impl HealthCheckRepository {
    pub fn new(db: Db) -> Self {
        Self { db: Arc::new(db) }
    }
}

#[async_trait]
impl HealthCheckPort for HealthCheckRepository {
    async fn check_connection(&self) -> anyhow::Result<()> {
        let pool = self.db.0.clone();
        pool.try_acquire()
            .map(|_| ())
            .ok_or_else(|| anyhow!("Failed to connect database `postgres`."))
    }
}
