# TEST_RUST 카탈로그 (T-T-01 ~ T-T-06)

이 카탈로그는 `/test-rust` 스킬이 테스트를 분류·작성할 때 사용하는 기준표다.
테스트 철학·Mocking 규칙·네이밍·PR 기준은 `rules/test-style.md`가 권위 문서다.
이 파일은 **Rust 프로젝트 고유의 구현 패턴**만 정의한다.

---

## 카탈로그 요약

| 코드 | 종류 | 작성 위치 | 외부 의존 | 핵심 조건 |
|------|------|-----------|-----------|-----------|
| **T-T-01** | 단위 테스트 | `src/**/*.rs` 내 `#[cfg(test)]` | 없음 | 순수 도메인 로직만 |
| **T-T-02** | Repository DB 테스트 | `{crate}/tests/` | PostgreSQL (testcontainers) | 트랜잭션 롤백 |
| **T-T-03** | Usecase 통합 테스트 | `{crate}/tests/` | PostgreSQL (testcontainers) | 실제 DB + 비즈니스 흐름 |
| **T-T-04** | HTTP API 테스트 | `{crate}/tests/` | axum TestClient | 엔드포인트 전체 |
| **T-T-05** | 프로퍼티 기반 테스트 | `src/**/*.rs` 내 `#[cfg(test)]` | 없음 | 불변 조건 |
| **T-T-06** | 공통 테스트 헬퍼 | `{crate}/tests/common/` | testcontainers | 재사용 픽스처 |

> **DB 테스트 원칙**: 모든 DB 기반 테스트는 `TEST_DATABASE_URL` 환경변수 없이
> testcontainers가 Docker를 자동으로 기동한다. `tests/common/container.rs` 참조.

---

## T-T-01 — 단위 테스트 (Unit Test)

### 작성 위치
```
src/
└── {crate}/src/{module}.rs     ← 같은 파일 하단에 #[cfg(test)] 블록
```

### 대상
- 도메인 모델 생성자 / Newtype 변환 / 불변 조건
- `From` / `TryFrom` / `Into` / 순수 변환 함수
- 에러 분기가 있는 순수 비즈니스 로직
- DB·외부 시스템과 무관한 계산·검증 로직

> **중요**: Repository·Usecase 협력 테스트는 **T-T-02/T-T-03(실제 DB)**으로 작성한다.
> 내부 모듈(Repository, Usecase)을 mock하는 것은 `rules/test-style.md §1. 모킹 경계`에서 금지한다.

### 패턴
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 순수 값 객체 / 도메인 로직 테스트
    #[test]
    fn create_user_id_succeeds_when_valid_uuid() {
        let id = UserId::parse("550e8400-e29b-41d4-a716-446655440000");
        assert!(id.is_ok());
    }

    #[test]
    fn create_user_id_fails_when_invalid_format() {
        let result = UserId::parse("not-a-uuid");
        assert!(result.is_err());
        // 에러 메시지에 내부 구현 정보가 노출되지 않는지도 검증
        assert!(!result.unwrap_err().to_string().contains("parse_str"));
    }

    #[test]
    fn validate_username_fails_when_empty() {
        let result = Username::new("");
        assert!(matches!(result, Err(DomainError::EmptyUsername)));
    }

    // 비동기 순수 로직 (외부 I/O 없음)
    #[tokio::test]
    async fn hash_password_returns_different_hash_each_time() {
        let h1 = hash_password("secret").await.unwrap();
        let h2 = hash_password("secret").await.unwrap();
        assert_ne!(h1, h2, "bcrypt salt must be unique");
    }
}
```

### 필수 케이스
- 정상 케이스 (happy path)
- 에러 케이스 (모든 `Result::Err` 분기)
- 경계 케이스 (빈 문자열, None, 최댓값 등)

### 작성 기준 (`rules/test-style.md §4. 테스트 피라미드` 준수)
- Getter, DI 연결 코드, 단순 CRUD 위임 → **작성하지 않는다**
- 비즈니스 규칙·상태 전환·복잡한 분기 → **반드시 작성한다**

---

## T-T-02 — Repository DB 테스트

### 작성 위치
```
{crate}/tests/
├── common/
│   ├── mod.rs
│   └── container.rs    ← postgres_url() — testcontainers 기반
└── {entity}_repository_test.rs
```

> **Rust 통합 테스트 주의**: `tests/` 바로 아래 `.rs` 파일만 별도 바이너리로 인식한다.
> 서브디렉토리(`tests/db/foo.rs`)는 모듈로만 사용 가능하며 `cargo test --test`로 직접 지정할 수 없다.

### 대상
- `infra/` 크레이트의 Repository 구현체
- 실제 SQL 쿼리 검증 (CRUD + 에지 케이스)
- Migration 후 스키마 정합성

### 환경
- testcontainers가 Docker를 자동으로 기동한다 (`tests/common/container.rs`)
- 각 테스트는 트랜잭션 시작 → 실행 → **반드시 롤백**
- 환경변수(`TEST_DATABASE_URL`) 불필요

### 패턴
```rust
// {crate}/tests/{entity}_repository_test.rs
mod common;

use common::container::postgres_url;
use sqlx::PgPool;

async fn setup() -> PgPool {
    PgPool::connect(&postgres_url()).await.expect("DB 연결 실패")
}

#[tokio::test]
async fn insert_user_succeeds_when_input_is_valid() {
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();

    let repo = PgUserRepository::new(&mut tx);
    let user = fixture_new_user();
    let created = repo.insert(&user).await.unwrap();

    assert_eq!(created.username, user.username);
    // 관찰 가능한 상태 검증 (DB에 실제 저장됐는지)
    let found = repo.find_by_id(&created.id).await.unwrap();
    assert!(found.is_some());

    tx.rollback().await.unwrap();   // 항상 롤백
}

#[tokio::test]
async fn find_user_returns_none_when_not_found() {
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();

    let repo = PgUserRepository::new(&mut tx);
    let result = repo.find_by_id(&UserId::new()).await.unwrap();

    assert!(result.is_none());
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn insert_user_fails_when_username_is_duplicate() {
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();

    let repo = PgUserRepository::new(&mut tx);
    let user = fixture_new_user();
    repo.insert(&user).await.unwrap();
    let result = repo.insert(&user).await;  // 중복 삽입

    assert!(result.is_err());
    tx.rollback().await.unwrap();
}
```

### 필수 케이스
- insert → find 정합성
- 존재하지 않는 ID 조회 → None 반환
- 중복 insert → Err 반환
- 조건 필터링 정확성

---

## T-T-03 — Usecase 통합 테스트

### 작성 위치
```
{crate}/tests/
├── common/
│   ├── mod.rs
│   └── container.rs    ← postgres_url()
└── {usecase}_integration_test.rs
```

### 대상
- `usecase/` 크레이트의 Usecase 구현체
- 여러 Repository 협력을 통한 비즈니스 흐름
- 트랜잭션 롤백 동작 검증

> **Mock 금지**: `rules/test-style.md §1. 모킹 경계` — DB/ORM·내부 Repository는 mock하지 않는다.
> Usecase 테스트는 실제 DB(testcontainers)와 실제 Repository 구현체를 사용한다.

### 패턴 — 정상 흐름 (실제 DB)
```rust
// {crate}/tests/{usecase}_integration_test.rs
mod common;

use common::container::postgres_url;
use sqlx::PgPool;
use std::sync::Arc;
use infra::persistence::postgres::Db;

async fn setup() -> PgPool {
    PgPool::connect(&postgres_url()).await.expect("DB 연결 실패")
}

#[tokio::test]
async fn create_user_succeeds_when_input_is_valid() {
    let pool = setup().await;
    let db = Db(Arc::new(pool.clone()));
    let usecase = UserUseCase::new(db);

    let input = CreateUserInput {
        username: "alice".to_string(),
        password: "secret123".to_string(),
    };
    let result = usecase.create_user(input).await;

    assert!(result.is_ok(), "{result:?}");
    let user = result.unwrap();
    assert_eq!(user.username, "alice");
    // 부작용(DB 쓰기) 명시적 검증
    let found = usecase.get_user(&user.id).await.unwrap();
    assert!(found.is_some());
}
```

### 패턴 — 트랜잭션 롤백 검증
```rust
/// Usecase 내부 실패 시 트랜잭션이 자동 롤백되어
/// DB 상태가 변경되지 않음을 검증한다.
#[tokio::test]
async fn create_user_fails_when_username_is_duplicate() {
    let pool = setup().await;
    let db = Db(Arc::new(pool.clone()));
    let usecase = UserUseCase::new(db);

    // 1차 생성 (커밋)
    let input = CreateUserInput { username: "alice".to_string(), password: "pw".to_string() };
    usecase.create_user(input.clone()).await.unwrap();

    // 2차 생성 (중복 → 실패 → 롤백)
    let result = usecase.create_user(input).await;

    assert!(result.is_err());
    // DB 상태: 1명만 존재해야 함 (롤백 검증)
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = 'alice'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "rollback 후 중복 row 없어야 함");
}
```

### 필수 케이스
- 정상 흐름 (전체 비즈니스 시나리오)
- 비즈니스 규칙 위반 케이스
- 트랜잭션 롤백 (중간 실패 시 DB 상태 무변경 검증)
- 부작용(DB 쓰기·상태 변경) 명시적 검증

---

## T-T-04 — HTTP API 테스트

### 작성 위치
```
controller/tests/
├── common/
│   └── mod.rs          ← build_test_app()
└── {endpoint}_api_test.rs
```

### 대상
- `controller/` 크레이트의 라우터/핸들러
- HTTP 상태 코드, 응답 바디, 헤더 검증
- 인증 미들웨어 동작 검증

### 환경
- `axum::Router`를 직접 테스트 (실제 바인딩 불필요)
- `tower::ServiceExt::oneshot` 사용
- testcontainers가 DB를 자동 기동 (`tests/common/mod.rs`의 `build_test_app()`)

### 패턴
```rust
// controller/tests/{endpoint}_api_test.rs
mod common;

use axum::http::{Request, StatusCode};
use common::build_test_app;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn create_user_returns_201_when_input_is_valid() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"username":"alice","password":"secret123","name":"Alice"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["username"], "alice");
    // 내부 구현 정보(비밀번호 해시 등)가 응답에 없는지 검증
    assert!(json.get("password_hash").is_none());
}

#[tokio::test]
async fn create_user_returns_400_when_username_is_empty() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"username":"","password":"pw","name":"A"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_user_returns_401_when_not_authenticated() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### 필수 케이스
- 200/201 정상 응답 + 응답 바디 스키마 검증
- 400 잘못된 요청 바디
- 401 인증 없음
- 404 리소스 없음
- 에러 응답에 내부 정보 미노출 (`security.md §에러 응답` 준수)

---

## T-T-05 — 프로퍼티 기반 테스트

> **작성 기준**: `rules/test-style.md §6. Property-Based Testing` 참조.
> 동일한 함수에 대해 네 번째 예제 테스트를 작성해야 한다면 이 카탈로그로 전환한다.

### 작성 위치
```
src/{crate}/src/{module}.rs 내 #[cfg(test)]
```

### 대상
- 복잡한 도메인 불변 조건 (validators, parsers, state machines)
- 입력 범위가 넓은 변환 함수
- DB·부작용 없는 순수 계산 로직

### 패턴
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn create_username_rejects_when_too_long(
        s in ".{51,200}",  // 50자 초과 문자열
    ) {
        prop_assert!(Username::new(&s).is_err());
    }

    #[test]
    fn serialize_deserialize_roundtrip_preserves_value(
        username in "[a-z][a-z0-9_]{2,29}",
    ) {
        let original = Username::new(&username).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Username = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(original, restored);
    }
}
```

### 사용 조건
- `proptest` 크레이트가 `[dev-dependencies]`에 추가된 경우에만 사용
- DB·외부 시스템과 무관한 순수 로직에만 적용

---

## T-T-06 — 공통 테스트 헬퍼

### 작성 위치
```
{crate}/tests/
└── common/
    ├── mod.rs          ← pub mod container; pub mod fixtures;
    ├── container.rs    ← postgres_url() — testcontainers 기반
    └── fixtures.rs     ← fixture_new_user(), fixture_new_todo()
```

API 테스트 크레이트(`controller`)는 추가로:
```
controller/tests/common/mod.rs  ← build_test_app() 포함
```

### container.rs 패턴 (testcontainers 0.23)
```rust
// {crate}/tests/common/container.rs
use sqlx::PgPool;
use std::sync::{Mutex, OnceLock};
use testcontainers::{core::ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

static POSTGRES_URL: OnceLock<String> = OnceLock::new();
static CONTAINER_IDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// 프로세스 종료 시 컨테이너 명시적 삭제
/// Mac Docker Desktop은 Ryuk이 동작하지 않으므로 docker rm -f를 직접 실행한다.
#[ctor::dtor]
fn cleanup_test_containers() {
    let ids = CONTAINER_IDS.lock().unwrap_or_else(|e| e.into_inner());
    for id in ids.iter() {
        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", id])
            .output();
    }
}

fn binary_base_name() -> String {
    let full = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "test".into());
    let base = full.rsplit_once('-').map(|(b, _)| b).unwrap_or(&full).to_string();
    base[..base.len().min(20)].to_string()
}

/// PostgreSQL 컨테이너를 바이너리당 1회 기동하고 연결 URL을 반환한다.
/// 컨테이너 이름: `test_pg_<바이너리명>_<PID>`
pub fn postgres_url() -> String {
    POSTGRES_URL
        .get_or_init(|| {
            std::thread::spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        let name = format!("test_pg_{}_{}", binary_base_name(), std::process::id());
                        let container = Postgres::default()
                            .with_container_name(&name)
                            .start()
                            .await
                            .expect("Postgres container 기동 실패");

                        CONTAINER_IDS.lock().unwrap().push(container.id().to_string());

                        let port = container.get_host_port_ipv4(5432).await.unwrap();
                        let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

                        let pool = PgPool::connect(&url).await.unwrap();
                        sqlx::migrate!("../migrations").run(&pool).await.unwrap();
                        pool.close().await;

                        Box::leak(Box::new(container));
                        url
                    })
            })
            .join()
            .unwrap()
        })
        .clone()
}
```

### fixtures.rs 패턴
```rust
// {crate}/tests/common/fixtures.rs
use domain::model::user::{NewUser, UserId};

pub fn fixture_new_user() -> NewUser {
    NewUser {
        id: UserId::new(),
        username: format!("user_{}", uuid::Uuid::new_v4().simple()),
        password_hash: "$2b$12$test_hash".to_string(),   // 하드코딩 시크릿 금지 — 테스트용 더미
        name: "Test User".to_string(),
    }
}
```

> **보안 주의**: 테스트 픽스처에 실제 API 키·토큰을 절대 하드코딩하지 않는다.
> `security.md §비밀 정보 관리` 참조.

### Cargo.toml dev-dependencies 템플릿
```toml
[dev-dependencies]
tokio                  = { version = "1", features = ["full"] }
testcontainers         = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
ctor                   = "0.2"
# API 테스트 전용 (controller 크레이트)
tower          = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
```

---

## 테스트 파일 위치 결정 기준

```
대상 코드                          카탈로그      파일 위치
────────────────────────────────────────────────────────────────
domain/src/model/*.rs            →  T-T-01  →  src 내부 #[cfg(test)]
domain/src/value_object/*.rs     →  T-T-01  →  src 내부 #[cfg(test)]
infra/src/repository/*.rs        →  T-T-02  →  infra/tests/*_repository_test.rs
usecase/src/usecase/*.rs         →  T-T-03  →  usecase/tests/*_integration_test.rs
controller/src/routes/*.rs       →  T-T-04  →  controller/tests/*_api_test.rs
복잡한 도메인 불변 조건            →  T-T-05  →  src 내부 #[cfg(test)]
공통 헬퍼·픽스처                  →  T-T-06  →  {crate}/tests/common/
```
