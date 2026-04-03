# TEST_RUST 카탈로그 (T-T-01 ~ T-T-06)

이 카탈로그는 `/test-rust` 스킬이 테스트를 분류·작성할 때 사용하는 기준표다.

---

## 카탈로그 요약

| 코드 | 종류 | 작성 위치 | 외부 의존 | 핵심 조건 |
|------|------|-----------|-----------|-----------|
| **T-T-01** | 단위 테스트 | `src/**/*.rs` 내 `#[cfg(test)]` | 없음 (Mock 사용) | 순수 로직만 |
| **T-T-02** | Repository DB 테스트 | `tests/db/` | PostgreSQL | 트랜잭션 롤백 |
| **T-T-03** | Usecase 통합 테스트 | `tests/integration/` | Mock DB | 비즈니스 흐름 |
| **T-T-04** | HTTP API 테스트 | `tests/api/` | axum TestClient | 엔드포인트 전체 |
| **T-T-05** | 프로퍼티 기반 테스트 | `src/**/*.rs` 내 `#[cfg(test)]` | 없음 | 불변 조건 |
| **T-T-06** | 공통 테스트 헬퍼 | `tests/common/` | 설정에 따라 | 재사용 픽스처 |

---

## T-T-01 — 단위 테스트 (Unit Test)

### 작성 위치
```
src/
├── {crate}/src/{module}.rs     ← 같은 파일 하단에 #[cfg(test)] 블록
```

### 대상
- 도메인 모델 생성자 / Newtype 변환
- `From` / `TryFrom` / `Into` 구현
- 순수 비즈니스 로직 함수
- 에러 분기 (Result 반환 함수)
- Repository Mock을 사용한 Usecase 로직

### 패턴
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    // Mock 정의 (async_trait 사용 시)
    mockall::mock! {
        UserRepositoryMock {}
        #[async_trait::async_trait]
        impl UserRepository for UserRepositoryMock {
            async fn find_by_username(&self, username: &str)
                -> anyhow::Result<Option<User>>;
        }
    }

    #[test]
    fn {대상}_{조건}_{기대결과}() {
        // Arrange
        // Act
        // Assert
    }

    #[tokio::test]
    async fn {대상}_{조건}_{기대결과}() {
        // 비동기 단위 테스트
    }
}
```

### 네이밍 규칙
```
형식: {테스트_대상}_{조건}_{기대_결과}

예시:
  user_new_stores_all_fields
  id_try_from_invalid_string_returns_error
  create_user_with_empty_username_returns_validation_error
  login_user_with_wrong_password_returns_unauthorized
```

### 필수 케이스
- 정상 케이스 (happy path)
- 에러 케이스 (모든 `Result::Err` 분기)
- 경계 케이스 (빈 문자열, None, 최대값 등)

---

## T-T-02 — Repository DB 테스트

### 작성 위치
```
tests/
└── db/
    ├── user_repository_test.rs
    └── todo_repository_test.rs
```

### 대상
- `infra/` 크레이트의 Repository 구현체
- 실제 SQL 쿼리 검증 (CRUD + 에지 케이스)
- Migration 후 스키마 정합성

### 환경 조건
- `TEST_DATABASE_URL` 환경변수 필수
- 각 테스트는 트랜잭션 시작 → 실행 → **반드시 롤백**
- `tests/common/db.rs` 의 `setup_test_db()` 헬퍼 사용

### 패턴
```rust
// tests/db/user_repository_test.rs
use crate::common::db::setup_test_db;

#[tokio::test]
async fn {대상}_{조건}_{기대결과}() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    // 테스트 실행
    let repo = PgUserRepository::new(&mut tx);
    // ... assertions ...

    tx.rollback().await.unwrap();  // 항상 롤백
}
```

### 필수 케이스
- insert → find 정합성
- 존재하지 않는 ID 조회 → None 반환
- 중복 insert → 에러 반환
- 조건 필터링 정확성

---

## T-T-03 — Usecase 통합 테스트

### 작성 위치
```
tests/
└── integration/
    ├── user_usecase_test.rs
    └── todo_usecase_test.rs
```

### 대상
- `usecase/` 크레이트의 Usecase 구현체
- 여러 Repository 협력을 통한 비즈니스 흐름
- Mock Repository를 통한 경계 조건

### 패턴
```rust
// tests/integration/user_usecase_test.rs
use mockall::predicate::*;

#[tokio::test]
async fn {대상}_{조건}_{기대결과}() {
    // 여러 Repository Mock 조합
    let mut user_repo_mock = MockUserRepository::new();
    user_repo_mock.expect_find_by_username()
        .returning(|_| Ok(None));

    let module = MockRepositoriesModule::new(user_repo_mock);
    let usecase = UserUseCase::new(Arc::new(module));

    let result = usecase.create_user(CreateUser::new(...)).await;
    assert!(result.is_ok(), "{result:?}");
}
```

### 필수 케이스
- 정상 흐름 (전체 비즈니스 시나리오)
- Repository 에러 전파
- 비즈니스 규칙 위반 케이스

---

## T-T-04 — HTTP API 테스트

### 작성 위치
```
tests/
└── api/
    ├── user_api_test.rs
    └── todo_api_test.rs
```

### 대상
- `controller/` 크레이트의 라우터/핸들러
- HTTP 상태 코드, 응답 바디, 헤더 검증
- 인증 미들웨어 동작 검증

### 환경 조건
- `axum::Router`를 직접 테스트 (실제 바인딩 불필요)
- `tower::ServiceExt` + `http::Request` 사용
- `TEST_DATABASE_URL` 또는 Mock 사용

### 패턴
```rust
// tests/api/user_api_test.rs
use axum::http::{Request, StatusCode};
use tower::ServiceExt;  // oneshot

#[tokio::test]
async fn {엔드포인트}_{조건}_{기대_HTTP상태}() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"username":"alice",...}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["username"], "alice");
}
```

### 필수 케이스
- 200/201 정상 응답
- 400 잘못된 요청 바디
- 401 인증 없음
- 404 리소스 없음
- 응답 바디 스키마 검증

---

## T-T-05 — 프로퍼티 기반 테스트

### 작성 위치
```
src/{crate}/src/{module}.rs 내 #[cfg(test)]
```

### 대상
- 복잡한 도메인 불변 조건
- 입력 범위가 넓은 변환 함수
- 직렬화/역직렬화 왕복 정합성

### 패턴
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn {대상}_with_arbitrary_input_{불변조건}(
        value in "[a-z]{1,50}",
        num in 0i64..10000,
    ) {
        // 불변 조건 검증
        prop_assert!(/* condition */);
    }
}
```

### 사용 조건
- `proptest` 크레이트가 `[dev-dependencies]`에 추가된 경우에만 사용
  (미추가 시 사전에 사용자에게 확인)

---

## T-T-06 — 공통 테스트 헬퍼

### 작성 위치
```
tests/
└── common/
    ├── mod.rs        ← re-export
    ├── db.rs         ← DB 연결 헬퍼
    ├── fixtures.rs   ← 테스트 픽스처 (User, Todo 팩토리)
    └── app.rs        ← axum 테스트 앱 생성
```

### 역할
- `setup_test_db()` — PgPool 생성 + Migration 적용
- `create_test_app()` — axum::Router 조립
- `fixture_user()`, `fixture_todo()` — 테스트 데이터 팩토리

### 패턴
```rust
// tests/common/db.rs
pub async fn setup_test_db() -> sqlx::PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL 환경변수가 필요합니다");
    let pool = sqlx::PgPool::connect(&url).await
        .expect("테스트 DB 연결 실패");
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Migration 실패");
    pool
}

// tests/common/fixtures.rs
pub fn fixture_user() -> NewUser {
    NewUser::new(
        Id::gen(),
        "test_user".to_string(),
        "hashed_password".to_string(),
        "Test User".to_string(),
    )
}
```

---

## 테스트 파일 위치 결정 기준

```
테스트 대상 코드                →  적용 카탈로그  →  파일 위치
────────────────────────────────────────────────────────────────
domain/src/model/*.rs           →  T-T-01         →  src 내부 #[cfg(test)]
domain/src/repository/*.rs      →  T-T-01         →  src 내부 #[cfg(test)]
infra/src/repository/*.rs       →  T-T-02         →  tests/db/
usecase/src/usecase/*.rs        →  T-T-01, T-T-03 →  src 내부 + tests/integration/
controller/src/routes/*.rs      →  T-T-04         →  tests/api/
공통 헬퍼·픽스처                →  T-T-06         →  tests/common/
복잡한 도메인 로직               →  T-T-05         →  src 내부 #[cfg(test)]
```
