use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// 테스트 컨테이너의 PostgreSQL에 연결된 PgPool을 반환한다.
///
/// 컨테이너는 최초 접근 시 자동으로 기동된다 (Docker 필요).
/// 마이그레이션은 컨테이너 초기화 시 1회만 실행된다.
/// 각 테스트는 트랜잭션 시작 → 검증 → 롤백 패턴으로 DB 상태를 격리한다.
pub async fn setup_test_db() -> PgPool {
    let url = super::container::postgres_url();
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url)
        .await
        .expect("Failed to connect to test PostgreSQL container")
}
