use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// TEST_DATABASE_URL 환경변수로 연결 후 마이그레이션을 실행한 PgPool을 반환.
/// 각 테스트는 트랜잭션 시작 → 검증 → 롤백 패턴으로 DB 상태를 오염시키지 않는다.
///
/// max_connections=2 로 제한하여 병렬 테스트 실행 시 커넥션 풀 고갈을 방지한다.
/// (#[tokio::test]는 테스트마다 독립 런타임을 생성하므로 OnceCell 공유 불가)
/// 테스트마다 독립 런타임이 풀을 생성하므로 커넥션 고갈 방지를 위해 제한
const TEST_POOL_MAX_CONNECTIONS: u32 = 2;

pub async fn setup_test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL env var is required for DB tests");
    let pool = PgPoolOptions::new()
        .max_connections(TEST_POOL_MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url)
        .await
        .expect("Failed to connect to test database");
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");
    pool
}
