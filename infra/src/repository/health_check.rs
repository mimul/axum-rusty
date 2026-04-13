use crate::db::IDatabasePool;
use anyhow::anyhow;
use async_trait::async_trait;
use shaku::Component;
use std::sync::Arc;

/// HealthCheck 레포지토리 인터페이스.
#[async_trait]
pub trait IHealthCheckRepository: shaku::Interface {
    async fn check_connection(&self) -> anyhow::Result<()>;
}

/// PostgreSQL HealthCheck 레포지토리 구현체.
#[derive(Component)]
#[shaku(interface = IHealthCheckRepository)]
pub struct HealthCheckRepository {
    #[shaku(inject)]
    db: Arc<dyn IDatabasePool>,
}

#[async_trait]
impl IHealthCheckRepository for HealthCheckRepository {
    async fn check_connection(&self) -> anyhow::Result<()> {
        self.db
            .pool()
            .try_acquire()
            .map(|_| ())
            .ok_or_else(|| anyhow!("Failed to connect database `postgres`."))
    }
}
