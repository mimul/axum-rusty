use axum::Router;
use common::config::ApplicationConfig;
use controller::module::usecase_module::{AppModule, AppState};
use infra::db::{Db, DbParameters};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub fn test_config(database_url: String) -> ApplicationConfig {
    ApplicationConfig {
        debug: true,
        database_url,
        jwt_secret: "test-jwt-secret-key-for-testing".to_string(),
        allowed_origin: "http://localhost:3000".to_string(),
        jwt_duration: "60".to_string(),
        jwt_max_age: 1,
    }
}

// 각 테스트마다 독립된 풀을 생성한다.
// OnceCell 공유 풀은 22개 병렬 테스트가 동시에 DB를 접근할 때
// 커넥션 경합으로 인한 쿼리 실패를 일으키므로 사용하지 않는다.
pub async fn build_test_app() -> Router {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("테스트 DB 연결 실패");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("마이그레이션 실패");

    // try_acquire()가 즉시 idle 연결을 반환하도록 풀 워밍업
    let _conn = pool.acquire().await.expect("pool warmup 실패");
    drop(_conn);

    let config = test_config(db_url);
    let module = Arc::new(
        AppModule::builder()
            .with_component_parameters::<Db>(DbParameters { pool })
            .build(),
    );
    let state = Arc::new(AppState::new(module, config));
    controller::startup::build_router(state)
}
