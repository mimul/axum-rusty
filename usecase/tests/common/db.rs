use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// TEST_DATABASE_URL 환경변수로 연결 후 마이그레이션을 실행한 PgPool을 반환.
/// 각 테스트는 필요한 경우 트랜잭션 시작 → 검증 → 롤백 패턴으로 DB 상태를 오염시키지 않는다.
pub async fn setup_test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL env var is required for DB tests");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to test database");
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");
    pool
}
