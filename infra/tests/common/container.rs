use sqlx::PgPool;
use std::sync::OnceLock;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

static POSTGRES_URL: OnceLock<String> = OnceLock::new();

/// PostgreSQL 테스트 컨테이너를 최초 1회 기동하고 연결 URL을 반환한다.
///
/// # 컨테이너 수명 관리
/// - `OnceLock`: 테스트 바이너리당 컨테이너 1개만 기동 (재사용)
/// - `Box::leak`: `ContainerAsync`를 프로세스 종료까지 유지
/// - Ryuk 사이드카: 프로세스 종료 시 컨테이너 자동 삭제
///
/// # 중첩 런타임 회피
/// `#[tokio::test]`가 만든 런타임 내부에서 `block_on`을 호출하면
/// "Cannot start a runtime from within a runtime" 패닉이 발생한다.
/// `std::thread::spawn`으로 별도 OS 스레드를 만들어 독립 런타임에서 실행한다.
///
/// # 요구사항
/// - Docker가 실행 중이어야 한다.
/// - 각 테스트는 트랜잭션 시작 → 검증 → 롤백으로 DB 상태를 격리한다.
pub fn postgres_url() -> String {
    POSTGRES_URL
        .get_or_init(|| {
            std::thread::spawn(|| {
                tokio::runtime::Runtime::new()
                    .expect("Failed to create container runtime")
                    .block_on(async {
                        // ContainerAsync를 leak → 프로세스 종료까지 유지
                        // Ryuk이 프로세스 종료 시 Docker 컨테이너를 자동 삭제
                        let container = Box::leak(Box::new(
                            Postgres::default()
                                .start()
                                .await
                                .expect("Failed to start Postgres container"),
                        ));
                        let port = container
                            .get_host_port_ipv4(5432)
                            .await
                            .expect("Failed to get Postgres port");
                        let url = format!(
                            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                            port
                        );

                        let pool = PgPool::connect(&url)
                            .await
                            .expect("마이그레이션용 DB 연결 실패");
                        sqlx::migrate!("../migrations")
                            .run(&pool)
                            .await
                            .expect("마이그레이션 실행 실패");
                        pool.close().await;

                        url
                    })
            })
            .join()
            .expect("Container setup thread panicked")
        })
        .clone()
}
