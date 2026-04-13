use async_trait::async_trait;
use infra::repository::health_check::IHealthCheckRepository;
use shaku::Component;
use std::sync::Arc;

/// HealthCheck 유스케이스 인터페이스.
#[async_trait]
pub trait IHealthCheckUseCase: shaku::Interface {
    async fn diagnose_db_conn(&self) -> anyhow::Result<()>;
}

/// HealthCheck 유스케이스 구현체.
#[derive(Component)]
#[shaku(interface = IHealthCheckUseCase)]
pub struct HealthCheckUseCase {
    #[shaku(inject)]
    repository: Arc<dyn IHealthCheckRepository>,
}

#[async_trait]
impl IHealthCheckUseCase for HealthCheckUseCase {
    async fn diagnose_db_conn(&self) -> anyhow::Result<()> {
        self.repository.check_connection().await
    }
}
