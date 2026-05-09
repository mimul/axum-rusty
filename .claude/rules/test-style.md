---
name: test-style
description: Axum-Rusty 프로젝트의 테스트 철학·전략·규칙을 정의한 문서. /test-rust 스킬의 1차 판단 기준으로 사용되며, 단위·통합·DB·HTTP API 테스트의 작성 방식, 모킹 경계, 네이밍, Assertion 스타일, Flaky 테스트 처리, Property-Based Testing(proptest), PR Red Flags를 포함한다. 새 테스트 작성, 기존 테스트 리뷰, Mockist→Classicist 마이그레이션 시 참조한다.
---

# Axum-Rusty Test Style Guide & Test Standards

본 문서는 axum-rusty 프로젝트의 테스트 철학과 테스트 표준을 정의한다.

이 문서는 단순 테스트 작성 규칙집이 아니다.

다음을 목표로 한다.

- 유지보수 가능한 테스트 구조
- Business behavior 중심 검증
- Architecture-aligned testing
- Property/Invariant 기반 사고
- Classicist TDD 철학 반영
- AI-assisted coding 시대의 테스트 일관성 확보

핵심 철학은 다음과 같다.

```text
Readable Test
> Behavior Verification
> Property Verification
> Architecture-Aligned Test
> Implementation-Coupled Test
```

즉:

- 읽기 쉬운 테스트
- 비즈니스 동작 검증
- invariant/property 검증
- 리팩토링에 강한 테스트

를 우선한다.

---

# 1. Think Before Testing

테스트는 "코드를 실행해보는 작업"이 아니다.

테스트는:

```text
무엇이 항상 참이어야 하는가?
```

를 정의하는 작업이다.

좋은 테스트 작성 전 질문:

- 이 기능의 핵심 business invariant는 무엇인가?
- 어떤 입력이 시스템을 깨뜨릴 수 있는가?
- 어떤 상태 전이가 허용되지 않는가?
- 이 테스트는 implementation에 결합되어 있는가?
- 리팩토링 후에도 유지 가능한가?

테스트는 코드보다 설계를 먼저 드러내야 한다.

## 요약 체크리스트

- invariant를 먼저 정의했는가?
- business rule 중심으로 생각하는가?
- implementation coupling을 줄였는가?
- edge case를 고려했는가?
- 테스트가 설계를 설명하는가?

---

# 2. Behavior First

테스트는 implementation verification이 아니라 사용자 관점 behavior verification 이어야 한다.

## 2.1 Test Observable Behavior

좋은 테스트:

```text
완료된 Todo는 다시 완료 처리할 수 없다
```

나쁜 테스트:

```text
repository.save()가 호출된다
```

내부 구현은 리팩토링 시 변경된다.

behavior는 상대적으로 안정적이다.

## 2.2 Prefer Public Contract

테스트는:

- public API
- business rule
- domain invariant
- observable result

에 의존해야 한다.

좋은 예:

```rust
assert_eq!(todo.status(), TodoStatus::Done)
```

## 요약 체크리스트

- observable behavior를 검증하는가?
- public contract를 사용하는가?
- 내부 구현에 의존하지 않는가?
- business rule을 검증하는가?

---

# 3. 모킹 경계

## 3.1 세 층의 경계

| 경계 | 대상 예시 | 처리 방법 |
|------|-----------|-----------|
| 프로세스 경계 | 외부 HTTP API, OAuth, 결제 | 반드시 대체 (wiremock-rs) |
| 시간/환경 경계 | SystemTime, 난수, 파일시스템 | 주입 가능한 인터페이스로 교체 |
| 논리적 경계 | 같은 crate 내 협력자 | 의도와 범위에 따라 결정 |

## 3.2 반드시 모킹할 대상

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

## 3.3 절대 모킹하면 안 되는 것

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

## 3.4 Fake vs Mock vs Stub

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

## 요약 체크리스트

- 프로세스 경계를 반드시 대체했는가?
- DB/Repository에 Mock 대신 Fake 또는 실제 DB를 사용하는가?
- 시간/환경 의존성을 주입 가능한 인터페이스로 교체했는가?
- 자체 소유 도메인 객체를 Mock하지 않는가?

---

# 4. Classicist TDD

axum-rusty는 Classicist TDD 관점을 기본 철학으로 사용한다.

## 4.1 Prefer State Verification

좋은 예:

```rust
#[test]
fn completed_todo_changes_status_to_done() {
    let mut todo = Todo::new(
        TodoTitle::new("write test".to_string()).unwrap()
    );

    todo.complete().unwrap();

    assert_eq!(
        todo.status(),
        TodoStatus::Done
    );
}
```

핵심은 무엇이 호출되었는가보다 결과적으로 상태가 어떻게 변했는가이다.

## 4.2 Prefer Real Domain Objects

```rust
#[test]
fn todo_title_cannot_be_empty() {
    let result = TodoTitle::new("".to_string());

    assert!(result.is_err());
}
```

도메인 로직을 mock 하지 않는다.

## 4.3 Mock at Infrastructure Boundary

mock은 외부 시스템 boundary에만 사용한다.

- TodoRepository
- EmailSender
- JwtProvider
- External API

등을 mock 한다.

## 요약 체크리스트

- state verification을 사용하는가?
- 실제 domain object를 사용하는가?
- boundary만 mock 하는가?

---

# 5. Property-Based Testing

Example-based test만으로는 충분하지 않다. 항상 유지되어야 하는 규칙(property)을 검증한다.

## 5.1 Example vs Property

Example-based:

```rust
#[test]
fn todo_title_accepts_normal_string() {
    let title = TodoTitle::new(
        "write guide".to_string()
    );

    assert!(title.is_ok());
}
```

Property-based:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_title_is_never_empty(
        s in ".{1,100}"
    ) {
        let title = TodoTitle::new(s);

        prop_assert!(title.is_ok());
    }
}
```

## 5.2 Invariant-Based Property

```rust
proptest! {
    #[test]
    fn completed_todo_never_returns_to_todo(
        title in ".{1,50}"
    ) {
        let mut todo = Todo::new(
            TodoTitle::new(title).unwrap()
        );

        todo.complete().unwrap();

        let result = todo.start();

        prop_assert!(result.is_err());
    }
}
```

## 5.3 Typed Arbitrary Generator

```rust
impl Arbitrary for TodoTitle {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters)
        -> Self::Strategy
    {
        ".{1,100}"
            .prop_map(|s|
                TodoTitle::new(s).unwrap()
            )
            .boxed()
    }
}
```

## 5.4 Stateful Workflow Property Testing

```rust
proptest! {
    #[test]
    fn deleted_todo_cannot_be_completed(
        title in ".{1,50}"
    ) {
        let mut todo = Todo::new(
            TodoTitle::new(title).unwrap()
        );

        todo.delete().unwrap();

        let result = todo.complete();

        prop_assert!(result.is_err());
    }
}
```

## 5.5 State Transition Invariant

상태 전이 순서 자체를 property로 검증한다.

```text
Todo → Doing → Done
```

유효하지 않은 전이는 항상 에러를 반환해야 한다:

```rust
proptest! {
    #[test]
    fn todo_state_cannot_skip_doing(
        title in ".{1,50}"
    ) {
        let mut todo = Todo::new(
            TodoTitle::new(title).unwrap()
        );

        // Todo 상태에서 바로 Done 전이 불가
        prop_assert!(todo.complete().is_err());
    }
}
```

## 요약 체크리스트

- property 기반 사고를 하는가?
- invariant를 검증하는가?
- typed generator를 사용하는가?
- workflow/state transition을 검증하는가?

---

# 6. Architecture-Aligned Testing

테스트 구조도 production architecture를 따라야 한다.

## 6.1 Domain Test First

```rust
#[test]
fn todo_cannot_complete_twice() {
    let mut todo = Todo::new(
        TodoTitle::new("write docs".to_string()).unwrap()
    );

    todo.complete().unwrap();

    let result = todo.complete();

    assert!(result.is_err());
}
```

## 6.2 Usecase Test

```rust
#[tokio::test]
async fn user_can_create_todo() {
    let repo = InMemoryTodoRepository::new();

    let usecase = CreateTodoUseCase::new(repo);

    let result = usecase.execute(
        CreateTodoCommand {
            title: "write guide".to_string(),
        }
    ).await;

    assert!(result.is_ok());
}
```

## 6.3 Controller Integration Test

```rust
#[tokio::test]
async fn create_todo_returns_201() {
    let app = test_app().await;

    let response = app
        .post("/api/todos")
        .json(&json!({
            "title": "write docs"
        }))
        .send()
        .await;

    assert_eq!(
        response.status(),
        StatusCode::CREATED
    );
}
```

## 요약 체크리스트

- domain 테스트를 우선하는가?
- usecase workflow를 검증하는가?
- integration test가 HTTP contract를 검증하는가?

---

# 7. Test Readability

테스트는 production code보다 읽기 쉬워야 한다.

## 7.1 AAA Structure

```text
Arrange
Act
Assert
```

## 7.2 Prefer Explicitness

```rust
let todo = result.unwrap();

assert_eq!(
    todo.status,
    TodoStatus::Done
);
```

## 7.3 Test Naming Convention

테스트 이름은 `<행동>_<기대결과>_when_<조건>` 패턴을 따른다.

```rust
// ❌ 구현 구조 반영
fn test_complete()
fn test_todo_repository_save()

// ✅ 행동(behavior) 기반
fn complete_todo_returns_error_when_already_done()
fn create_todo_fails_when_title_is_empty()
fn get_todo_returns_not_found_when_id_unknown()
```

`handle_`, `process_`, `run_` 같은 의미가 약한 접두사는 사용하지 않는다.

## 요약 체크리스트

- AAA 구조를 사용하는가?
- explicit assertion을 사용하는가?
- 테스트 이름이 `<행동>_<기대결과>_when_<조건>` 패턴을 따르는가?
- 함수명이나 내부 구현 구조를 그대로 반영하지 않는가?

---

# 8. Flaky & Deterministic Tests

불안정한 테스트는 버그와 같다.

## 8.1 핵심 원칙

- 불안정한 테스트는 절대 커밋하지 않는다.
- 이미 존재하는 Flaky 테스트는 **24시간 내** 격리(Quarantine)한다.
- 결정적(deterministic)이지 않은 테스트는 신뢰할 수 없다.

## 8.2 근본 원인

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

## 8.3 결정론적 테스트 원칙

타이밍에 의존하는 패턴을 제거한다:

```rust
// ❌ sleep으로 타이밍 조정
sleep(Duration::from_secs(1)).await;

// ✅ 상태 기반 대기 또는 채널 동기화
rx.recv().await.expect("이벤트 수신 실패");
```

각 테스트는 독립적으로 실행 가능해야 한다:

```rust
// ❌ 공유 인스턴스에 의존
static APP: OnceCell<TestApp> = OnceCell::new();

// ✅ 테스트마다 독립적인 인스턴스
let app = TestApp::new().await;
```

## 8.4 격리(Quarantine) 기준

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

## 요약 체크리스트

- flaky 테스트를 커밋하지 않는가?
- 발견된 flaky 테스트를 24시간 내 격리했는가?
- timing dependency를 피하는가?
- 공유 전역 상태를 제거했는가?
- 각 테스트가 독립적으로 실행 가능한가?
- 격리 시 이슈 링크·담당자·기한을 기록했는가?

---

# 9. Boundary & Failure Testing

성공 케이스만 테스트하지 않는다.

## 9.1 Validation Failure

다음을 테스트한다.

- malformed JSON
- invalid enum
- oversized payload
- invalid UUID

## 9.2 Unauthorized Access

```text
다른 사용자의 Todo 수정 시 403 반환
```

## 요약 체크리스트

- 실패 케이스를 테스트하는가?
- validation failure를 검증하는가?
- authorization failure를 검증하는가?

---

# 10. Integration over Mock Chains

긴 mock chain은 brittle test를 만든다.

## 10.1 Prefer Real Integration

가능하면 실제:

- routing
- middleware
- DB
- serialization

을 검증한다.

## 10.2 Black Box Style

```rust
let response = client
    .post("/todos")
    .json(&payload)
    .send()
    .await?;
```

## 요약 체크리스트

- black-box style을 사용하는가?
- 실제 integration을 검증하는가?
- mock chain을 줄이는가?

---

# 11. Security Testing

보안은 테스트 가능한 규칙이어야 한다.

## 11.1 Authentication Testing

검증 대상:

- invalid token
- expired token
- missing auth header

## 11.2 Authorization Testing

```text
사용자는 자신의 Todo만 수정 가능
```

## 요약 체크리스트

- auth failure를 테스트하는가?
- authorization을 검증하는가?
- 공격 시나리오를 테스트하는가?

---

# 12. Performance & Reliability Testing

성능 테스트는 운영 안정성을 검증하는 것이다.

## 12.1 Benchmark Critical Path

측정 대상:

- auth middleware
- DB hotspot
- serialization

## 12.2 Detect N+1 Query

integration/logging 기반으로 검출한다.

## 요약 체크리스트

- critical path를 측정하는가?
- N+1 query를 감지하는가?
- reliability를 우선하는가?

---

# 13. AI-Assisted Testing

AI-generated test도 동일한 품질 기준을 따라야 한다.

## 13.1 Never Trust Blindly

검토 항목:

- business meaning 있는가?
- flaky 가능성 없는가?
- invariant를 검증하는가?
- implementation coupling 없는가?

## 요약 체크리스트

- AI 테스트를 리뷰하는가?
- invariant를 검증하는가?
- 사람이 읽을 수 있는가?

---

# 14. Recommended Tooling

| Category | Tool |
|---|---|
| unit/integration | cargo test |
| property testing | proptest |
| async test | tokio::test |
| HTTP test | reqwest |
| coverage | cargo llvm-cov |
| benchmark | criterion |
| lint | clippy |
| formatting | cargo fmt |

CI 최소 권장:

```text
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo llvm-cov
cargo audit
```

---

# 15. 테스트 파일 구조

## 15.1 테스트 피라미드 비율

| 종류 | 비율 | 특징 |
|------|------|------|
| 단위 테스트 | 70% | 빠름, 격리, domain/usecase 중심 |
| 통합 테스트 | 20% | 실제 DB/HTTP, 계층 간 계약 검증 |
| E2E 테스트 | 10% | 중요 사용자 흐름 |

순수 CRUD는 통합 테스트 1개로 충분하다. 단위 테스트를 많이 작성하는 것이 목표가 아니라, 각 레이어에서 가장 효과적인 테스트 종류를 선택하는 것이 목표다.

## 15.2 배치 원칙

| 테스트 종류 | 위치 | 이유 |
|------------|------|------|
| 단위 테스트 | `src/` 내부 `#[cfg(test)]` 모듈 | 비공개 함수·구현 세부 접근 필요 시 |
| 통합 테스트 | `{crate}/tests/` 하위 | 공개 API만 테스트, 외부 시선 유지 |
| DB/HTTP API 테스트 | `{crate}/tests/` 하위 | 실제 인프라 필요 |

```
controller/
  src/
    routes/
      user.rs                        # User API 구현체 + #[cfg(test)] 단위 테스트
  tests/
    api_test.rs                      # HTTP API 통합 테스트
usecase/
  src/
    usecase/
      user.rs                        # User 유스케이스 구현체.
  tests/
    user_usecase_integration_test.rs # User 유스케이스 통합 테스트
infra/
  src/
    repository/
      user.rs                         # user repository 구현체
  tests/
    user_repository_test.rs           # DB 구현체
```

## 15.3 단위 테스트 모듈 구조

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

# 16. 테스트 피해야 할 경우

다음의 경우 테스트 작성을 피한다:

- 로직 없는 순수 CRUD → E2E 1개로 충분
- axum 라우팅, shaku DI 같은 프레임워크 배선
- 타입 시스템이 보장하는 정적 설정값, 상수
- 삭제 예정 코드
- 단순 getter/setter

> "이 테스트가 보호하는 동작을 한 문장으로 설명할 수 없으면, 작성하지 말 것"

---

# 17. PR 거절 신호 (Red Flags)

## 17.1 즉시 반려

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

## 17.2 주의 깊게 검토

1. `mockall` expect 호출이 실제 assert보다 압도적으로 많음
2. Arrange(설정·모킹) 코드가 Assert(검증) 코드보다 10배 이상 긴 경우 → Builder/Fixture 도입 필요

---

# 18. Rust 관례

## 18.1 테스트 모듈 위치 요약

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

## 18.2 테스트 헬퍼 크레이트

반복되는 test helper는 별도 모듈로 분리:

```rust
// tests/common/mod.rs 또는 tests/helpers.rs
pub async fn setup_test_app() -> TestApp { /* ... */ }
pub fn create_test_token(user_id: i64) -> String { /* ... */ }
```

## 18.3 성능 목표

- 단위 테스트: 1ms 이하
- 통합 테스트 1개당: 100~300ms
- 전체 테스트 스위트: 5분 이내 (CI 기준)
- PR 단위: 1분 이내

## 18.4 추천 크레이트

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

# 19. 마이그레이션 (기존 Mockist 코드베이스)

**새 테스트부터 Classicist로**

- 도메인 로직 → in-memory 단위 테스트
- 유스케이스 → `sqlx::test` 통합 테스트

**수정하는 파일과 함께 개선**

- 내부 모킹 제거, 외부 경계 모킹은 유지
- 동작 변경과 테스트 리팩토링을 별도 커밋으로 분리

**최악의 파일부터**

- `expect_xxx().times(n)` 가장 많은 곳
- 변경 빈도 높고 버그 자주 나는 곳
- Flaky가 자주 나타나는 곳
- 3~5개 파일부터 시작

**성공 지표**

- Flaky 비율 감소
- 테스트 실패 시 실제 버그 재현율 증가
- CI 안정성 향상

---

# 20. Final Testing Principles

1. 테스트는 behavior verification이다.
2. implementation coupling을 최소화한다.
3. state verification을 우선한다.
4. mock보다 real object를 선호한다.
5. invariant/property를 중요하게 본다.
6. domain 테스트를 가장 중요하게 본다.
7. AAA 구조를 유지한다.
8. flaky test를 허용하지 않는다.
9. 실패 시나리오를 반드시 검증한다.
10. boundary만 mock 한다.
11. integration test는 black-box style을 선호한다.
12. property testing을 적극 활용한다.
13. AI-generated test도 동일한 기준을 따른다.
14. readable test가 좋은 테스트다.
15. 유지보수 가능한 테스트가 가장 좋은 테스트다.
