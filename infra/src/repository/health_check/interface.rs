use async_trait::async_trait;

/// HealthCheck 레포지토리 인터페이스.
#[async_trait]
pub trait IHealthCheckRepository: shaku::Interface {
    async fn check_connection(&self) -> anyhow::Result<()>;
}
