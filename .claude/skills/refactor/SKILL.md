---
name: refactor
description: |
  /refactor 커맨드로 실행되는 Rust 코드 리팩토링 자동화 스킬.
  `.claude/rules/coding-style.md` 19개 섹션을 권위 문서로 삼아 Behavior Preserving·Domain First·Small Safe Steps 원칙을 적용한다.

  지원 옵션:
    /refactor                           프로젝트 전체 리팩토링
    /refactor [scope]                   모듈·디렉토리·파일·glob 패턴 단위 리팩토링
    /refactor [scope] --goal <goal>     목표 지정 (readability|maintainability|testability|domain-model|complexity)
    /refactor [scope] --level <level>   강도 지정 (safe|moderate|aggressive)
    /refactor [scope] --with-tests      characterization·regression·edge case 테스트 함께 작성
    /refactor [scope] --dry-run         실제 수정 없이 code smell·영향 범위·위험도 분석만 출력

  실행 흐름 (3단계):
    Preparation     목표 정의 → 변경 범위 식별 → 기존 테스트 확인 → Code Smell 분석(Rust 특화 포함)
                    → 리팩토링 전략 선택 → 리스크 분석
    Execute         coding-style.md §1~19 체크리스트 기반 일괄 점검 후 17개 항목 순차 수행
                    (Naming·함수·Struct/Trait·조건문·데이터구조·에러처리·로깅·Dependency·
                     Dead Code·Comments·pub 범위·DTO 경계·Usecase 책임·Async·Auth·문서화)
                    브랜치: feature/refactor-{작업단위} / 각 항목마다 리팩토링→테스트→커밋 반복
    Verification    cargo clippy·fmt → cargo test --all → /security-full-scan →
                    아키텍처 리뷰(controller→usecase→domain←infra 방향) → complexity 측정 → diff 검토

  피드백 분류:
    🚫 Blocking     behavior change·transaction 손상·architecture 위반·security regression 등 즉시 수정 필수
    ⚠️ Recommended  long method·duplicate code·testability 부족 등 강력 권장
    💡 Suggestions  선택적 개선 아이디어 (extensibility·performance·domain refinement 등)
    📝 Tech Debt    현재 범위 초과 구조적 부채 → 별도 이슈로 추적

  핵심 제약:
    - 기능 추가와 리팩토링을 같은 커밋에 혼합 금지
    - behavior change 발생 시 즉시 중단
    - 테스트 미통과 상태로 커밋 금지
    - aggressive 레벨에서도 behavior verification 반드시 유지
---

# Claude용 리팩토링 가이드

본 문서는 Axum Rusty 프로젝트의 코딩 철학과 표준을 반영한 리팩토링 가이드를 제공한다.

# 1. 커맨드 문법

권장 명령은 아래 수준으로 단순하게 유지한다.

```bash
/refactor [scope]
```

필요 시에만 아래 옵션을 추가한다.

```bash
--goal
--level
--with-tests
--dry-run
```

예:

```bash
/refactor
/refactor payments
/refactor ./src/domain/order
/refactor payments --goal maintainability
/refactor legacy --dry-run
```

## Scope 해석 규칙

scope가 없으면 프로젝트 전체를 대상으로 한다. scope는 아래 중 하나로 해석한다.

- module
- directory
- file
- glob pattern

Claude는 scope를 기반으로 영향 범위와 의존성을 분석한다.

## Goal 해석 규칙

### readability
- naming 개선 우선 (금지 접두사 제거, `<동사>_<대상>` 형태 적용)
- extract method 우선
- 불필요한 comment 제거 (단, `///` doc 주석과 `// SAFETY:` 주석은 보호)
- 함수 depth 감소

### maintainability
- dependency reduction 우선
- duplication 제거
- large struct/impl 분리
- 응집도 향상

### testability
- 테스트 어려운 구조 개선
- side effect 감소
- dependency injection 개선
- characterization test 추가

### domain-model
- primitive obsession 제거 (`Id<T>` Newtype, Enum 도입)
- domain object 강화
- business rule 응집
- domain terminology 우선

### complexity
- conditional logic 단순화 (match 과다 분기 정리)
- nested structure 감소
- cognitive complexity 감소
- dead code 제거

## Level 해석 규칙

### safe
- behavior preserving을 최우선으로 한다
- public API 변경을 피한다
- architecture rewrite를 지양한다
- rename / extract method 중심으로 진행한다
- 작은 단계로 나누어 수행한다
- 각 단계마다 테스트를 수행한다

### moderate
- 일반적인 구조 개선을 허용한다
- struct 분리와 dependency cleanup을 허용한다
- 내부 API 개선을 허용한다
- maintainability 향상을 우선한다

### aggressive
- architecture 개선을 허용한다
- domain restructuring을 허용한다
- large-scale struct split을 허용한다
- legacy abstraction 제거를 허용한다
- 다만 behavior verification은 반드시 유지한다

## Test 정책

`--with-tests` 옵션이 있으면 Claude는:

- characterization test
- regression test
- edge case test
- flaky test 개선

을 함께 수행한다.

테스트가 부족한 경우:
- 기존 동작을 먼저 캡처한다
- behavior preserving verification을 우선한다

## Dry Run 정책

`--dry-run` 옵션이 있으면 실제 수정 대신:

- code smell 분석
- architecture issue 분석
- dependency risk 분석
- 영향 범위 분석
- incremental 실행 계획
- 예상 위험도

를 우선 출력한다.

## 기본 실행 정책

옵션이 없더라도 Claude는 본 가이드의:

- Refactoring Principles (섹션 2)
- Preparation 절차 (섹션 3.1)
- Execute Refactoring 절차 (섹션 3.2)
- Verification & Cleanup 절차 (섹션 3.3)
- 피드백 작성 가이드라인 (섹션 4)
- PR 준비 정책 (섹션 5)

을 자동으로 적용한다.

즉:

```bash
/refactor payments
```

만 수행해도 Claude는:

- code smell 분석
- dependency 분석
- 안전한 단계 분리
- 테스트 검증
- lint / formatter 검증
- dead code 제거
- security regression 검토
- PR 전략 생성

을 본 가이드 기준으로 수행한다.

---

# 2. 리팩토링 원칙 (Refactoring Principles)

### 2.1 Behavior Preserving
- 리팩토링은 **동작 변경 없이 구조를 개선**하는 활동이다.
- 기능 추가와 리팩토링을 같은 커밋에 섞지 않는다.
- 작은 단위로 변경하고 매 단계마다 검증한다.

### 2.2 Small Safe Steps
- 한 번에 큰 구조 변경을 하지 않는다.
- 항상 작은 변경, 즉시 테스트, 즉시 검증, 즉시 커밋 순서로 진행한다.
- 실패 시 쉽게 rollback 가능해야 한다.

### 2.3 Readability First
- 코드는 작성보다 읽기가 더 많다.
- 코드 길이보다 의도 전달, 응집도, 변경 용이성을 우선한다.

### 2.4 Domain First
- 기술 구조보다 도메인 개념을 우선한다.
- 도메인 모델이 비즈니스 개념을 자연스럽게 표현해야 한다.
- Primitive Obsession을 제거한다.

### 2.5 Simplicity First
- 미래를 위한 추상화보다 현재 문제를 명확히 해결하는 구조를 선호한다.
- Speculative Generality를 피한다.

### 2.6 Explicit Intent
- 이름만 보고 역할을 이해할 수 있어야 한다.
- 숨겨진 side effect를 제거한다.
- 상태 변경은 명시적으로 표현한다.

### 2.7 Cohesion & Loose Coupling
- 함께 변경되는 코드는 함께 위치시킨다.
- 불필요한 의존성을 제거한다.
- Message Chain / Middle Man / Shotgun Surgery를 줄인다.

### 2.8 Testability
- 테스트하기 어려운 코드는 설계 문제가 존재할 가능성이 높다.
- 테스트 용이성을 설계 품질의 핵심 지표로 본다.

### 2.9 Refactor Continuously
- 리팩토링은 별도 이벤트가 아니라 지속적 활동이다.
- Boy Scout Rule: "코드를 발견했을 때보다 더 깨끗하게 남긴다."

---

# 3. Refactoring Process (단계별 실행 절차)

## 3.1 Preparation

### 3.1.1 목표 정의
리팩토링 목적을 명확히 정의한다.

예:
- 가독성 개선
- 복잡도 감소
- 중복 제거
- 테스트 용이성 향상
- 도메인 모델 개선
- 성능 병목 제거
- 결합도 감소
- 보안 취약 구조 개선

---

### 3.1.2 변경 범위 식별
아래를 분석한다.

- 영향받는 모듈
- 의존 관계
- API 계약
- 데이터 흐름
- 상태 변경 지점
- 트랜잭션 경계
- 동시성 영향
- 외부 시스템 영향

---

### 3.1.3 Existing Tests 확인
반드시 확인:
- Unit Test 존재 여부
- Integration Test 존재 여부
- E2E Test 존재 여부
- Critical Path 보호 여부

부족하면 먼저 테스트를 작성한다.

특히:
- 현재 동작 캡처 (Characterization Test)
- 회귀 방지 테스트
- Edge Case 테스트

---

### 3.1.4 Code Smell 분석

**Bloaters**
- Long Method
- Large Struct / God Impl Block
- Long Parameter List
- Primitive Obsession
- Data Clumps

**OO Abusers**
- match 과다 분기
- Temporary Field
- Alternative Structs with Different Interfaces

**Change Preventers**
- Divergent Change
- Shotgun Surgery
- Parallel Inheritance Hierarchies

**Dispensables**
- Duplicate Code
- Dead Code
- Lazy Module
- Speculative Generality
- 과도한 Comment

**Couplers**
- Feature Envy
- Inappropriate Intimacy
- Message Chain
- Middle Man

**Rust 특화 Code Smell**
- `unwrap()` / `expect()` 남용 — recoverable error에서 panic 가능성
- format string 로깅 — `error!("msg: {:?}", err)` 형태로 집계 도구에서 필드 파싱 불가
- String 식별자 — `id: String` 대신 `Id<T>` Newtype 필요
- String 상태값 — `status: String` 대신 `enum` 필요
- 레이어 간 에러 미변환 — `sqlx::Error`가 usecase 이상으로 노출
- framework 타입이 usecase에 직접 사용 — `Json<T>`, `axum::Extension` 등
- `pub` 과잉 노출 — 외부 공개가 불필요한 항목에 `pub` 사용
- `String` 파라미터 — `&str` / `&[T]` 로 대체 가능한 경우
- 금지 접두사 함수명 — `handle_`, `process_`, `run_`, `do_` 로 시작하는 함수

---

### 3.1.5 Refactoring 전략 선택

문제 유형에 따라 전략 선택:

| 문제 | 대표 리팩토링 |
|---|---|
| Long Method | Extract Method |
| Primitive Obsession | Replace Primitive with Object |
| Duplicate Code | Extract Function / Template Method |
| Large Struct / God Impl | Extract Struct / Trait 분리 |
| match 과다 분기 | Strategy / Polymorphism (trait 활용) |
| Shotgun Surgery | Move Method / Move Field |
| Message Chain | Hide Delegate |
| Long Parameter | Parameter Object / Command Object |
| `unwrap()` 남용 | `map_err` + `?` + 레이어별 에러 타입 |
| String 식별자 | `Id<T>` Newtype 도입 |
| String 상태값 | `enum StatusName { ... }` 도입 |
| 레이어 에러 노출 | `impl From<LowerError> for UpperError` |
| format string 로깅 | `tracing` 필드 기반 구조화 로깅 |
| framework coupled usecase | Command Object 추출 |
| `String` 파라미터 | `&str` / `&[T]` 로 교체 |
| `pub` 과잉 노출 | `pub(crate)` / `pub(super)` 로 축소 |

---

### 3.1.6 리스크 분석
다음을 반드시 점검한다.

- Public API 변경 여부
- Backward Compatibility
- Migration 필요 여부
- 데이터 손상 가능성
- 성능 영향
- Lock/Concurrency 영향
- Security 영향

---

## 3.2 Execute Refactoring

### 3.2.0 coding-style.md 기준 일괄 점검

Execute Refactoring을 시작하기 전에, `.claude/rules/coding-style.md`의 19개 섹션 체크리스트를 기준으로 전체 코드를 점검한다. 각 체크리스트 항목의 위반 사항을 식별하고 이후 단계에서 우선 개선한다.

| 섹션 | 주요 체크 |
|---|---|
| §1 Domain First | business meaning 표현, primitive obsession 제거, explicit conversion, DB concern 분리 |
| §2 Architecture First | 의존 방향(`controller→usecase→domain←infra`), domain이 framework 무지, workspace boundary |
| §3 Explicit & Intentional | 데이터 흐름 명확, hidden behavior 없음, 함수명이 의도 표현 |
| §4 Readability First | 단순 control flow, 낮은 nesting depth, `<동사>_<대상>` 명명 |
| §5 Complexity Control | 과도한 추상화 없음, 낮은 cognitive load, composition 우선 |
| §6 Changeability | framework coupling 낮음, stable boundary 존재, concern separation |
| §7 Consistency | naming 일관성, module structure 예측 가능, error strategy 통일 |
| §8 Usecase Oriented | controller thin 여부, business logic이 controller에 없음, usecase가 workflow 소유 |
| §9 Dependency Injection | constructor injection 사용, trait boundary 존재, composition root |
| §10 Error Handling | unwrap/expect 남용 없음, 의미 있는 에러, 레이어별 에러 경계 |
| §11 Type-Driven Design | 타입이 business rule 표현, enum 활용, invalid state 제거 |
| §12 Async & Concurrency | Arc로 공유 상태 관리, mutable state 최소화, async가 I/O 중심 |
| §13 Database & Repository | persistence detail 숨김, domain이 SQL 모름, transaction boundary 명확 |
| §14 API Design | explicit DTO 분리, boundary validation, serialization이 controller 책임 |
| §15 Documentation | pub 항목에 `///` 주석 존재, API schema 최신 상태 |
| §16 Auth & Middleware | 인증이 middleware 분리, auth 중복 없음, security failure explicit |
| §17 Observability | tracing 필드 기반 구조화 로깅, context-rich log, error 삼킴 없음 |
| §18 Testing Philosophy | business behavior 테스트, implementation coupling 낮음 |
| §19 AI Coding Alignment | 예측 가능한 패턴, architecture consistency |

점검 결과를 기반으로 아래 3.2.1~3.2.17 중 우선 수행할 항목을 결정한다.

---

### 3.2.1 작은 단위로 진행

작업 단위를 작게 쪼개고, 브랜치는 `feature/refactor-{작업단위}` 형태로 만든다. 각 단위마다 리팩토링 → 테스트 → 커밋을 반복한다.

---

### 3.2.2 Naming 개선
다음을 개선한다.

- 의미 없는 변수명 제거
- 타입 기반 이름 제거
- 축약어 최소화
- 도메인 용어 사용
- Boolean Flag 제거

**금지 접두사** — 의미가 약해 역할을 드러내지 못한다:

```rust
// 나쁜 예
handle_auth()
process_order()
run_job()
do_cleanup()

// 좋은 예: <동사>_<대상> 형태로 의도를 명시
validate_access_token()
complete_order()
execute_batch_export()
remove_expired_sessions()
```

예:
- `data` → `order_items`
- `flag` → `is_expired`

---

### 3.2.3 함수 리팩토링

목표:
- 단일 책임
- 의도 중심
- Side Effect 최소화

체크:
- 함수가 여러 역할 수행하는가?
- 조건문이 과도한가?
- depth가 깊은가?
- mutable state가 많은가?

---

### 3.2.4 Struct / Impl / Trait 리팩토링

체크:
- 하나의 impl block이 여러 책임을 지는가?
- struct 필드와 impl 로직이 응집되어 있는가?
- 도메인 규칙이 usecase에 흩어져 있는가?
- trait 경계가 명확한가?

개선:
- 책임별 struct 분리
- 도메인 로직을 domain layer로 이동
- trait 추출로 의존성 역전
- composition 우선 (Rust는 상속이 없다)

---

### 3.2.5 조건문 리팩토링

다음을 우선 제거한다.

- 거대한 if/else
- match 과다 분기
- 상태 기반 분기

대체:
- Polymorphism (trait 활용)
- Strategy Pattern
- State Pattern
- Lookup Table

---

### 3.2.6 데이터 구조 개선

다음을 제거한다.

- Primitive Obsession
- Magic Number
- Stringly Typed 구조

대체:
- Value Object
- Enum
- Domain Type (`Id<T>` Newtype, `enum Status`)

---

### 3.2.7 에러 처리 리팩토링

체크:
- `unwrap()` / `expect()` 남용 여부
- 의미 없는 에러 메시지 (`anyhow!("something wrong")`)
- 레이어 간 에러 타입 누출 여부
- `thiserror` (라이브러리) / `anyhow` (바이너리 main) 사용 맥락 혼용 여부

개선:

```rust
// Before
let val = risky_operation().unwrap();

// After
let val = risky_operation().map_err(AppError::Internal)?;
```

레이어별 에러 변환 경계 확립:

```text
infra:      sqlx::Error      → RepositoryError
usecase:    RepositoryError  → UsecaseError
controller: UsecaseError     → AppError (HTTP 응답용)
```

`expect()`는 진입 불가능한 상태임을 증명할 수 있을 때만 허용하며, 이유를 주석으로 명시한다.

```rust
// 컴파일 타임에 유효성이 보장된 리터럴이므로 항상 유효
let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("always valid literal");
```

---

### 3.2.8 로깅 리팩토링

체크:
- format string 방식 로깅 사용 여부
- context 정보 (request_id, user_id) 누락 여부
- `.ok()` 남용으로 에러가 삼켜지는지 여부

개선:

```rust
// Before (code smell)
error!("authorization failed: {:?}", err);

// After: tracing 필드 기반 구조화 로깅
error!(error = ?err, user_id = %user.id, "authorization failed");
```

---

### 3.2.9 Dependency 정리

체크:
- 순환 참조
- 숨겨진 의존성
- 테스트 어려운 구조
- 전역 상태 사용

개선:
- Dependency Injection (constructor injection 우선)
- Trait Boundary 분리
- Layer 명확화

---

### 3.2.10 Dead Code 제거

제거 대상:
- 사용되지 않는 함수 / struct / trait
- obsolete feature flag
- obsolete comment
- commented-out code

---

### 3.2.11 Comments 관리

**제거 대상** — 코드 자체로 이해 가능한 것:
- 내부 구현 "what" 주석
- 자명한 단계 설명 주석
- commented-out code

**유지 / 추가 대상** — 반드시 남겨야 할 것:
- `pub fn` / `pub struct` / `pub trait` 의 `///` doc 주석 (필수)
- `unsafe` 블록의 `// SAFETY:` 주석 (필수)
- 비자명한 제약·불변식을 설명하는 주석 (숨겨진 이유, 외부 버그 우회 등)

```rust
// ✓ 유지: pub 항목 doc 주석
/// 완료 처리. 이미 Done 상태이면 Err 반환.
pub fn complete(&mut self) -> Result<(), DomainError> { ... }

// ✗ 제거: 자명한 구현 주석
// status를 Done으로 설정
self.status = TodoStatus::Done;
```

---

### 3.2.12 가시성(pub) 범위 점검

(coding-style.md §2, §3 연계)

체크:
- `pub`이 외부 공개가 실제 필요한 경우에만 사용되는가?
- 내부 공유 시 `pub(crate)` / `pub(super)` 로 범위를 제한했는가?
- `pub fn` / `pub struct` / `pub trait` 에 `///` doc 주석이 존재하는가?

개선:

```rust
// Before: 불필요한 pub
pub fn internal_helper() { ... }

// After: 범위 제한
pub(crate) fn internal_helper() { ... }
```

---

### 3.2.13 API / DTO Boundary 점검

(coding-style.md §14 연계)

체크:
- request/response DTO가 domain model과 분리되어 있는가?
- validation이 controller boundary에서 수행되는가?
- domain model이 그대로 API response로 노출되지 않는가?
- serde 관련 derive가 domain이 아닌 DTO에 있는가?

개선:

```rust
// Before: domain model을 직접 직렬화
#[derive(Serialize)]
pub struct Todo { ... }

// After: DTO 분리
#[derive(Deserialize, Validate)]
pub struct CreateTodoRequest { ... }

#[derive(Serialize)]
pub struct TodoResponse { ... }
```

---

### 3.2.14 Usecase / Controller 책임 분리 점검

(coding-style.md §8 연계)

체크:
- controller에 business logic이 있는가?
- usecase가 HTTP 타입(`Json<T>`, `axum::Extension`)을 직접 사용하는가?
- 하나의 usecase가 두 가지 이상의 비즈니스 의도를 표현하는가?
- transaction boundary를 usecase가 소유하는가?

개선:

```rust
// Before: framework coupled usecase
async fn execute(Json(req): Json<CreateTodoRequest>) { ... }

// After: Command Object로 분리
async fn execute(command: CreateTodoCommand) { ... }
```

---

### 3.2.15 Async / Concurrency 패턴 점검

(coding-style.md §12 연계)

체크:
- 멀티 스레드 공유 상태를 `Arc`로 명시적으로 관리하는가?
- 불필요한 mutable 전역 상태가 있는가?
- async가 I/O boundary 이외에 남용되는가?
- `std::sync::Mutex` 대신 `tokio::sync::Mutex`를 써야 할 곳이 있는가?

개선:

```rust
// Before: 공유 상태 불명확
struct Service {
    cache: HashMap<String, Value>,
}

// After: Arc로 명시적 공유
struct Service {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}
```

---

### 3.2.16 Authentication & Middleware 점검

(coding-style.md §16 연계)

체크:
- JWT parsing / 인증 로직이 각 handler에 반복되는가?
- 인증 실패가 암묵적으로 처리되는가?
- request context에 사용자 정보가 적절히 주입되는가?

개선:

```rust
// Before: handler마다 인증 로직 반복
async fn get_todo(req: Request) {
    let token = req.headers().get("Authorization")...;
    // JWT parse 반복
}

// After: middleware에서 처리, context 주입
async fn auth_middleware(req: Request, next: Next) -> Response {
    let user = verify_token(&req)
        .map_err(|_| AppError::Unauthorized)?;
    req.extensions_mut().insert(user);
    next.run(req).await
}
```

---

### 3.2.17 문서화 점검

(coding-style.md §15 연계)

체크:
- `pub fn` / `pub struct` / `pub trait` 에 `///` doc 주석이 존재하는가?
- `unsafe` 블록에 `// SAFETY:` 주석이 있는가?
- API schema(utoipa)가 구현과 일치하는가?

개선:

```rust
// Before: doc 주석 누락
pub fn complete(&mut self) -> Result<(), DomainError> { ... }

// After
/// 완료 처리. 이미 Done 상태이면 `DomainError::InvalidTransition` 반환.
pub fn complete(&mut self) -> Result<(), DomainError> { ... }
```

---

## 3.3 Verification & Cleanup

### 3.3.1 Linter & Formatter 실행

아래를 순서대로 실행하고 결과를 개선한다.

```bash
cargo clippy -- -D warnings          # 경고를 오류로 처리
cargo clippy --fix --allow-dirty     # 자동 수정 가능한 항목 수정
cargo fmt                            # 포맷 자동 적용
cargo fmt --check                    # 포맷 위반 확인 (CI용)
```

---

### 3.3.2 전체 테스트 실행

반드시 수행:
- Unit Test
- Integration Test
- E2E Test
- Regression Test

실패 시:
- 원인 분석
- behavior change 여부 확인

테스트 수행 후 `/test-align` 명령을 실행하고 피드백을 자동 수정한다.

```bash
cargo test --all
cargo tarpaulin --out Html --output-dir coverage/   # 커버리지 확인
```

---

### 3.3.3 Security Scan

1. `/security-full-scan` 명령을 실행해 정적 분석을 진행하고 결과 피드백을 반영한다.
2. `/security-scan` 명령으로 동적 분석을 진행한다. 서버가 구동되지 않은 경우 `cargo run`으로 서버를 실행한 뒤 `/security-scan`을 다시 실행한다.

위 명령을 수행하기 위해서는 [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다. 설치가 안되어 있을 경우 인간에게 설치를 안내한다.

```bash
cargo audit                          # 의존성 보안 취약점 확인
```

---

### 3.3.4 Static Analysis 수행

확인:
- unused code
- nullability
- race condition
- unreachable code
- complexity 증가 여부

---

### 3.3.5 Architecture Review

검증:
- `controller → usecase → domain ← infra` 의존 방향 유지
- domain이 axum / sqlx를 import하지 않는가?
- usecase가 SQL을 직접 사용하지 않는가?
- infra가 domain trait를 구현하는가?
- Cargo workspace 경계가 잘못된 의존을 물리적으로 차단하는가?
- 도메인 경계 / Aggregate 경계 유지
- Transaction boundary 위치 (usecase가 소유하는가?)
- validation 로직이 controller 경계에 있는가?

---

### 3.3.6 Complexity Review

측정:
- Cyclomatic Complexity
- Cognitive Complexity
- Method Length
- Struct / Impl Size
- Dependency Depth

리팩토링 후 감소했는지 확인한다.

---

### 3.3.7 Diff Review

확인:
- 불필요한 formatting noise 제거
- 기능 변경 섞이지 않았는가?
- rename-only commit 분리 가능한가?

---

# 4. 리팩토링 피드백 작성 가이드라인

## 4.1 리팩토링 피드백 분류

위 체크사항들을 모두 점검하고 실제 작업단위를 나누어수 래팩토링 작업을 한 결과에 대해 아래 4가지 카테고리로 분류한다.

| 분류 | 의미 | 대응 |
|---|---|---|
| 🚫 Blocking Refactoring Issues | 반드시 수정이 필요한 구조적 문제 | 다음 단계 진행 전 필수 수정 |
| ⚠️ Recommended Refactoring Changes | 유지보수성과 안정성을 위한 권장 개선 | 가능하면 현재 작업에서 반영 |
| 💡 Refactoring Suggestions | 선택적 개선 아이디어 및 리팩토링 기회 | 향후 개선 후보로 고려 |
| 📝 Refactoring Tech Debt | 현재 수정 범위를 넘어서는 구조적 부채 | 별도 이슈로 추적 |

## 4.2 리팩터링 피드백 분류 기준

**1. 🚫 Blocking Refactoring Issues**

반드시 수정해야 하는 항목:

- behavior change 발생 가능성
- public API compatibility 문제
- transaction boundary 손상
- validation/auth logic 누락
- concurrency 문제 가능성
- rollback 어려움
- architecture rule 위반
- cyclic dependency 생성
- 테스트 실패
- security regression
- data corruption 가능성
- excessive complexity 증가
- unsafe refactoring

**2. ⚠️ Recommended Refactoring Changes**

강하게 권장되는 개선 사항:

- long method 개선 가능
- large struct/impl 분리 필요
- duplicate code 존재
- dependency direction 개선 가능
- testability 부족
- naming 품질 개선 필요
- unnecessary abstraction 존재
- excessive nesting
- readability 저하
- weak encapsulation
- magic number/string 존재
- excessive mutable state

**3. 💡 Refactoring Suggestions**

선택적 개선 아이디어:

- future extensibility 개선 가능
- reusable abstraction 가능
- performance optimization 여지
- domain model refinement 가능
- modern language idiom 적용 가능
- helper/util extraction 가능
- builder/factory 적용 가능
- async optimization 가능

**4. 📝 Refactoring Tech Debt**

현재 범위를 넘는 기술 부채:

- legacy architecture limitation
- monolith boundary 문제
- outdated framework dependency
- missing integration test infra
- migration 필요한 구조
- global state architecture
- insufficient observability
- low test coverage
- inconsistent domain modeling
- duplicated business logic across modules

## 4.3 리팩토링 결과 피드백 가이드라인

리팩토링 결과 피드백은 코드 냄새 유형 + Before/After 비교 형식으로 작성한다:

- 코드 위치: 파일명과 라인 번호를 명시 (예: `src/domain/order/service.rs:42`)
- 냄새 유형: 3.1.4 Code Smell 분석의 유형을 표시
- 문제 설명(Problem): 왜 문제가 되는가, 어떤 위험이 있는가, 어떤 영향(API 영향, transaction 영향, concurrency 영향, rollback risk, migration 필요 여부)이 있는지 구체적으로 기술
- 개선 방향(Recommendation): refactoring strategy, pattern, extraction 방향, dependency 개선 방향을 포함
- Before/After: 리팩토링 전후 코드 예시 제공
- 우선순위: 각 항목의 우선순위 (🚫/⚠️/💡/📝) 명시

---

# 5. PR 준비 및 제출

## 5.1 PR 제목 규칙

```
refactor([모듈명]): [핵심 변경 내용 한 줄 요약]
```

## 5.2 PR 본문

주요 변경 사항 단락에 리팩토링 결과 피드백의 내용 전체를 간략한 버전으로 정리해서 기술한다.

검증 내용에 테스트 결과와 security scan 결과를 기술한다.

## 5.3 최종 완료 조건

다음을 만족해야 완료로 간주한다.

- 모든 테스트 통과
- lint 0 warning
- security scan 통과
- dead code 제거 완료
- backward compatibility 확인
- PR 설명 최신 상태 유지
- 작은 단위 commit 유지
- rollback 가능 상태 확보
