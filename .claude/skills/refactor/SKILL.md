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

  실행 흐름 (6단계):
    STEP 0 사전 조건   브랜치 확인·빌드+테스트 baseline
    STEP 1 준비        목표·옵션·scope 정의
    STEP 2 범위 식별   변경 범위·기존 테스트 확인
    STEP 3 일괄 점검   coding-style.md §1~19 체크리스트·Code Smell 분석(Rust 특화 포함)
    STEP 4 전략 수립   리스크 분석(High/Medium/Low 분기)·goal×level→항목 매핑으로 전략 선택
    STEP 5 수행        선택 항목 순서대로 리팩토링→테스트→커밋 반복
                       브랜치: feature/refactor-{모듈명} / --dry-run 시 이 단계 없이 종료
    STEP 6 검증        cargo clippy --fix→-D warnings→fmt → cargo test --all →
                       /security-full-scan(옵션)→ 아키텍처 리뷰→ complexity 측정→ diff 검토

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

# `/refactor` 커맨드 스킬

본 문서는 Axum Rusty 프로젝트의 코딩 철학과 표준을 반영한 리팩토링 실행 절차 및 지침을 제공한다.

## 커맨드 문법

권장 명령은 아래 수준으로 단순하게 유지한다.

```bash
/refactor [scope]
```

필요 시에만 아래 옵션을 추가한다.

```bash
--goal
--level
--with-tests
--with-security
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

`--with-tests` 옵션이 있으면 Claude는 6.2.2 `/test-align` 명령을 수행해 테스트 갭 분석 및 보완을 진행한다.

## 보안 정책

`--with-security` 옵션이 있으면 Claude는: 6.3 Security Scan을 수행해 보안 갭 분석 및 보완을 진행한다.

**전제 조건으로 [claude-security-scan](https://github.com/mimul/claude-security-scan)** 이 설치되어야 한다.

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

- 사전 조건 체크 (STEP 0)
- Refactoring 준비 (STEP 1)
- 변경 범위 식별과 기존 테스트 확인 (STEP 2)
- coding-style.md 기준 일괄 점검 (STEP 3)
- 리스크 분석과 리팩토링 전략 선택 (STEP 4)
- 리팩토링 수행 (STEP 5)
- Verification & Cleanup (섹션 6)
- 피드백 작성 가이드라인 (섹션 7)
- PR 준비 정책 (섹션 8)

을 자동으로 적용한다.

---

# STEP 0 사전 조건 체크

리팩토링 시작 전 아래를 순서대로 확인한다. 하나라도 실패하면 사용자에게 보고하고 중단한다.

```bash
git fetch origin && git status && git log --oneline -5 && git log --oneline origin/main -5  # 최신 브랜치 확인
cargo build                 # 빌드 통과 여부 확인
cargo test --all            # 기존 테스트 baseline 확인
```

확인 항목:
- [ ] 최신 브랜치 확인
- [ ] `cargo build` 통과
- [ ] `cargo test --all` 통과 (리팩토링 전 baseline 확보)

scope가 명시되지 않아 프로젝트 전체가 대상인 경우:
파일 수와 예상 작업량을 사용자에게 보고하고 계속 진행 여부를 확인한다.

# STEP 1 Refactoring 준비

커맨드 옵션을 파악해 리팩토링의 목표를 명확히 정의한다.

**STEP 1 산출물** (다음 단계로 전달):
- 적용 goal: `<readability|maintainability|testability|domain-model|complexity>` (기본값: `readability`)
- 적용 level: `<safe|moderate|aggressive>` (기본값: `safe`)
- scope: `<전체|모듈명|파일경로>`
- 활성 옵션: `--with-tests` / `--with-security` / `--dry-run` 여부

# STEP 2 변경 범위 식별과 기존 테스트 확인

## 2.1 변경 범위 식별

STEP 1의 scope를 기반으로 아래 관점에서 영향 범위를 식별한다.

- 영향받는 모듈
- 의존 관계
- API 계약
- 데이터 흐름
- 상태 변경 지점
- 트랜잭션 경계
- 동시성 영향
- 외부 시스템 영향

## 2.2 기존 테스트 확인

변경 범위 내 테스트 현황을 확인한다.

```bash
cargo test --all 2>&1 | tail -20   # 테스트 통과 현황
```

테스트가 부족한 경우:
- behavior preserving verification을 위해 characterization test를 먼저 작성한다.
- `--with-tests` 옵션이 없더라도, 테스트 전무 파일을 리팩토링할 때는 최소 smoke test를 확보한다.

# STEP 3 coding-style.md 기준 일괄 점검

## 3.1 `.claude/rules/coding-style.md`의 19개 섹션 체크리스트를 기준으로 전체 코드를 점검한다. 각 체크리스트 항목의 위반 사항을 식별한다.

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

## 3.2 Code Smell 분석

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

# STEP 4 리스크 분석과 리팩토링 전략 선택

## 4.1 리스크 분석

- Public API 변경 여부
- Backward Compatibility
- Migration 필요 여부
- 데이터 손상 가능성
- 성능 영향
- Lock/Concurrency 영향
- Security 영향

**리스크 대응 분기**:

| 리스크 수준 | 판단 기준 | 대응 |
|---|---|---|
| 🔴 High | Public API 파괴적 변경 / Migration 필요 / data 손상 가능 | 사용자에게 보고 후 중단. 별도 마이그레이션 계획 필요 |
| 🟡 Medium | 내부 API 변경 / 성능 영향 / concurrency 주의 | 사용자에게 보고 후 계속. 해당 항목에 별도 테스트 추가 |
| 🟢 Low | 명명 개선 / 로깅 포맷 / 가시성 조정 | 그대로 진행 |

## 4.2 리팩토링 전략 선택

STEP 1의 goal·level과 STEP 3의 위반 목록을 기반으로, 아래 표에서 수행할 항목과 순서를 결정한다.

**goal → 우선 수행 항목 매핑**:

| goal | 우선 수행 항목 (STEP 5에서 먼저 실행) |
|---|---|
| `readability` | 5.2 Naming → 5.3 함수 → 5.12 pub 범위 → 5.11 Comments |
| `maintainability` | 5.9 Dependency → 5.3 함수 → 5.4 Struct/Trait → 5.10 Dead Code |
| `testability` | 5.9 Dependency → 5.4 Struct/Trait → 5.14 Usecase 책임 → 5.3 함수 |
| `domain-model` | 5.6 데이터구조 → 5.4 Struct/Trait → 5.14 Usecase 책임 → 5.13 DTO Boundary |
| `complexity` | 5.5 조건문 → 5.3 함수 → 5.10 Dead Code → 5.4 Struct/Trait |

**level → 허용 범위**:

| level | 허용 범위 |
|---|---|
| `safe` | rename / extract method / 로깅 포맷 / pub 범위 조정. Public API 변경 금지 |
| `moderate` | 내부 struct 분리 / dependency 정리 / 내부 API 개선. Public API 변경 금지 |
| `aggressive` | architecture 개선 / domain restructuring / legacy 추상화 제거. behavior verification 필수 유지 |

# STEP 5 리팩토링 수행

STEP 3과 STEP 4의 분석 결과를 기반으로, STEP 4.2에서 선택한 항목과 순서로 리팩토링을 진행한다.

**실행 원칙**:
- 브랜치는 `feature/refactor-{모듈명 또는 작업 내용}` 형태로 만든다. (예: `feature/refactor-todo-domain`, `feature/refactor-error-handling`)
- 각 항목마다 리팩토링 → 테스트 → 커밋을 반복한다.
- 커밋 메시지는 `refactor(scope): 내용` 형태를 따른다. (CLAUDE.md 커밋 컨벤션)
- 기능 추가와 리팩토링을 같은 커밋에 혼합 금지.
- behavior change 발생 시 즉시 중단.

**`--dry-run` 옵션 시**: STEP 0~4 분석 결과를 출력하고, STEP 5 실행 없이 종료한다. 출력 형식: 발견된 위반 목록, 예상 수행 항목, 리스크 수준.

## 5.2 Naming 개선

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

## 5.3 함수 리팩토링

목표:
- 단일 책임
- 의도 중심
- Side Effect 최소화

체크:
- 함수가 여러 역할 수행하는가?
- 조건문이 과도한가?
- depth가 깊은가?
- mutable state가 많은가?

## 5.4 Struct / Impl / Trait 리팩토링

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

## 5.5 조건문 리팩토링

다음을 우선 제거한다.

- 거대한 if/else
- match 과다 분기
- 상태 기반 분기

대체:
- Polymorphism (trait 활용)
- Strategy Pattern
- State Pattern
- Lookup Table

## 5.6 데이터 구조 개선

다음을 제거한다.

- Primitive Obsession
- Magic Number
- Stringly Typed 구조

대체:
- Value Object
- Enum
- Domain Type (`Id<T>` Newtype, `enum Status`)

## 5.7 에러 처리 리팩토링

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

## 5.8 로깅 리팩토링

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

## 5.9 Dependency 정리

체크:
- 순환 참조
- 숨겨진 의존성
- 테스트 어려운 구조
- 전역 상태 사용

개선:
- Dependency Injection (constructor injection 우선)
- Trait Boundary 분리
- Layer 명확화

## 5.10 Dead Code 제거

제거 대상:
- 사용되지 않는 함수 / struct / trait
- obsolete feature flag
- obsolete comment
- commented-out code

## 5.11 Comments 관리

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

## 5.12 가시성(pub) 범위 점검

(coding-style.md §2, §3 연계)

체크:
- `pub`이 외부 공개가 실제 필요한 경우에만 사용되는가?
- 내부 공유 시 `pub(crate)` / `pub(super)` 로 범위를 제한했는가?

개선:

```rust
// Before: 불필요한 pub
pub fn internal_helper() { ... }

// After: 범위 제한
pub(crate) fn internal_helper() { ... }
```

## 5.13 API / DTO Boundary 점검

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

## 5.14 Usecase / Controller 책임 분리 점검

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

## 5.15 Async / Concurrency 패턴 점검

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

## 5.16 Authentication & Middleware 점검

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

## 5.17 문서화 점검

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

# 6 Verification & Cleanup

## 6.1 Linter & Formatter 실행

아래를 순서대로 실행하고 결과를 개선한다.

```bash
cargo clippy --fix --allow-dirty     # 자동 수정 가능한 항목 수정
cargo clippy -- -D warnings          # 잔존 경고 확인 (경고를 오류로 처리)
cargo fmt                            # 포맷 자동 적용
```

## 6.2 전체 테스트 실행

### 6.2.1 반드시 수행:

- Unit Test
- Integration Test
- E2E Test
- Regression Test

실패 시:
1. 원인 분석: 어느 변경이 테스트를 깨뜨렸는지 특정한다.
2. behavior change 판단:
   - **의도하지 않은 behavior change** → 해당 커밋을 `git revert`하고 STEP 5로 돌아간다.
   - **의도한 behavior change** → 사용자에게 보고하고 확인을 받은 뒤 테스트를 갱신한다.
3. 재실행: 수정 후 `cargo test --all`을 다시 실행해 통과를 확인한다.

### 6.2.2 `/test-align` 명령을 실행(옵션 : `--with-tests` 옵션이 있을 경우 수행함)

- characterization test
- regression test
- edge case test
- flaky test 개선

을 함께 수행한다.

테스트가 부족한 경우:
- 기존 동작을 먼저 캡처한다
- behavior preserving verification을 우선한다

```bash
cargo test --all
cargo tarpaulin --out Html --output-dir coverage/   # 커버리지 확인
```

## 6.3 Security Scan(옵션 : `--with-security` 옵션이 있을 경우 수행함)

1. `/security-full-scan` 명령을 실행해 정적 분석을 진행하고 결과 피드백을 반영한다.
2. `/security-scan` 명령으로 동적 분석을 진행한다. 서버가 구동되지 않은 경우 `cargo run`으로 서버를 실행한 뒤 `/security-scan`을 다시 실행한다.

```bash
cargo audit                          # 의존성 보안 취약점 확인
```

## 6.4 Static Analysis 수행

확인:
- unused code
- nullability
- race condition
- unreachable code
- complexity 증가 여부

## 6.5 Architecture Review

검증:
- `controller → usecase → domain ← infra` 의존 방향 유지
- domain이 axum / sqlx를 import하지 않는가?
- usecase가 SQL을 직접 사용하지 않는가?
- infra가 domain trait를 구현하는가?
- Cargo workspace 경계가 잘못된 의존을 물리적으로 차단하는가?
- 도메인 경계 / Aggregate 경계 유지
- Transaction boundary 위치 (usecase가 소유하는가?)
- validation 로직이 controller 경계에 있는가?

## 6.6 Complexity Review

측정:
- Cyclomatic Complexity
- Cognitive Complexity
- Method Length
- Struct / Impl Size
- Dependency Depth

리팩토링 후 감소했는지 확인한다.

## 6.7 Diff Review

확인:
- 불필요한 formatting noise 제거
- 기능 변경 섞이지 않았는가?
- rename-only commit 분리 가능한가?

# 7. 리팩토링 결과 피드백 작성 가이드라인

## 7.1 리팩토링 결과 피드백 분류

위 체크사항들을 모두 점검하고 실제 작업단위를 나누어수 래팩토링 작업을 한 결과에 대해 아래 4가지 카테고리로 분류한다.

| 분류 | 의미 | 대응 |
|---|---|---|
| 🚫 Blocking Refactoring Issues | 반드시 수정이 필요한 구조적 문제 | 다음 단계 진행 전 필수 수정 |
| ⚠️ Recommended Refactoring Changes | 유지보수성과 안정성을 위한 권장 개선 | 가능하면 현재 작업에서 반영 |
| 💡 Refactoring Suggestions | 선택적 개선 아이디어 및 리팩토링 기회 | 향후 개선 후보로 고려 |
| 📝 Refactoring Tech Debt | 현재 수정 범위를 넘어서는 구조적 부채 | 별도 이슈로 추적 |

## 7.2 리팩터링 결과 피드백 분류 기준

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

## 7.3 리팩토링 결과 피드백 가이드라인

리팩토링 결과 피드백은 코드 냄새 유형 + Before/After 비교 형식으로 작성한다:

- 코드 위치: 파일명과 라인 번호를 명시 (예: `src/domain/order/service.rs:42`)
- 유형: STEP 3.1 .claude/rules/coding-style.md의 19개 섹션 체크리스트를 기준 위반 및 STEP 3.2 Code Smell 분석의 유형을 표시
- 문제 설명(Problem): 왜 문제가 되는가, 어떤 위험이 있는가, 어떤 영향(API 영향, transaction 영향, concurrency 영향, rollback risk, migration 필요 여부)이 있는지 구체적으로 기술
- 개선 방향(Recommendation): refactoring strategy, pattern, extraction 방향, dependency 개선 방향을 포함
- Before/After: 리팩토링 전후 코드 예시 제공
- 우선순위: 각 항목의 우선순위 (🚫/⚠️/💡/📝) 명시

---

# 8. PR 준비 및 제출

## 8.1 PR 제목 규칙

```
refactor([모듈명]): [핵심 변경 내용 한 줄 요약]

예시:
  refactor(todo): complete() clone() 제거
  refactor(auth): handle_auth → validate_access_token 개명
```

## 8.2 PR 본문

주요 변경 사항 단락에 리팩토링 결과 피드백의 내용 전체를 간략한 버전으로 정리해서 기술한다.

검증 내용에 테스트 결과와 security scan 결과를 기술한다.

## 8.3 최종 완료 조건

다음을 만족해야 완료로 간주한다.

- 모든 테스트 통과
- lint 0 warning
- security scan 통과(`--with-security` 옵션이 있을 경우)
- dead code 제거 완료
- backward compatibility 확인
- PR 설명 최신 상태 유지
- 작은 단위 commit 유지
- rollback 가능 상태 확보
