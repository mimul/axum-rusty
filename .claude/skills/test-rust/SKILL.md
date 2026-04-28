---
name: test-rust
description: >
  /test-rust 커맨드로 실행되는 Rust 테스트 작성 스킬.
  feature/test-{module} 브랜치를 자동 생성하고, 단위 테스트는
  src 내부 #[cfg(test)]에, 통합/DB/HTTP API 테스트는 {crate}/tests/ 하위에
  분리하여 작성한다. T-T-01~T-T-06 카탈로그 기준으로 분류하고,
  항목별 테스트 코드를 제시한 뒤 인간 확인 후에만 작성한다.
  rules/rust-test-style.md를 테스트 철학·규칙의 권위 문서로 사용한다.
---

# `/test-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/test-rust` 커맨드가 입력될 때 자동으로 실행**된다.
아래 테스트 카탈로그(T-T-01~T-T-06)를 기준으로 테스트 작성 계획을
수립한 뒤, **항목별로 테스트 코드를 먼저 제시하고 인간의 확인을
받은 뒤에만 파일에 작성한다.**

**테스트 철학 (`rules/rust-test-style.md` 준수)**:
- 구현이 아닌 **동작**을 테스트한다
- **시스템 경계**(외부 HTTP API, 파일시스템 등)에서만 Mock을 사용한다
- DB/ORM·내부 Repository는 **절대 mock하지 않는다** — testcontainers로 실제 DB를 사용한다
- Classicist 접근: 단위 테스트보다 통합 테스트를 선호한다

**핵심 운영 규칙**:
- **브랜치 자동 생성** — `feature/test-{module}` 브랜치에서만 작업
- **위치 분리** — 단위 테스트는 `src/` 내부, 나머지는 `{crate}/tests/` 하위
- **항상 그린** — 매 파일 작성 후 `cargo test` 통과 확인
- **보여주고 확인받기** — 코드를 먼저 제시, 인간 승인 후에만 파일 저장

---

## 테스트 카탈로그 (T-T-01 ~ T-T-06)

테스트 철학·Mocking 규칙·네이밍·PR 기준은 `rules/rust-test-style.md`(이하 **§섹션** 표기)가 권위 문서다.
이 카탈로그는 그 철학을 **이 프로젝트의 Rust 구현 패턴**으로 구체화한다.

> **선택 원칙 (§1.4 통합 테스트 우선)**
> 단위 테스트로 충분한지 먼저 묻는다. 협력 객체가 있거나 DB가 필요하면 T-T-02/T-T-03을 선택한다.
> "이 테스트가 보호하는 동작을 한 문장으로 설명할 수 없으면, 작성하지 말 것" (§12)

### 카탈로그 요약

| 코드 | 종류 | 적용 결정 기준 | 위치 | 외부 의존 |
|------|------|----------------|------|-----------|
| **T-T-01** | 단위 테스트 | 도메인 로직이 DB·외부 I/O 없이 테스트 가능할 때 | `src/**/*.rs` `#[cfg(test)]` | 없음 |
| **T-T-02** | Repository DB 테스트 | SQL 쿼리·스키마 정합성을 직접 검증해야 할 때 | `{crate}/tests/` | testcontainers |
| **T-T-03** | Usecase 통합 테스트 | 여러 Repository 협력과 트랜잭션 경계를 검증할 때 | `{crate}/tests/` | testcontainers |
| **T-T-04** | HTTP API 테스트 | 인증·라우팅·직렬화·미들웨어를 포함한 전체 흐름이 필요할 때 | `controller/tests/` | testcontainers |
| **T-T-05** | 프로퍼티 기반 테스트 | 같은 함수에 네 번째 예제 테스트를 작성하려는 순간 | `src/**/*.rs` `#[cfg(test)]` | 없음 |
| **T-T-06** | 공통 테스트 헬퍼 | T-T-02/03/04 진입 전 공유 인프라 준비 | `{crate}/tests/common/` | testcontainers |

> **DB 테스트 원칙**: `TEST_DATABASE_URL` 환경변수 없이 testcontainers가 Docker를 자동 기동한다.
> FIDT의 I(Isolated)는 각 테스트의 트랜잭션 롤백으로 보장한다 (§1.5).

### 테스트 파일 위치 결정 기준

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

> **Rust 통합 테스트 주의**: `tests/` 바로 아래 `.rs` 파일만 별도 바이너리로 인식한다.
> 서브디렉토리(`tests/db/foo.rs`)는 `cargo test --test`로 직접 지정할 수 없다.

---

### T-T-01 — 단위 테스트 (Unit Test)

**§ 근거**: §1.4(통합 테스트 우선), §6(테스트 피라미드), §8(도메인 엔티티 추출)

**선택 기준**: 도메인 로직이 순수 함수로 격리되어 DB·Clock·외부 서비스 없이 호출 가능한 경우.
서비스에 도메인 로직이 뒤섞여 있다면 §8 추출 기준을 먼저 확인한다.
단위 테스트는 추출된 도메인 객체가 전제다.

**대상**
- 도메인 모델 생성자·Newtype 변환·불변 조건
- `From` / `TryFrom` / 순수 비즈니스 연산
- 상태 전환·에러 분기 로직

**작성 위치**
```
src/
└── {crate}/src/{module}.rs   ← 같은 파일 하단 #[cfg(test)] 블록
```

**패턴 — AAA + 네이밍 §3.2 준수**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 정상 케이스: 유효한 입력 → 성공
    #[test]
    fn create_user_id_succeeds_when_uuid_is_valid() {
        // Arrange
        let raw = "550e8400-e29b-41d4-a716-446655440000";

        // Act
        let result = UserId::parse(raw);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), raw);
    }

    // 에러 케이스: 잘못된 형식 → 에러 타입 검증
    #[test]
    fn create_user_id_fails_when_format_is_invalid() {
        // Arrange & Act
        let err = UserId::parse("not-a-uuid").unwrap_err();

        // Assert: 에러 타입 검증 (내부 구현 문자열이 아닌 도메인 에러 타입으로)
        assert!(matches!(err, DomainError::InvalidId(_)));
    }

    // 경계 케이스: 빈 문자열
    #[test]
    fn create_username_fails_when_empty() {
        // Arrange & Act
        let result = Username::new("");

        // Assert
        assert!(matches!(result, Err(DomainError::EmptyUsername)));
    }

    // 상태 전환: 할인 적용 후 금액이 정확히 감소
    #[test]
    fn apply_discount_reduces_total_when_rate_is_valid() {
        // Arrange
        let mut order = Order::new(Money::new(10_000));

        // Act
        order.apply_discount(0.1).unwrap();

        // Assert: 반환값이 아닌 상태 변화를 검증 (§5.2)
        assert_eq!(order.total(), Money::new(9_000));
    }

    // 에러 케이스: 경계를 벗어난 rate → 에러
    #[test]
    fn apply_discount_fails_when_rate_exceeds_one() {
        // Arrange
        let mut order = Order::new(Money::new(10_000));

        // Act
        let err = order.apply_discount(1.5).unwrap_err();

        // Assert
        assert!(matches!(err, DomainError::InvalidDiscountRate(_)));
    }
}
```

**작성 기준 (§6.2 생략 가능 / §6.3 반드시)**
- 작성하지 않는다: getter, DI 연결, 단순 CRUD 위임, 프레임워크 초기화
- 반드시 작성한다: 비즈니스 규칙, 상태 전환, 에러 분기, 과거 버그 경로

**금지 (§1.5 FIDT, §10 Flaky)**
```rust
// ❌ 비결정적 — 실행마다 결과가 달라지므로 Deterministic 위반
assert_ne!(hash_password("pw").await, hash_password("pw").await);

// ❌ 의미 없는 Assertion (§13.1 ⑨) — 존재 여부만 확인, 내용 미검증
assert!(result.is_ok());  // 반환값이 있다면 내용까지 검증할 것

// ❌ 내부 구현 문자열 검증 — 리팩토링 시 깨짐
assert!(err.to_string().contains("parse_str"));
```

---

### T-T-02 — Repository DB 테스트

**§ 근거**: §1.2(시스템 경계 모킹), §4.3(DB mock 금지), §5.2(상태 검증), §7(비동기 테스트)

**선택 기준**: `infra/` Repository 구현체의 SQL 쿼리·INSERT/SELECT 정합성·스키마 제약을 검증한다.
Usecase 비즈니스 흐름이 포함되면 T-T-03을 선택한다.

**작성 위치**
```
{crate}/tests/
├── common/
│   ├── mod.rs
│   └── container.rs    ← postgres_url()
└── {entity}_repository_test.rs
```

**패턴 — 트랜잭션 격리 + 상태 검증**
```rust
// {crate}/tests/user_repository_test.rs
mod common;

use common::{container::postgres_url, fixtures::fixture_new_user};
use sqlx::PgPool;

async fn setup() -> PgPool {
    PgPool::connect(&postgres_url()).await.expect("DB 연결 실패")
}

// 정상 케이스: insert 후 find로 상태 검증
#[tokio::test]
async fn insert_user_succeeds_when_input_is_valid() {
    // Arrange
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = PgUserRepository::new(&mut tx);
    let user = fixture_new_user();

    // Act
    let created = repo.insert(&user).await.unwrap();

    // Assert: 반환값 + DB 실제 상태 모두 검증 (§5.2)
    assert_eq!(created.username, user.username);
    let found = repo.find_by_id(&created.id).await.unwrap();
    assert_eq!(found.unwrap().username, user.username); // is_some()이 아닌 실제 값 검증

    tx.rollback().await.unwrap(); // 항상 롤백 — FIDT Isolated 보장 (§1.5)
}

// 에러 케이스: 없는 ID 조회
#[tokio::test]
async fn find_user_returns_none_when_id_does_not_exist() {
    // Arrange
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = PgUserRepository::new(&mut tx);

    // Act
    let result = repo.find_by_id(&UserId::new()).await.unwrap();

    // Assert
    assert!(result.is_none());
    tx.rollback().await.unwrap();
}

// 에러 케이스: DB 제약 위반
#[tokio::test]
async fn insert_user_fails_when_username_is_duplicate() {
    // Arrange
    let pool = setup().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = PgUserRepository::new(&mut tx);
    let user = fixture_new_user();
    repo.insert(&user).await.unwrap();

    // Act: 중복 삽입
    let result = repo.insert(&user).await;

    // Assert: Err 타입까지 검증
    assert!(matches!(result.unwrap_err(), InfraError::UniqueViolation(_)));
    tx.rollback().await.unwrap();
}
```

**필수 케이스**
- insert → find 정합성 (반환값과 DB 상태 모두)
- 존재하지 않는 ID 조회 → `None`
- DB 제약 위반 (unique, not-null 등) → `Err` 타입 검증
- 필터 쿼리 정확성 (조건에 따른 결과 집합)

**금지 (§4.3)**
```rust
// ❌ Repository를 mock으로 대체
let mut mock_repo = MockUserRepository::new();
mock_repo.expect_insert().returning(|_| Ok(fixture_new_user()));

// ❌ 롤백 누락 — 다음 테스트 오염
let mut tx = pool.begin().await.unwrap();
repo.insert(&user).await.unwrap();
// tx.rollback() 없이 종료 → 공유 컨테이너 오염
```

---

### T-T-03 — Usecase 통합 테스트

**§ 근거**: §1.2(시스템 경계), §1.3(Classicist TDD), §4.3(내부 mock 금지), §5.2(상태 검증)

**선택 기준**: 여러 Repository가 협력하는 비즈니스 흐름, 트랜잭션 롤백 동작, 도메인 이벤트 발생을 검증한다.
단일 SQL 검증이면 T-T-02, HTTP 계층까지 필요하면 T-T-04를 선택한다.

**Mock 금지 이유**: Usecase가 Repository mock을 주입받으면 리팩토링마다 `expect_*` 호출이 깨진다.
실제 DB + 실제 Repository로 최종 상태를 검증하는 것이 Classicist 원칙이다 (§1.3).

**작성 위치**
```
{crate}/tests/
├── common/
│   ├── mod.rs
│   └── container.rs    ← postgres_url()
└── {usecase}_integration_test.rs
```

**패턴 — 정상 흐름 (AAA + 부작용 명시 검증)**
```rust
// {crate}/tests/create_user_integration_test.rs
mod common;

use common::container::postgres_url;
use sqlx::PgPool;
use std::sync::Arc;

async fn setup() -> PgPool {
    PgPool::connect(&postgres_url()).await.expect("DB 연결 실패")
}

#[tokio::test]
async fn create_user_succeeds_when_input_is_valid() {
    // Arrange
    let pool = setup().await;
    let usecase = UserUseCase::new(Db(Arc::new(pool.clone())));
    let input = CreateUserInput {
        username: "alice".to_string(),
        password: "secret123".to_string(),
    };

    // Act
    let result = usecase.create_user(input).await;

    // Assert: 반환값 + 부작용(DB 쓰기) 명시적 검증 (§5.2)
    assert!(result.is_ok(), "예상치 못한 에러: {result:?}");
    let created = result.unwrap();
    assert_eq!(created.username, "alice");

    // DB에 실제로 저장됐는지 독립 쿼리로 확인
    let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
        .bind(created.id.as_ref())
        .fetch_one(&pool)
        .await
        .expect("생성된 user가 DB에 없음");
    assert_eq!(row.username, "alice");
}
```

**패턴 — 트랜잭션 롤백 검증 (실패 시 DB 무변경)**
```rust
#[tokio::test]
async fn create_user_fails_when_username_is_duplicate() {
    // Arrange: 선행 데이터 삽입 (커밋됨)
    let pool = setup().await;
    let usecase = UserUseCase::new(Db(Arc::new(pool.clone())));
    let input = CreateUserInput { username: "alice".to_string(), password: "pw".to_string() };
    usecase.create_user(input.clone()).await.unwrap();

    // Act: 중복 → 실패 → Usecase 내부 트랜잭션 롤백
    let result = usecase.create_user(input).await;

    // Assert: 에러 반환 + DB 상태가 1건으로 유지됨
    assert!(result.is_err());
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = 'alice'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "롤백 후 중복 row가 없어야 한다");
}
```

**필수 케이스**
- 정상 흐름: 반환값 + DB 부작용 모두 검증
- 비즈니스 규칙 위반 → 에러 타입
- 트랜잭션 롤백: 실패 시 DB 상태 무변경 검증
- 복수 Repository 협력이 있다면 각 Repository 상태 검증

**금지 (§4.3)**
```rust
// ❌ Repository mock — Classicist 원칙 위반
let mut mock_repo = MockUserRepository::new();
mock_repo.expect_save().times(1).returning(|_| Ok(()));
let usecase = UserUseCase::new(mock_repo);

// ❌ assert!(result.is_ok()) 단독 — 부작용 미검증 (§13.1 ⑨)
assert!(result.is_ok()); // DB에 저장됐는지 모름
```

---

### T-T-04 — HTTP API 테스트

**§ 근거**: §6.3(인증·권한 반드시 테스트), §5.2(상태 검증), §14(Rust 관례)

**선택 기준**: 인증 미들웨어·라우팅·JSON 직렬화·에러 포맷을 포함한 HTTP 전체 흐름을 검증한다.
비즈니스 로직만 검증하면 T-T-03이 적합하다.

**반드시 포함해야 할 케이스 (§6.3)**
- 인증 없음 → 401
- 권한 없음 → 403
- 유효하지 않은 입력 → 400 + 에러 바디 스키마
- 정상 → 201/200 + 응답 바디 스키마

**작성 위치**
```
controller/tests/
├── common/
│   └── mod.rs    ← build_test_app()
└── {endpoint}_api_test.rs
```

**패턴 — AAA + 보안·스키마 검증**
```rust
// controller/tests/user_api_test.rs
mod common;

use axum::http::{Request, StatusCode};
use common::build_test_app;
use http_body_util::BodyExt;
use tower::ServiceExt;

// 정상 케이스: 응답 바디 스키마 + 보안 필드 미노출
#[tokio::test]
async fn create_user_returns_201_when_input_is_valid() {
    // Arrange
    let app = build_test_app().await;
    let body = r#"{"username":"alice","password":"secret123","name":"Alice"}"#;

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: 상태 코드 + 응답 바디 스키마 + 보안 필드 미노출 (§6.3)
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["username"], "alice");
    assert!(json.get("password_hash").is_none(), "비밀번호 해시가 응답에 노출되면 안 된다");
    assert!(json.get("password").is_none(), "평문 비밀번호가 응답에 노출되면 안 된다");
}

// 인증 케이스: 반드시 테스트 (§6.3)
#[tokio::test]
async fn get_me_returns_401_when_not_authenticated() {
    // Arrange
    let app = build_test_app().await;

    // Act
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

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// 유효성 검사 케이스: 에러 응답 내부 정보 미노출
#[tokio::test]
async fn create_user_returns_400_when_username_is_empty() {
    // Arrange
    let app = build_test_app().await;

    // Act
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

    // Assert: 400 + 에러 메시지에 스택 트레이스·내부 경로 미포함
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("error").is_some(), "에러 메시지 필드가 있어야 한다");
    let error_msg = json["error"].as_str().unwrap_or("");
    assert!(!error_msg.contains("src/"), "파일 경로가 에러 메시지에 노출되면 안 된다");
}
```

**금지 (§4.3, §13)**
```rust
// ❌ 상호작용 검증만 있고 상태 검증 없음 (§13.1 ①)
mock_service.expect_create_user().times(1);
// HTTP 응답 바디나 DB 상태를 검증하지 않음

// ❌ 인증·권한 테스트 생략 (§6.3)
// create_user 정상 케이스만 작성하고 401·403 생략
```

---

### T-T-05 — 프로퍼티 기반 테스트

**§ 근거**: §9(Property-Based Testing)

**선택 기준 (§9.1 전환 트리거)**
동일한 함수에 대해 네 번째 예제 테스트를 작성하려는 순간 proptest로 전환을 검토한다.
아래에 해당하면 proptest가 적합하다:
- 검증기·파서·직렬화 왕복(round-trip)
- 대수적 성질: 멱등성, 가환성, 결합성
- 도메인 불변 조건: "어떤 입력에도 총금액은 0 이상이어야 한다"

**작성 위치**: `src/{crate}/src/{module}.rs` 내 `#[cfg(test)]`

**패턴 — §3.2 네이밍 + §5.1 AAA**
```rust
use proptest::prelude::*;

proptest! {
    // 도메인 규칙: 모든 유효 길이 이름은 파싱 성공
    #[test]
    fn create_username_succeeds_when_length_is_within_limit(
        name in "[a-z][a-z0-9_]{1,29}",  // 2~30자 유효 범위
    ) {
        prop_assert!(Username::new(&name).is_ok());
    }

    // 경계 조건: 길이 초과는 항상 실패
    #[test]
    fn create_username_fails_when_length_exceeds_limit(
        name in ".{31,200}",
    ) {
        prop_assert!(Username::new(&name).is_err());
    }

    // 불변 조건: 할인 후 금액은 항상 0 이상
    #[test]
    fn apply_discount_never_makes_total_negative_when_rate_is_valid(
        amount in 1_000i64..100_000_000,
        rate in 0.0f64..=1.0,
    ) {
        let mut order = Order::new(Money::new(amount));
        let _ = order.apply_discount(rate); // 에러가 나도 불변 조건은 유지
        prop_assert!(order.total().amount() >= 0, "총금액이 음수: {}", order.total().amount());
    }

    // 직렬화 왕복: 변환 전후 값 동일
    #[test]
    fn serialize_deserialize_roundtrip_preserves_username_when_valid(
        name in "[a-z][a-z0-9_]{1,29}",
    ) {
        let original = Username::new(&name).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Username = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(original, restored);
    }
}
```

**금지 (§9.3, §10)**
```rust
// ❌ 네트워크·DB 포함 — 속도·결정성 보장 불가
proptest! {
    #[test]
    fn create_user_succeeds(username in "[a-z]{3,20}") {
        pool.execute(...); // proptest 내 I/O 금지
    }
}

// ❌ when_ 절 없는 네이밍 (§3.2)
fn serialize_deserialize_roundtrip(/* ... */) { }  // 조건 불명확
```

---

### T-T-06 — 공통 테스트 헬퍼

**§ 근거**: §1.5(FIDT), §11(픽스처와 빌더), §14.2(헬퍼 크레이트)

**역할**: T-T-02/03/04가 공유하는 인프라(컨테이너, 픽스처, 앱 팩토리)를 중앙화한다.
반복되는 Arrange 코드가 Assert보다 10배 길어지면 Builder/Fixture 도입 신호다 (§13.2).

**FIDT 격리 설계 (§1.5)**
- `postgres_url()`은 `OnceLock`으로 **바이너리당 1회** 컨테이너를 기동한다 (Fast)
- 각 테스트는 트랜잭션을 시작하고 종료 시 반드시 `rollback()`한다 (Isolated)
- `fixture_new_user()`는 `uuid::Uuid::new_v4()`로 충돌 없는 데이터를 생성한다 (Deterministic)
- `#[ctor::dtor]`로 프로세스 종료 시 컨테이너를 정리한다 (Mac Docker Desktop Ryuk 미동작 대응)

**작성 위치**
```
{crate}/tests/
└── common/
    ├── mod.rs          ← pub mod container; pub mod fixtures;
    ├── container.rs    ← postgres_url()
    └── fixtures.rs     ← fixture_new_user() 등

controller/tests/common/mod.rs  ← build_test_app() 추가
```

**container.rs (testcontainers 0.23)**
```rust
use sqlx::PgPool;
use std::sync::{Mutex, OnceLock};
use testcontainers::{core::ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

static POSTGRES_URL: OnceLock<String> = OnceLock::new();
static CONTAINER_IDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Mac Docker Desktop은 Ryuk이 비활성화되어 있으므로 직접 정리한다
#[ctor::dtor]
fn cleanup_test_containers() {
    let ids = CONTAINER_IDS.lock().unwrap_or_else(|e| e.into_inner());
    for id in ids.iter() {
        let _ = std::process::Command::new("docker").args(["rm", "-f", id]).output();
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

/// 바이너리당 1회 PostgreSQL 컨테이너를 기동하고 URL을 반환한다.
/// 컨테이너는 `#[ctor::dtor]`에서 정리된다.
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

**fixtures.rs — Builder 패턴 적용 (§11.1)**
```rust
use domain::model::user::{NewUser, UserId};

/// 기본 픽스처 — 필요한 필드만 오버라이드
pub struct NewUserBuilder {
    username: String,
    password_hash: String,
    name: String,
}

impl NewUserBuilder {
    pub fn new() -> Self {
        Self {
            username: format!("user_{}", uuid::Uuid::new_v4().simple()),
            password_hash: "$2b$12$fixedhashfortest".to_string(), // 실제 시크릿 금지 (rust-security-style.md §7 시크릿 관리)
            name: "Test User".to_string(),
        }
    }

    pub fn username(mut self, name: &str) -> Self {
        self.username = name.to_string();
        self
    }

    pub fn build(self) -> NewUser {
        NewUser {
            id: UserId::new(),
            username: self.username,
            password_hash: self.password_hash,
            name: self.name,
        }
    }
}

/// 단순 호출용 축약 함수
pub fn fixture_new_user() -> NewUser {
    NewUserBuilder::new().build()
}
```

**Cargo.toml dev-dependencies 템플릿**
```toml
[dev-dependencies]
tokio                  = { version = "1", features = ["full"] }
testcontainers         = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
ctor                   = "0.2"
# API 테스트 전용 (controller 크레이트)
tower          = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
# 선택: Assertion 가독성 향상 (§14.4)
pretty_assertions = "1"
```

**금지 (§1.5, §13)**
```rust
// ❌ 픽스처에 실제 시크릿 하드코딩 (rust-security-style.md §7 시크릿 관리)
password_hash: "실제_bcrypt_해시_또는_토큰".to_string(),

// ❌ 롤백 없는 공유 DB 사용 — 테스트 간 상태 오염 (§1.5 Isolated)
// tx.rollback() 누락

// ❌ 시스템 시계 직접 사용 — Deterministic 위반 (§10.2)
let now = SystemTime::now();
```

---

## 커맨드 문법

```
/test-rust                         전체 코드베이스 커버리지 분석 → 계획 수립
/test-rust [파일명 또는 모듈명]    특정 대상만 분석·작성 (Claude가 Read 도구로 파일 직접 읽음)
/test-rust --type unit             단위 테스트(T-T-01)만 작성
/test-rust --type db               Repository DB 테스트(T-T-02)만 작성
/test-rust --type integration      Usecase 통합 테스트(T-T-03)만 작성
/test-rust --type api              HTTP API 테스트(T-T-04)만 작성
/test-rust --type property         프로퍼티 기반 테스트(T-T-05)만 작성
/test-rust --catalog               카탈로그 전체 항목 출력
/test-rust --help                  사용법 출력
```

### `--help` 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📖  /test-rust 사용법
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

용도:
  Rust 코드의 테스트 갭을 분석하고 T-T-01~T-T-06 카탈로그 기준으로
  테스트 코드를 제시·작성합니다. 인간 확인 후에만 파일에 저장합니다.

기본 사용:
  /test-rust                    전체 코드베이스 갭 분석 → 계획 수립
  /test-rust src/domain/user.rs 특정 파일만 분석 (자동으로 파일 읽기)

타입 옵션 (특정 카탈로그 항목만 작성):
  --type unit         T-T-01 단위 테스트 (src 내부 #[cfg(test)])
  --type db           T-T-02 Repository DB 테스트 (testcontainers)
  --type integration  T-T-03 Usecase 통합 테스트 (testcontainers)
  --type api          T-T-04 HTTP API 테스트 (axum TestClient)
  --type property     T-T-05 프로퍼티 기반 테스트 (proptest)

  ※ --type db, integration, api 는 T-T-06 공통 헬퍼를 자동 포함
  ※ --type unit, property 는 T-T-06 포함하지 않음 (Docker 불필요)

기타 옵션:
  --catalog           카탈로그 전체 목록 출력
  --help              이 도움말 출력

실행 흐름:
  STEP 0   Git 브랜치 준비 (자동)
  STEP 1   코드 분석 및 커버리지 갭 탐지
  STEP 2   rules 로드 + 갭 분석 리포트 출력
  STEP 3   테스트 작성 계획 → 사용자 승인 대기
  STEP 4   항목별 테스트 코드 제시 → 사용자 확인 후 저장 (반복)
  STEP 5-0 커버리지 게이트 (80% 미만 시 PR 차단)
  STEP 5   완료 요약
  STEP 6   PR 초안 제시 → 사용자 승인 후 push + PR 생성

불변 규칙:
  - 파일 저장은 사용자 승인 후에만 수행
  - cargo test 실패 상태로 커밋하지 않음
  - 커버리지 80% 미만이면 PR 생성 금지
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 실행 흐름

```
STEP 0  →  STEP 1  →  STEP 2  →  STEP 3  →  STEP 4  (반복 루프) ─┐
브랜치     코드 분석  rules 로드  작성 계획  테스트 코드 제시       │
자동생성   커버리지   + 갭 탐지   승인 대기  ↓ [승인]              │
(Bash)    측정                            파일 저장 + cargo test   │
                                          ↓ [다음 항목]        ←──┘
                                        STEP 5-0
                                        (커버리지 게이트)
                                          ↓ [80% 이상]
                                        STEP 5
                                        완료 요약
                                          ↓
                                        STEP 6
                                        PR 초안 → 승인 → push + PR 생성
```

**인간의 응답을 기다리는 단계: STEP 3(계획 승인), STEP 4(항목별 저장 확인), STEP 6(PR 생성 확인)**
**STEP 0는 Claude가 Bash 도구로 자동 실행한다.**

---

## STEP 0 — Git 브랜치 자동 생성

테스트 작성 전 가장 먼저 수행한다.
**Claude가 단일 Bash 호출로 직접 실행하며, 사용자 확인 없이 자동으로 진행한다.**
(**Bash 도구는 호출마다 독립 셸이므로 모든 커맨드를 하나의 블록으로 실행한다.**)

**파일/모듈명을 인수로 지정한 경우**: 0-2 bash script 선두에서 파일 존재를 확인한다.
파일이 없으면 git 작업 없이 즉시 STEP 1 오류 출력 형식으로 종료한다.

### 0-1. 브랜치 이름 결정

```
브랜치 네이밍 규칙: feature/test-{module-name}

module-name 결정 기준 (우선순위):
  1. 사용자가 명시한 파일명/모듈명 (확장자·경로 제거)
  2. --type 옵션 키워드 (unit, db, integration, api, property)
  3. 코드에서 파악한 최상위 크레이트/모듈명
  4. 기능 키워드 (user, todo, auth 등)
  5. 인수와 --type 옵션이 모두 없는 경우: whole-codebase 고정

예시:
  /test-rust usecase/user.rs        → feature/test-user-usecase
  /test-rust --type db              → feature/test-db
  /test-rust --type api             → feature/test-api
  /test-rust (전체)                 → feature/test-whole-codebase
```

### 0-2. 브랜치 자동 실행

```bash
set -e
BRANCH="feature/test-[module-name]"

# 0. 파일 지정 시 존재 여부 확인 (git 작업 전에 검사)
# TARGET_FILE=[지정된 경로] — 인수 없을 때는 이 블록을 실행하지 않는다
if [ -n "$TARGET_FILE" ] && [ ! -e "$TARGET_FILE" ]; then
  echo "FILE_NOT_FOUND:$TARGET_FILE"
  exit 1
fi

# 1. main 최신화
git checkout main && git pull origin main

# 2. 브랜치 생성 (케이스별 분기)
if git branch --list "$BRANCH" | grep -q "^[[:space:]]*$BRANCH$"; then
  git checkout "$BRANCH"
else
  git checkout -b "$BRANCH"
fi

# 3. 기준선 측정
cargo test --all 2>&1 | tail -5
```

실행 결과를 아래 형식으로 보고하고 즉시 STEP 1로 진행한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  브랜치 준비 완료 (자동 실행됨)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
브랜치: feature/test-[module-name]
✅ cargo test — 기준선 N 통과 / 0 실패
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 1 — 코드 분석 및 커버리지 갭 탐지

**파일/모듈명을 인수로 지정한 경우** — 먼저 존재 여부를 확인한다:

```bash
# 지정된 경로가 절대/상대 경로인 경우 그대로 사용.
# 파일명만 지정된 경우 프로젝트 루트(워킹 디렉토리) 기준으로 확인.
ls [지정된 경로]
```

파일이 없으면 후보 목록을 출력하고 **즉시 종료**한다:

```bash
find . -name "*.rs" | grep -v "mod.rs" | grep -v "target/" | head -10
```

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  파일을 찾을 수 없음: [지정 경로]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
분析 가능한 Rust 파일 목록 (상위 10개):
  [위 find 명령 실행 결과]

다시 실행:
  /test-rust [올바른 파일 경로]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

파일이 존재하면 Claude가 `Read` 도구로 해당 파일을 직접 읽는다.
사용자에게 코드를 붙여넣으라고 요청하지 않는다.

대상 코드를 읽고 아래 항목을 파악한다:

- 각 크레이트(`common`, `domain`, `infra`, `usecase`, `controller`)의 파일 목록
- 이미 작성된 `#[cfg(test)]` 모듈 및 `tests/` 파일 현황
- 테스트가 없는 `pub fn` / `pub struct` / `impl` 블록
- `Result` 반환 함수 중 에러 케이스 테스트가 없는 항목
- `tests/common/` 헬퍼 존재 여부

---

## STEP 2 — rules 로드 및 테스트 갭 분석 리포트

**분석 시작 전 아래 파일들을 반드시 로드하고, 각 규칙을 분석에 직접 적용한다.**

로드 순서:
1. `../../rules/rust-test-style.md` — 테스트 철학·Mocking·Naming·PR 기준 (권위 문서)
2. `../../rules/rust-security-style.md` — 보안 규칙 §1~§12 (테스트 작성 시 보안 검증 기준)

**`--type` 옵션이 지정된 경우**: 해당 타입에 매핑된 카탈로그 항목만 탐지한다.

### rust-test-style.md 적용 항목

| rust-test-style.md 섹션 | 갭 분석 적용 |
|---|---|
| §1. 테스트 철학 | Classicist 접근, 통합 테스트 우선 여부 확인 |
| §4. 모킹 경계 | 내부 mock(mockall 등) 사용 여부 → 발견 시 보고 |
| §5. Assertion 스타일 | 상호작용 검증만 있는 테스트 → 상태 검증으로 교체 권고 |
| §3. 테스트 네이밍 | 기존 테스트명 검증, 위반 시 보고 |
| §6. 테스트 피라미드 | 레이어별 테스트 예산 준수 여부, 단순 CRUD 생략 기준 |
| §13. PR 거절 신호 (Red Flags) | mockall 내부 사용, 스냅샷 비결정값, #[ignore] 무단 사용 등 |
| §9. Property-Based Testing (proptest) | 4번째 예제 테스트 → proptest 전환 권고 |

### rust-security-style.md 적용 항목 (§1~§12 우선순위 기반)

🔴 **Critical** — 테스트 픽스처에서도 즉시 수정:

| 검사 항목 | 근거 |
|-----------|------|
| 테스트 픽스처·상수에 실제 JWT 시크릿·API 키·비밀번호 하드코딩 없음 | §7 시크릿 관리 |
| Newtype 생성자에 잘못된 입력 거부 케이스 포함 여부 (Smart Constructor 불변식) | §3.1 입력 검증 |
| SQL 파라미터 바인딩 사용 검증 케이스 포함 여부 (포맷 조합 금지) | §3.3 SQL 인젝션 방지 |

🟠 **High** — 테스트 갭으로 보고:

| 검사 항목 | 근거 |
|-----------|------|
| 에러 응답이 내부 정보(스택 트레이스·DB 에러)를 노출하지 않는지 검증 케이스 포함 여부 | §5.1 정보 노출 방지 |
| 소유권·권한 검증 로직에 타인 리소스 접근 거부 케이스 포함 여부 (BOLA 방지) | §3.2 객체 레벨 권한 |
| `#[serde(deny_unknown_fields)]` 적용 타입에 알 수 없는 필드 거부 케이스 포함 여부 | §3.4 역직렬화 보안 |
| 패스워드 검증 로직에 Argon2id 해싱 결과 검증 케이스 포함 여부 | §4.3 패스워드 해싱 |

🟡 **Medium** — 가능하면 추가:

| 검사 항목 | 근거 |
|-----------|------|
| 비밀값 비교 함수에 타이밍 어택 방지(상수 시간) 케이스 포함 여부 | §4.2 상수 시간 비교 |
| 보안 이벤트(로그인 성공·실패, 권한 거부) 감사 로그 검증 케이스 포함 여부 | §9 감사 로그 |

### 갭 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /test-rust 갭 분석 리포트
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 현황 요약
   기존 테스트: [N]개
   테스트 없는 pub fn: [N]개
   에러 케이스 누락: [N]개
   내부 mock 사용 (개선 대상): [N]건

🚨 테스트 갭 ([N]건)

  [T-T-01 단위 테스트 누락]
  • [크레이트/파일명] — [fn명]
    이유: [정상/에러/경계 케이스 중 누락 항목]

  [T-T-02 DB 테스트 누락]
  • infra/repository/[파일명] — [fn명]
    이유: [어떤 SQL 경로가 미검증인지]

  [T-T-03 통합 테스트 누락]
  • usecase/[파일명] — [fn명]
    이유: [어떤 비즈니스 흐름이 미검증인지]

  [T-T-04 HTTP API 테스트 누락]
  • controller/routes/[파일명] — [경로]
    이유: [어떤 엔드포인트가 미검증인지]

  [T-T-06 공통 헬퍼 미비]
  • tests/common/ 없음 또는 일부 누락

⚠️ 개선 권고 (rules/rust-test-style.md 위반)
  • [파일:행] 내부 mock 사용 → 실제 DB 테스트로 전환 권고
  • [파일:행] 테스트명 Naming Rules 위반

✅ 이미 테스트된 항목
  • [크레이트/파일명] — 충분한 커버리지

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 테스트 작성 계획 수립 및 승인

**갭이 0건인 경우** — 아래 형식으로 출력하고 STEP 5-0으로 이동한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  테스트 갭 없음 — 추가 작성 불필요
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
분석 결과 누락된 테스트가 없습니다.
커버리지 게이트(STEP 5-0)를 확인합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**갭이 있는 경우** — 우선순위 기반 작성 계획을 제안하고 승인을 받는다.

**`--type` 옵션이 지정된 경우** — 해당 카탈로그 그룹만 계획 표에 표시하고,
나머지 그룹은 `⏭️ 보류 (--type 필터)`로 한 줄 표기한다.
단, `--type db`, `--type integration`, `--type api`는 T-T-06(공통 헬퍼)에 의존하므로
T-T-06을 자동으로 포함하고 "(--type 필터에 의해 자동 포함)" 표기한다.
`--type unit`, `--type property`는 T-T-06을 포함하지 않는다.

예시: `/test-rust --type db` 실행 시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  테스트 작성 계획  (필터: --type db)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
총 [N]개 테스트 파일 · [M]개 테스트 함수 예정

⏭️ 보류 (--type 필터): T-T-01 단위 테스트, T-T-03~05

┌─ T-T-06 공통 헬퍼 (--type 필터에 의해 자동 포함) ──┐
│ tests/common/container.rs — postgres_url()          │
│ tests/common/fixtures.rs  — fixture_new_user() 등   │
└────────────────────────────────────────────────────┘

┌─ T-T-02 Repository DB 테스트 ──────────────────────┐
│ {crate}/tests/{entity}_repository_test.rs           │
│   fn [테스트명] — [SQL 경로 검증]                    │
└────────────────────────────────────────────────────┘

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
어떻게 진행할까요?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

전체 계획 형식:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  테스트 작성 계획
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

총 [N]개 테스트 파일 · [M]개 테스트 함수 예정

┌─ 1그룹: T-T-06 공통 헬퍼 (먼저 준비) ─────────────┐
│ tests/common/mod.rs       — pub mod container; pub mod fixtures; │
│ tests/common/container.rs — postgres_url() (testcontainers)     │
│ tests/common/fixtures.rs  — fixture_new_user() 등               │
└─────────────────────────────────────────────────────┘

┌─ 2그룹: T-T-01 단위 테스트 ────────────────────────┐
│ [크레이트/파일명]                                    │
│   fn [테스트명] — [검증 내용]                        │
│   fn [테스트명] — [에러 케이스]                      │
└─────────────────────────────────────────────────────┘

┌─ 3그룹: T-T-02 Repository DB 테스트 ──────────────┐
│ {crate}/tests/{entity}_repository_test.rs           │
│   fn [테스트명] — [SQL 경로 검증]                    │
└─────────────────────────────────────────────────────┘

┌─ 4그룹: T-T-03 Usecase 통합 테스트 ───────────────┐
│ {crate}/tests/{usecase}_integration_test.rs         │
│   fn [테스트명] — [비즈니스 흐름 검증]               │
└─────────────────────────────────────────────────────┘

┌─ 5그룹: T-T-04 HTTP API 테스트 ───────────────────┐
│ controller/tests/{endpoint}_api_test.rs             │
│   fn [테스트명] — [HTTP 상태코드 + 응답 바디]         │
└─────────────────────────────────────────────────────┘

⏭️  보류: [의존성 추가 필요 항목 — T-T-05 proptest 등]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
어떻게 진행할까요?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**STEP 3 응답 처리:**

| 사용자 응답 | Claude 행동 |
|-------------|-------------|
| `"전체 진행"` | 모든 그룹을 1그룹부터 순서대로 STEP 4 진행 |
| `"1그룹만"` | 1그룹(T-T-06 헬퍼)만 STEP 4 진행 |
| `"T-T-01만"` / `"[T-T-XX]만"` | 지정 카탈로그 항목만 STEP 4 진행 |
| `"[숫자]그룹만"` / `"[숫자]그룹까지"` | 해당 그룹 범위만 STEP 4 진행 |
| `"보류 제외"` | 보류 항목 제외 후 나머지 STEP 4 진행 |
| `"취소"` / `"cancel"` | 테스트 작성 중단, 브랜치 정리 안내 출력 |

---

## STEP 4 — 테스트 코드 제시 → 인간 확인 → 파일 저장

이 단계는 승인된 그룹의 파일 수만큼 반복한다.
**Claude는 절대 먼저 파일을 저장하지 않는다. 코드를 제시하고 승인 후에만 저장한다.**

### 4-A. DB/API 테스트 환경 확인 (T-T-02, T-T-03, T-T-04 진입 전)

T-T-02, T-T-03, T-T-04 작성 전 환경을 먼저 확인한다.

확인 항목:
- Docker 실행 중 여부 (testcontainers 필수)
- `tests/common/container.rs` 존재 여부 (T-T-06)
- `tests/common/fixtures.rs` 존재 여부 (T-T-06)
- `Cargo.toml [dev-dependencies]`에 testcontainers, ctor 추가 여부
- `tower` / `http-body-util` dev-dependencies 추가 여부 (API 테스트)

환경 상태별 분기:

| 상태 | 처리 |
|------|------|
| Docker 실행 중 + deps 있음 | 정상 진행 |
| Docker **미실행** | T-T-02~04 중단, T-T-01·T-T-05는 계속 진행 가능함을 안내 |
| dev-dependencies 미설치 | Cargo.toml `[dev-dependencies]` 추가 안내 후 재실행 요청 |

Docker 미실행 시 출력:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  Docker 미실행 — DB/API 테스트 불가
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
T-T-02 / T-T-03 / T-T-04 는 testcontainers가 Docker를 필요로 합니다.

지금 진행 가능한 항목:
  ✅ T-T-01 단위 테스트 (Docker 불필요)
  ✅ T-T-05 프로퍼티 기반 테스트 (Docker 불필요)
  ⏭️  T-T-02~04 는 Docker 기동 후 재실행

계속하려면: "T-T-01만 진행" / "중단"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Docker가 실행 중이면 testcontainers가 자동으로 PostgreSQL을 기동한다.

### 4-B. 테스트 코드 제시 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪  [T-T-XX] [카탈로그 제목]  —  테스트 코드 제시
    ([진행: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 작성 위치:  [파일 경로]
📖 검증 대상:  [fn명 / 비즈니스 흐름]
📝 테스트 수:  [N]개 (정상 [a]개 + 에러 [b]개 + 경계 [c]개)
📏 규칙:       [rust-test-style.md §섹션명, rust-security-style.md §섹션명 해당 시]

─── 테스트 코드 ──────────────────────────
[전체 테스트 코드 — 기존 파일에 추가하는 경우 #[cfg(test)] 블록 포함]
[tests/ 디렉토리 신규 파일인 경우 파일 전체 내용 출력]

─── 커밋 메시지 제안 ─────────────────────
  test([scope]): [T-T-XX] [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 테스트를 저장할까요?

   ✅ "저장" / "ok" / "yes" / "ㅇ"  → 파일 저장 + cargo test 실행 + 커밋
   ❌ "건너뜀" / "skip" / "no" / "ㄴ" → 건너뛰고 다음 파일
   ✏️  "수정해줘: [요청]"               → 코드 재제안
   💬 "왜?" / "설명해줘"               → 상세 설명 후 동일 코드 유지
   ⏸️  "여기서 멈춰" / "stop"           → 완료 요약으로 이동
   🔁 "전체 저장"                       → 남은 항목 일괄 저장
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-C. 저장 후 처리

승인을 받으면 Claude가 Bash 도구로 아래를 **직접 실행**한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  [T-T-XX] 저장 중...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

실행 순서 (Claude가 직접 수행):
1. Write/Edit 도구로 파일 저장
2. `cargo fmt`
3. `cargo test [크레이트명] 2>&1` → 결과 출력
4. `git add [파일]`
5. `git commit -m "test([scope]): [T-T-XX] [요약]"`

### 4-D. cargo test 실패 시 처리

저장 후 테스트 실패 시 즉시 원인을 분석하고 수정안을 제시한다.
커밋은 테스트가 **반드시 통과한 뒤에만** 실행한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  cargo test 실패 — 원인 분석 중
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

실패 테스트: [fn명]
오류 메시지: [에러 내용]
원인: [분석 내용]

수정안:
[수정된 코드]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 5-0 — 커버리지 게이트 (PR 생성 전 필수 통과)

PR 초안을 생성하기 전에 반드시 `cargo tarpaulin`을 실행하고
커버리지가 **80% 이상**인지 확인한다.
**이 단계를 통과하지 못하면 PR을 절대 생성하지 않는다.**

```bash
cargo tarpaulin --out Stdout 2>&1 | tail -5
```

| 결과 | 조건 | 다음 단계 |
|------|------|-----------|
| ✅ 통과 | 커버리지 ≥ 80% | STEP 5 완료 요약 진행 |
| 🚫 차단 | 커버리지 < 80% | PR 생성 금지, 커버리지 갭 리포트 출력 |

#### 통과 시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  커버리지 게이트 통과
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
→ 완료 요약을 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 차단 시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚫  PR 차단 — 커버리지 기준 미달
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
부족분:        +Y.YY%p 필요

커버리지 낮은 파일:
  • [파일명] — X.X%  (기준 미달)

대응 방법:
  1. 위 파일에 추가 테스트 작성 (/test-rust [파일명])
  2. cargo tarpaulin 재측정
  3. 80% 달성 후 PR 진행

ℹ️  지금까지 커밋된 내용은 브랜치에 유지됩니다. PR만 차단됩니다.
🔒 정책: 커버리지 80% 미만이면 PR을 생성하지 않습니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 5 — 완료 요약 출력

### 5-A. 작업 완료 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉  테스트 작성 완료 요약
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

작성된 테스트 ([N]개):
  ✅ [T-T-XX] [파일 경로] — [N]개 테스트

건너뛴 항목 ([M]건):
  ⏭️  [설명] — [사유]

최종 검증 커맨드:
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test --all
  cargo tarpaulin --out Html --output-dir coverage/  # STEP 5-0에서 이미 통과

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  ■ 커버리지 ≥ 80% 확인 완료 (STEP 5-0 통과 필수)
  □ 단위 테스트: src 내부 #[cfg(test)] 위치 확인
  □ DB/통합 테스트: {crate}/tests/ 위치 + testcontainers 기반 확인
  □ HTTP API 테스트: controller/tests/ 위치 확인
  □ 공통 헬퍼: tests/common/ 완비 여부
  □ 금지 사항 전체 준수 (아래 금지 사항 목록 참조)
  □ 테스트명 네이밍 규칙 준수 (rules/rust-test-style.md §3. 테스트 네이밍)
  □ 테스트에 하드코딩 시크릿 없음 (rust-security-style.md §7 시크릿 관리)
  □ PR Red Flags 없음 (rules/rust-test-style.md §13. PR 거절 신호 (Red Flags))
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-B. 커버리지 측정 결과 (STEP 5-0 참조)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  커버리지 (STEP 5-0 측정 결과)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

전체: XX.XX%  (기준: 80%+)

크레이트별 목표 (CLAUDE.md 기준):
  common/     80%+
  domain/     80%+
  infra/      85%+
  usecase/    80%+
  controller/ 80%+

HTML 리포트 보기:
  cargo tarpaulin --out Html --output-dir coverage/
  open coverage/tarpaulin-report.html
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 6 — PR 생성 확인 및 실행

완료 요약 출력 후 PR 초안을 제시하고 사용자 승인을 받은 뒤에만 실제로 push + PR 생성을 수행한다.
**사용자가 명시적으로 승인하기 전까지 `git push`와 `gh pr create`를 절대 실행하지 않는다.**

### 6-A. PR 초안 제시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝  PR 초안 확인
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

■ 브랜치:  feature/test-[module-name]  →  main

■ PR 제목
  test([모듈명]): [테스트 종류 요약] [N]개 추가

■ PR 본문
────────────────────────────────────────
## 개요
[모듈명]에 대한 테스트를 추가합니다.

## 테스트 구조

| 종류 | 위치 | 개수 | 카탈로그 |
|------|------|------|---------|
| 단위 테스트 | src/**/*.rs #[cfg(test)] | [N]개 | T-T-01 |
| DB 테스트 | {crate}/tests/ | [N]개 | T-T-02 |
| 통합 테스트 | {crate}/tests/ | [N]개 | T-T-03 |
| HTTP API 테스트 | controller/tests/ | [N]개 | T-T-04 |
| 공통 헬퍼 | tests/common/ | — | T-T-06 |

## 커버리지 변화
  이전: [N]% → 이후: [N]% (목표: 80%+)

## 검증
(STEP 5-A PR 체크리스트 참조)
────────────────────────────────────────

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚦  위 PR을 생성할까요?

   ✅ "PR 생성" / "ok" / "ㅇ"     → push + gh pr create 실행
   ✏️  "제목 수정: [새 제목]"       → 제목 변경 후 재확인
   ✏️  "본문 수정: [요청]"          → 본문 변경 후 재확인
   ❌ "취소" / "skip"              → PR 생성 없이 종료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 6-B. 사용자 응답별 처리

| 응답 | Claude 행동 |
|------|-------------|
| `"PR 생성"` / `"ok"` / `"ㅇ"` | 6-C 실행 (push + PR 생성) |
| `"제목 수정: [내용]"` | 제목 변경 → 6-A 재출력 |
| `"본문 수정: [내용]"` | 본문 변경 → 6-A 재출력 |
| `"취소"` / `"skip"` / `"ㄴ"` | 종료. 수동 실행용 커맨드 출력 |

### 6-C. push 및 PR 생성 실행

승인 후 Claude가 Bash 도구로 직접 실행한다:

```bash
# 1. push (upstream 설정 포함)
git push -u origin feature/test-[module-name]

# 2. PR 생성
gh pr create \
  --title "test([모듈명]): [요약]" \
  --body "$(cat <<'EOF'
## 개요
[본문 내용]
EOF
)" \
  --base main
```

### 6-D. 결과 출력

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  PR 생성 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PR URL: [GitHub PR URL]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 6-E. 취소 시 수동 실행 안내

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PR 생성을 건너뜁니다.
수동으로 생성하려면:

  git push -u origin feature/test-[module-name]
  gh pr create --title "..." --body "..." --base main
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 금지 사항 (`rules/rust-test-style.md §13. PR 거절 신호 (Red Flags)` 전체 적용)

```
🚫 내부 모듈(Repository, Usecase) mock — 실제 DB 사용 (rust-test-style.md §4. 모킹 경계)
🚫 mockall 등 mock 프레임워크 내부 추가 — 정당성 없으면 PR Reject
🚫 #[ignore] 무단 추가 — 이슈·담당자·이유 없으면 삭제
🚫 테스트에 하드코딩된 시크릿 (rust-security-style.md §7 시크릿 관리)
🚫 비결정적 출력(타임스탬프, ID) 스냅샷 저장
🚫 상호작용 검증만 있고 상태 검증 없는 테스트
🚫 #[should_panic] — Result 반환 + assert!(result.is_err()) 방식으로 대체
🚫 sleep() 사용 — 근본 원인 해결 (rust-test-style.md §10. Flaky 테스트)
🚫 tests/ 외부 파일에 DB/HTTP 통합 테스트 작성
🚫 cargo test 실패 상태로 커밋
🚫 TEST_DATABASE_URL 환경변수 설정 요구 — testcontainers로 자동 처리
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `../../rules/rust-test-style.md` | 테스트 철학·Mocking·Naming·PR 기준 (권위 문서) | **STEP 2 분석 시작 전 로드** |
| `../../rules/rust-security-style.md` | 보안 규칙 (공통) | **STEP 2 분석 시작 전 로드** |
| `../../rules/rust-security-style.md` | 보안 규칙 (Rust 전용) | **STEP 2 분석 시작 전 로드** |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
