---
name: rust-test-style
description: >
  Rust 프로젝트의 테스트 철학·전략·규칙을 정의한 권위 문서.
  /test-rust 스킬의 1차 판단 기준으로 사용되며,
  단위·통합·DB·HTTP API 테스트의 작성 방식, 모킹 경계,
  네이밍, Assertion 스타일, Flaky 테스트 처리,
  Property-Based Testing(proptest), PR Red Flags를 포함한다.
  새 테스트 작성, 기존 테스트 리뷰, Mockist→Classicist 마이그레이션 시 참조한다.
---

# Rust 테스트 스타일 가이드

## 1. 테스트 철학 (6원칙)

### 1.1 동작 기반 검증
구현이 아닌 동작을 테스트한다. 순수 리팩토링은 테스트를 깨뜨려서는 안 된다.

메서드 이름 변경, 반복문을 재귀로 교체 같은 내부 구현 변화에 테스트가 깨진다면, 동작이 아닌 구현을 잠그는 것이다.

### 1.2 시스템 경계에서만 모킹
내가 통제하고 소유하는 모든 것은 실제 구현을 사용하고, 통제 불가능한 것만 대체한다.

DB, 캐시, 내부 서비스는 실제 구현을 사용하되, 외부 결제 API나 OAuth 서버는 경계 밖이므로 모킹한다.

### 1.3 Classicist TDD 선호
Mockist 방식은 AI 기반 코드베이스에서 신뢰도가 빠르게 낮아진다.

AI가 구현을 리팩토링할 때마다 Mockist 테스트(상호작용 검증)는 깨지지만, Classicist 테스트(최종 상태 검증)는 사양을 만족하면 통과한다.

### 1.4 통합 테스트 우선
단위 테스트는 빠르지만, 유스케이스 하나를 실제 DB로 검증하면 단위 테스트 여러 개를 대체한다.

단위 테스트는 복잡한 도메인 로직과 엣지 케이스에만 집중한다.

### 1.5 FIDT 원칙
테스트는 빠르고(Fast), 격리되고(Isolated), 결정적(Deterministic)이어야 한다.

하나라도 무너지면 전체 스위트의 신뢰도가 떨어진다.

### 1.6 품질 중심
의미 있는 소수의 테스트가 신뢰도 낮은 다수의 테스트보다 가치 있다.

단순 getter나 프레임워크 초기화 코드까지 테스트하면 실행 비용만 올라간다.

---

## 2. 테스트 파일 구조

### 2.1 배치 원칙

| 테스트 종류 | 위치 | 이유 |
|------------|------|------|
| 단위 테스트 | `src/` 내부 `#[cfg(test)]` 모듈 | 비공개 함수·구현 세부 접근 필요 시 |
| 통합 테스트 | `{crate}/tests/` 하위 | 공개 API만 테스트, 외부 시선 유지 |
| DB/HTTP API 테스트 | `{crate}/tests/` 하위 | 실제 인프라 필요 |

```
src/
  domain/
    order.rs          # 도메인 로직 + #[cfg(test)] 단위 테스트
  usecase/
    create_order.rs   # 유스케이스 구현
  infra/
    order_repo.rs     # DB 구현체
tests/
  integration/
    order_api.rs      # HTTP API 통합 테스트
  db/
    order_repo.rs     # DB 통합 테스트
```

### 2.2 단위 테스트 모듈 구조

```rust
// src/domain/order.rs
pub struct Order { /* ... */ }

impl Order {
    pub fn apply_discount(&mut self, rate: f64) -> Result<(), DomainError> {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_discount_reduces_total_when_rate_is_valid() {
        // ...
    }
}
```

---

## 3. 테스트 네이밍

### 3.1 원칙
구현이 아닌 관찰 가능한 동작을 설명한다.

**나쁜 이름 (구현 중심)**:
```rust
#[test]
fn test_find_unique_called_once() { }

#[test]
fn test_calls_upsert_then_emits_event() { }
```

**좋은 이름 (행동 중심)**:
```rust
#[test]
fn returns_cached_result_when_fetched_within_ttl() { }

#[test]
fn rejects_login_when_password_is_expired() { }
```

### 3.2 권장 템플릿

```
<동작>_<예상_결과>_when_<조건>
```

**Rust 예시**:
```rust
#[test]
fn create_order_succeeds_when_stock_is_sufficient() { }

#[test]
fn create_order_fails_when_stock_is_zero() { }

#[test]
fn get_user_returns_none_when_not_found() { }

#[test]
fn apply_discount_returns_err_when_rate_exceeds_max() { }
```

일관된 동사 사용: `create`, `get`, `update`, `delete`, `returns`, `fails`, `rejects`, `succeeds`

---

## 4. 모킹 경계

### 4.1 세 층의 경계

| 경계 | 대상 예시 | 처리 방법 |
|------|-----------|-----------|
| 프로세스 경계 | 외부 HTTP API, OAuth, 결제 | 반드시 대체 (wiremock-rs) |
| 시간/환경 경계 | SystemTime, 난수, 파일시스템 | 주입 가능한 인터페이스로 교체 |
| 논리적 경계 | 같은 crate 내 협력자 | 의도와 범위에 따라 결정 |

### 4.2 반드시 모킹할 대상

```rust
// 외부 결제 API → wiremock-rs로 HTTP 레벨 Fake
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

let mock_server = MockServer::start().await;
Mock::given(method("POST"))
    .and(path("/payments"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&payment_response))
    .mount(&mock_server)
    .await;
```

### 4.3 절대 모킹하면 안 되는 것

```rust
// ❌ DB/Repository를 mock으로 대체
let mut mock_repo = MockOrderRepository::new();
mock_repo.expect_save().returning(|_| Ok(()));

// ✅ in-memory Fake 또는 실제 DB(sqlx::test) 사용
struct InMemoryOrderRepository {
    store: Mutex<HashMap<OrderId, Order>>,
}

// ✅ sqlx::test로 실제 DB 사용
#[sqlx::test]
async fn save_persists_order(pool: PgPool) {
    let repo = PostgresOrderRepository::new(pool);
    // ...
}
```

**절대 모킹하면 안 되는 것**:
- 자체 소유 Value Object, DTO, 도메인 엔티티
- 순수 함수와 유틸리티
- 같은 crate 내 협력자
- 테스트 대상 자체

### 4.4 Fake vs Mock vs Stub

| 종류 | 특징 | Rust 사용 시점 |
|------|------|----------------|
| **Fake** | 실제 동작하는 간단한 구현 (in-memory) | Repository, Clock → 상태 검증에 적합 |
| **Stub** | 고정된 응답만 반환, 호출 검증 없음 | 외부 API 응답 시뮬레이션 |
| **Mock** | 상호작용 검증, 호출 여부 확인 (mockall) | side-effect가 비즈니스 요구사항일 때만 |

```rust
// Fake Clock 예시 — 시간 의존성 제거
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct FakeClock {
    fixed_time: DateTime<Utc>,
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        self.fixed_time
    }
}

// 프로덕션 코드에 Clock 주입
pub struct OrderService<C: Clock> {
    clock: C,
    // ...
}
```

---

## 5. Assertion 스타일

### 5.1 AAA 패턴

```rust
#[test]
fn apply_discount_reduces_total_by_rate() {
    // Arrange
    let mut order = Order::new(Money::new(10_000));

    // Act
    order.apply_discount(0.1).unwrap();

    // Assert
    assert_eq!(order.total(), Money::new(9_000));
}
```

- **Arrange**: 테스트 상태 준비
- **Act**: 정확히 한 번만 동작 실행
- **Assert**: 관찰 가능한 결과 검증

### 5.2 관찰 가능한 것 vs 관찰 불가능한 것

**관찰 가능한 것** (검증해야 할 것):
- 함수 반환값
- DB에 저장된 실제 상태
- HTTP 응답 상태 코드·본문
- 이벤트 발생 여부

**관찰 불가능한 것** (검증하면 안 되는 것):
- 내부 메서드 호출 순서
- 협력 객체에 전달된 인자

```rust
// ❌ 나쁜 예 — 구현에 결합됨
let mut mock_repo = MockOrderRepository::new();
mock_repo.expect_save()
    .with(predicate::eq(order.clone()))
    .times(1)
    .returning(|_| Ok(()));

// ✅ 좋은 예 — 결과 상태에 집중
#[sqlx::test]
async fn create_order_persists_to_db(pool: PgPool) {
    let service = OrderService::new(PostgresOrderRepository::new(pool.clone()));
    service.create(CreateOrderCommand { /* ... */ }).await.unwrap();

    let saved = sqlx::query_as::<_, OrderRow>("SELECT * FROM orders WHERE ...")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(saved.status, "pending");
}
```

### 5.3 객체 비교 전략

```rust
// 전체 비교 — 안정적인 DTO나 결과 객체
assert_eq!(result, expected_order);

// 부분 비교 — 타임스탬프, 자동 생성 ID 같은 비결정적 필드 제외
assert_eq!(result.status, OrderStatus::Pending);
assert_eq!(result.total, Money::new(10_000));
// created_at은 검증하지 않음
```

### 5.4 에러 검증

```rust
// Result 에러 타입 검증
let err = service.create(invalid_command).await.unwrap_err();
assert!(matches!(err, DomainError::InvalidInput(_)));

// 에러 메시지 검증이 필요할 때
assert!(err.to_string().contains("title must not be empty"));
```

---

## 6. 테스트 피라미드

### 6.1 권장 비율

| 레이어 | 목적 | 비율 | Rust 도구 |
|--------|------|------|-----------|
| Unit | 도메인 로직, 알고리즘, 경계 조건 | 70% | `#[test]` |
| Integration | 유스케이스 + 실제 DB/인프라 | 20% | `#[sqlx::test]`, `#[tokio::test]` |
| E2E / API | 핵심 HTTP 엔드포인트 | 10% | `axum::test`, reqwest |

### 6.2 생략 가능한 경우

- 권한·필터링 없는 순수 CRUD → 통합 테스트 1개로 충분
- 프레임워크 자체 기능 (axum 라우팅, DI) → 생략 가능하되, 연결 확인은 필수
- 정적 설정값, 상수 → 타입 시스템이 검증
- 삭제 예정 코드 → 변경 발생 시 최소 보호 테스트만 추가

### 6.3 반드시 테스트할 것

- 인증, 권한 체크, 결제 등 핵심 로직
- 복잡한 분기, 상태 전환 로직
- 과거 버그 발생 경로 (회귀 테스트)

---

## 7. 비동기 테스트

### 7.1 tokio::test

```rust
#[tokio::test]
async fn fetch_user_returns_user_when_exists() {
    let service = setup_service().await;
    let result = service.find_by_id(UserId::new(1)).await;
    assert!(result.is_ok());
}
```

### 7.2 sqlx::test — 실제 DB 테스트

```rust
// sqlx::test는 테스트마다 독립된 트랜잭션을 자동으로 롤백
#[sqlx::test(fixtures("users", "orders"))]
async fn find_order_returns_order_with_user(pool: PgPool) {
    let repo = PostgresOrderRepository::new(pool);
    let order = repo.find_by_id(OrderId::new(1)).await.unwrap();
    assert_eq!(order.user_id, UserId::new(1));
}
```

### 7.3 axum HTTP 테스트

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn create_order_returns_201_when_valid() {
    let app = create_app(test_state().await);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/orders")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"product_id": 1, "quantity": 2}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

---

## 8. 도메인 엔티티 추출

### 8.1 추출 기준 (하나 이상 해당 시)

1. 같은 데이터를 대상으로 하는 비즈니스 로직이 2개 이상 서비스에 흩어져 있음
2. 서비스가 DB 행에 직접 산술 연산이나 상태 전환 수행
3. 순수 로직인데 테스트 위해 DB를 매번 시작해야 함
4. 중요한 불변식이 여러 서비스에 중복되어 체크됨

### 8.2 Before / After

```rust
// ❌ Before — 서비스에 도메인 로직 뒤섞임
impl OrderService {
    pub async fn apply_discount(&self, order_id: i64, rate: f64) {
        let mut row = self.repo.find(order_id).await.unwrap();
        let discount = (row.total_price as f64 * rate).min(MAX_DISCOUNT as f64) as i64;
        row.total_price -= discount;  // DB 행에 직접 산술
        self.repo.save(row).await.unwrap();
    }
}

// ✅ After — 도메인 엔티티에 로직 이전
impl Order {
    pub fn apply_discount(&mut self, rate: f64) -> Result<(), DomainError> {
        if !(0.0..=1.0).contains(&rate) {
            return Err(DomainError::InvalidDiscountRate(rate));
        }
        let discount = (self.total.amount() as f64 * rate) as i64;
        self.total = self.total.subtract(Money::new(discount.min(MAX_DISCOUNT)))?;
        Ok(())
    }
}

// DB 없이 순수 단위 테스트 가능
#[test]
fn apply_discount_clamps_at_max_discount() {
    let mut order = Order::new(Money::new(1_000_000));
    order.apply_discount(0.5).unwrap();
    assert!(order.total().amount() >= MIN_TOTAL);
}
```

---

## 9. Property-Based Testing (proptest)

### 9.1 사용 시점

- 같은 함수에 4번째 예시 테스트를 작성하려는 순간 전환 검토
- 검증기, 도메인 규칙, 파서, 상태 머신
- 대수적 성질 있는 함수 (idempotent, commutative, associative)

### 9.2 proptest 예시

```rust
use proptest::prelude::*;

proptest! {
    // 멱등성: 정렬을 두 번 해도 결과 동일
    #[test]
    fn sort_is_idempotent(mut v: Vec<i32>) {
        let once = { let mut x = v.clone(); x.sort(); x };
        let twice = { let mut x = once.clone(); x.sort(); x };
        prop_assert_eq!(once, twice);
    }

    // 도메인 규칙: 유효한 이메일 형식은 항상 파싱 성공
    #[test]
    fn valid_email_always_parses(
        local in "[a-z]{1,20}",
        domain in "[a-z]{2,10}",
    ) {
        let email = format!("{}@{}.com", local, domain);
        prop_assert!(Email::parse(&email).is_ok());
    }

    // 불변식: 할인 적용 후 금액은 항상 0 이상
    #[test]
    fn discount_never_makes_total_negative(
        amount in 1_000i64..10_000_000,
        rate in 0.0f64..=1.0,
    ) {
        let mut order = Order::new(Money::new(amount));
        let _ = order.apply_discount(rate);
        prop_assert!(order.total().amount() >= 0);
    }
}
```

### 9.3 부적합한 경우

- 네트워크 호출, 외부 API (속도·결정성 보장 불가)
- Side-effect가 주요 목적인 유스케이스

---

## 10. Flaky 테스트

### 10.1 핵심 원칙

불안정한 테스트는 절대 커밋하지 않는다.

### 10.2 근본 원인

```rust
// ❌ 공유 전역 상태 — 테스트 간 격리 미흡
static COUNTER: AtomicUsize = AtomicUsize::new(0);

// ❌ 시스템 시계 직접 의존
let now = SystemTime::now(); // 비결정적

// ❌ 시드 없는 난수
let random_id = rand::random::<u64>(); // 비결정적

// ✅ Clock 인터페이스 주입으로 결정적 처리
let clock = FakeClock::fixed(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
```

**공통 원인**:
- 공유 전역 상태 (테스트 간 격리 미흡)
- `SystemTime::now()` 직접 사용
- 실행 순서 의존
- 시드 없는 난수
- 네트워크/외부 서비스 직접 의존
- 비동기 Race Condition (`tokio::time::sleep` 남용)
- 병렬 실행 시 DB 자원 충돌 (sqlx::test으로 해결)

### 10.3 격리(Quarantine) 기준

단순 `#[ignore]`가 아닌, 이슈 링크 + 담당자 + 기한 + 원인 가설 기록:

```rust
// ✅ 올바른 격리
#[ignore = "Flaky: race condition in async setup. Issue: #123, Owner: @mimul, Due: 2024-03-01"]
#[tokio::test]
async fn flaky_test_with_context() { }

// ❌ 이유 없는 ignore
#[ignore]
#[tokio::test]
async fn ignored_without_reason() { }
```

임시방편 금지: 재시도, `tokio::time::sleep`, 타임아웃 증가

---

## 11. 테스트 픽스처와 빌더

### 11.1 Builder 패턴

설정과 모킹 코드가 검증 코드보다 훨씬 많다면 Builder 패턴 도입 신호다.

```rust
// 테스트 픽스처 빌더
struct OrderBuilder {
    total: Money,
    status: OrderStatus,
    user_id: UserId,
}

impl OrderBuilder {
    fn new() -> Self {
        Self {
            total: Money::new(10_000),
            status: OrderStatus::Pending,
            user_id: UserId::new(1),
        }
    }

    fn with_total(mut self, amount: i64) -> Self {
        self.total = Money::new(amount);
        self
    }

    fn with_status(mut self, status: OrderStatus) -> Self {
        self.status = status;
        self
    }

    fn build(self) -> Order {
        Order::reconstruct(self.total, self.status, self.user_id)
    }
}

// 사용
#[test]
fn cancel_succeeds_when_order_is_pending() {
    let mut order = OrderBuilder::new()
        .with_status(OrderStatus::Pending)
        .build();
    assert!(order.cancel().is_ok());
}
```

### 11.2 sqlx fixtures

```sql
-- tests/fixtures/users.sql
INSERT INTO users (id, email, name) VALUES
    (1, 'test@example.com', 'Test User');
```

```rust
#[sqlx::test(fixtures("users"))]
async fn find_user_by_email_returns_user(pool: PgPool) { }
```

---

## 12. 테스트 피해야 할 경우

다음의 경우 테스트 작성을 피한다:

- 로직 없는 순수 CRUD → E2E 1개로 충분
- axum 라우팅, shaku DI 같은 프레임워크 배선
- 타입 시스템이 보장하는 정적 설정값, 상수
- 삭제 예정 코드
- 단순 getter/setter

> "이 테스트가 보호하는 동작을 한 문장으로 설명할 수 없으면, 작성하지 말 것"

---

## 13. PR 거절 신호 (Red Flags)

### 13.1 즉시 반려

1. 상호작용 검증만 있고 결과 상태 검증 없음
   - 예외: 이메일 발송, 이벤트 발행 등 side-effect가 비즈니스 요구사항
2. 통합 테스트에서 Mock DB/Repository 사용 (in-memory Fake는 허용)
3. `mod tests` 외부에서 `pub(super)` 등으로 내부 구현에 직접 접근
4. `SystemTime::now()`, LLM 응답 등 비결정적 출력을 고정값처럼 사용
5. 이슈 링크·담당자·기한 없이 단순 `#[ignore]`
6. 테스트 이름이 함수명이나 내부 구현 구조를 그대로 반영
7. 기존 도구로 충분한데 새 모킹 크레이트 추가
8. **Assertion이 없는 테스트**
9. 의미 없는 Assertion (`assert!(result.is_some())` 단독 사용)

### 13.2 주의 깊게 검토

1. `mockall` expect 호출이 실제 assert보다 압도적으로 많음
2. Arrange(설정·모킹) 코드가 Assert(검증) 코드보다 10배 이상 긴 경우 → Builder/Fixture 도입 필요

---

## 14. Rust 관례

### 14.1 테스트 모듈 위치 요약

```rust
// 단위 테스트: 같은 파일 하단
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}

// 통합 테스트: tests/ 디렉토리 (별도 크레이트처럼 컴파일)
// tests/integration/order_api.rs
```

### 14.2 테스트 헬퍼 크레이트

반복되는 test helper는 별도 모듈로 분리:

```rust
// tests/common/mod.rs 또는 tests/helpers.rs
pub async fn setup_test_app() -> TestApp { /* ... */ }
pub fn create_test_token(user_id: i64) -> String { /* ... */ }
```

### 14.3 성능 목표

- 단위 테스트: 1ms 이하
- 통합 테스트 1개당: 100~300ms
- 전체 테스트 스위트: 5분 이내 (CI 기준)
- PR 단위: 1분 이내

### 14.4 추천 크레이트

| 목적 | 크레이트 |
|------|---------|
| 비동기 테스트 | `tokio::test` (내장) |
| DB 통합 테스트 | `sqlx::test` |
| HTTP 모킹 | `wiremock-rs` |
| Property-Based | `proptest` |
| 단순 Mock/Stub | `mockall` (최소한으로) |
| HTTP 클라이언트 테스트 | `axum::test` + `tower::ServiceExt` |
| Assertion 보강 | `pretty_assertions` |

---

## 15. 마이그레이션 (기존 Mockist 코드베이스)

1. **새 테스트부터 Classicist로**
   - 도메인 로직 → in-memory 단위 테스트
   - 유스케이스 → `sqlx::test` 통합 테스트

2. **수정하는 파일과 함께 개선**
   - 내부 모킹 제거, 외부 경계 모킹은 유지
   - 동작 변경과 테스트 리팩토링을 별도 커밋으로 분리

3. **최악의 파일부터**
   - `expect_xxx().times(n)` 가장 많은 곳
   - 변경 빈도 높고 버그 자주 나는 곳
   - Flaky가 자주 나타나는 곳
   - 3~5개 파일부터 시작

4. **성공 지표**
   - Flaky 비율 감소
   - 테스트 실패 시 실제 버그 재현율 증가
   - CI 안정성 향상
