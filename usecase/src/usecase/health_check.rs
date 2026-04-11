use async_trait::async_trait;
use std::sync::Arc;

/// 헬스체크 인프라 포트 인터페이스.
///
/// 구현체는 `infra` 크레이트의 `HealthCheckRepository`가 담당한다.
#[async_trait]
pub trait HealthCheckPort: Send + Sync {
    async fn check_connection(&self) -> anyhow::Result<()>;
}

pub struct HealthCheckUseCase {
    repository: Arc<dyn HealthCheckPort>,
}

impl HealthCheckUseCase {
    pub fn new(repository: impl HealthCheckPort + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn diagnose_db_conn(&self) -> anyhow::Result<()> {
        self.repository.check_connection().await
    }
}
