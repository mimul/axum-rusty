use async_trait::async_trait;

/// HealthCheck 유스케이스 인터페이스.
#[async_trait]
pub trait IHealthCheckUseCase: shaku::Interface {
    async fn diagnose_db_conn(&self) -> anyhow::Result<()>;
}
