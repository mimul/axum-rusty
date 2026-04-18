use sqlx::PgPool;
use std::sync::{Mutex, OnceLock};
use testcontainers::{core::ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

static POSTGRES_URL: OnceLock<String> = OnceLock::new();

/// 이 바이너리에서 기동한 컨테이너 ID 목록.
/// `#[ctor::dtor]`가 프로세스 종료 시 일괄 삭제한다.
static CONTAINER_IDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// 프로세스 종료 시 자동 실행 — 테스트 컨테이너를 명시적으로 삭제한다.
///
/// Ryuk 대신 `#[ctor::dtor]`를 사용하는 이유:
/// Mac Docker Desktop은 `/var/run/docker.sock` 대신
/// `~/.docker/run/docker.sock`을 사용하여 Ryuk TCP 세션이 연결되지 않는다.
/// `docker rm -f`를 직접 호출하면 소켓 경로와 무관하게 정리된다.
#[ctor::dtor]
fn cleanup_test_containers() {
    let ids = CONTAINER_IDS.lock().unwrap_or_else(|e| e.into_inner());
    for id in ids.iter() {
        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", id])
            .output();
    }
}

/// 현재 실행 중인 테스트 바이너리 이름을 반환한다.
/// 빌드 해시(`binary_name-<hash>`)는 제거하고 최대 20자로 자른다.
fn binary_base_name() -> String {
    let full = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "test".into());
    let base = full
        .rsplit_once('-')
        .map(|(b, _)| b)
        .unwrap_or(&full)
        .to_string();
    base[..base.len().min(20)].to_string()
}

/// PostgreSQL 테스트 컨테이너를 최초 1회 기동하고 연결 URL을 반환한다.
///
/// # 컨테이너 수명 관리
/// - `OnceLock`: 바이너리당 컨테이너 1개만 기동 (재사용)
/// - `Box::leak`: `ContainerAsync`의 Drop을 막아 컨테이너를 살려 둠
/// - `#[ctor::dtor]`: 프로세스 종료 시 `docker rm -f`로 명시적 삭제
///
/// # 컨테이너 이름
/// `test_pg_<바이너리명>_<PID>` 형식으로 지정한다.
///
/// # 중첩 런타임 회피
/// `std::thread::spawn`으로 별도 OS 스레드를 만들어
/// `#[tokio::test]`의 런타임과 완전히 분리된다.
pub fn postgres_url() -> String {
    POSTGRES_URL
        .get_or_init(|| {
            std::thread::spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build runtime")
                    .block_on(async {
                        let name =
                            format!("test_pg_{}_{}", binary_base_name(), std::process::id());
                        let container = Postgres::default()
                            .with_container_name(&name)
                            .start()
                            .await
                            .expect("Failed to start Postgres container");

                        // 컨테이너 ID를 dtor 정리 목록에 등록
                        CONTAINER_IDS
                            .lock()
                            .unwrap()
                            .push(container.id().to_string());

                        let port = container
                            .get_host_port_ipv4(5432)
                            .await
                            .expect("Failed to get host port");
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

                        // Box::leak으로 Drop을 막아 컨테이너를 프로세스 종료까지 유지
                        // 삭제는 #[ctor::dtor]가 담당
                        Box::leak(Box::new(container));

                        url
                    })
            })
            .join()
            .expect("Container setup thread panicked")
        })
        .clone()
}
