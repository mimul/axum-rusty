---
name: Axum-Rusty Style Guide & Coding Standards
description: Rust 도메인 중심 코딩 원칙. 소유권·네이밍·설계·경계 조건·성능 등 Rust 고유 관례를 포함한 코딩 스타일 가이드.
---

# Axum-Rusty Style Guide & Coding Standards

단순히 Axum으로 API를 만든 프로젝트가 아니라, Rust의 타입 시스템을 활용해 Clean Architecture 사상을 코드의 구조(계층)에 반영하고 DDD는 도메인 모델링과 비즈니스 로직에 집중하여 서로의 약점을 보완해 유지보수성과 확장성이 뛰어난 시스템을 구축하는 것이 목표다.

특정 문법 스타일만 정의하는 문서가 아니라:

- 왜 이런 구조를 선택했는가
- 어떤 사고 흐름으로 코드를 작성해야 하는가
- 어떤 변경 비용을 줄이려 하는가
- Rust 타입 시스템을 어떻게 설계 제약으로 사용하는가
- AI-assisted coding 시대에 어떻게 구조 일관성을 유지할 것인가

까지 포함한다.

핵심 철학은 다음과 같다.

```text
Domain First
> Architecture First
> Explicitness
> Readability
> Simplicity
> Changeability
> Consistency
```

즉:

- framework보다 domain
- runtime보다 compile-time restriction
- cleverness보다 readability
- hidden magic보다 explicit flow
- 복잡성보다 유지보수 가능성

을 우선한다.

---

# 1. Architecture First

Rust의 타입 시스템을 활용해 Clean Architecture의 의존 방향을 컴파일 타임에 강제하려는 프로젝트다. 핵심은 "기능 구현"보다 **"의존 구조 유지"** 가 우선이라는 점이다.

## 1.1 Layer Responsibility

| Layer | 책임 |
|---|---|
| controller | HTTP 요청/응답, validation, serialization |
| usecase | 애플리케이션 유스케이스 orchestration   |
| domain | 핵심 도메인 모델과 비즈니스 규칙  |
| infra | DB/외부 시스템 |

각 레이어는 자신보다 상위 관심사를 몰라야 한다. domain은 axum을 몰라야 하고, infra는 HTTP 응답 타입을 몰라야 하고, usecase는 SQL을 직접 몰라야 한다.

좋은 예:

```rust
pub struct TodoUseCase<R: RepositoriesModuleExt> {
    repositories: Arc<R>,
}
```

나쁜 예:

```rust
pub struct TodoUseCase {
    db: PgPool,
}
```

## 1.2 Dependency Direction

의존 방향은 항상 안쪽(domain)으로 향한다.

```text
controller -> usecase -> domain <- infra
```

infra는 domain trait를 구현한다.

```rust
#[async_trait]
pub trait TodoRepository {
    async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>;
}
```

```rust
impl TodoRepository for DatabaseRepositoryImpl<Todo> {
    async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>> {
        ...
    }
}
```

domain은 구현이 아니라 contract를 소유한다.

## 1.3 Cargo Workspace as Architecture Boundary

workspace는 단순 build 분리가 아니라 architecture boundary다.

```toml
todo-domain = { path = "../todo-domain" }
todo-infra = { path = "../todo-infra" }
```

usecase crate가 controller crate를 dependency에 추가하지 않았다면, 컴파일 레벨에서 잘못된 구조(의존 위반)를 어렵게 만든다.이 불가능하다. 

## 요약 체크리스트

- 레이어 책임이 명확한가?
- dependency direction이 유지되는가?
- domain이 framework를 모르는가?
- contract는 domain에 있는가?
- infra가 implementation을 소유하는가?
- workspace boundary가 존재하는가?

---

# 2. Domain First

DB 중심이 아니라 business meaning 중심으로 모델링한다.

## 2.1 Domain Model != Database Model

좋은 예:

```rust
pub enum TodoStatus {
    Todo,
    Doing,
    Done,
}
```

나쁜 예:

```rust
status_id: String
```

`status_id`는 DB concern이다. 도메인은 상태 의미를 가져야 한다.


## 2.2 Rich Domain over Primitive Obsession

문자열(String) 남용을 줄인다.

좋은 예:

```rust
pub struct Id<T> {
    value: Uuid,
}
```

나쁜 예:

```rust
id: String
```

강한 타입은 컴파일 타임 안정성을 높인다. 특히 Rust에서는 타입 설계가 곧 validation 전략이다.

## 2.3 Explicit Conversion

레이어 간 객체 전달은 명시적으로 변환한다.

```rust
impl TryFrom<StoredTodo> for Todo
```

장점:

* 레이어 경계가 명확해진다.
* 암묵적 coupling이 줄어든다.
* 변환 시 validation을 넣을 수 있다.
* persistence 변경 영향이 줄어든다.

## 요약 체크리스트

- business meaning을 표현하는가?
- primitive obsession을 줄였는가?
- invalid state를 타입으로 제거하는가?
- explicit conversion이 존재하는가?
- DB concern이 domain으로 새지 않는가?

---

# 3. Explicit & Intentional Code

Rust에서는 명시성이 verbosity가 아니라 안정성이다. 암묵성보다 명시성을, 마법보다 의도를, 축약보다 이해 가능성을 우선한다.


## 3.1 Prefer Explicit Data Flow

좋은 예:

```rust
let todo = repository
    .find_by_id(todo_id)
    .await?;

todo.complete()?;

repository.save(todo).await?;
```

나쁜 예:

```rust
process(todo_id).await?;
```

## 3.2 Avoid Hidden Mutation

좋은 예:

```rust
todo.complete()?;
```

나쁜 예:

```rust
update(todo)?;
```

## 3.3 Intent-Revealing Naming

좋은 예:

```rust
complete_todo()
validate_access_token()
```

나쁜 예:

```rust
process()
handle()
run()
```

## 3.4 Explicit Error Handling

좋은 예:

```rust
match token.verify() {
    Ok(claims) => claims,
    Err(_) => return Err(AppError::Unauthorized),
}
```

나쁜 예:

```rust
token.verify().unwrap()
```

## 요약 체크리스트

- 데이터 흐름이 명확한가?
- hidden behavior가 없는가?
- 함수명이 의도를 설명하는가?
- 상태 변화가 드러나는가?
- error handling이 explicit한가?

---

# 4. Readability as Primary Quality

읽기 어려운 코드는 유지보수 불가능한 코드다. Readability(Cleverness, Brevity)를 우선한다.

## 4.1 Prefer Linear Flow

좋은 예:

```rust
let user = repo.find(id).await?;

authorize(&user)?;

user.activate();

repo.save(user).await?;
```

## 4.2 Avoid Deep Nesting

좋은 예:

```rust
let user = user.ok_or(AppError::NotFound)?;

if !user.active {
    return Err(AppError::Forbidden);
}
```

## 4.3 Prefer Small Functions

좋은 함수:

```rust
create_todo()
authorize_user()
```

나쁜 함수:

```rust
process_request()
```

## 4.4 Avoid Macro Abuse

macro는 readability를 해치지 않는 범위까지만 사용한다.

## 요약 체크리스트

- 읽기 쉬운가?
- control flow가 단순한가?
- 함수 역할이 명확한가?
- nesting depth가 낮은가?
- cleverness보다 clarity를 우선하는가?

---

# 5. Complexity Control & Simplicity

복잡성은 가장 큰 유지보수 비용이다. 추상화는 중복 제거보다 이해 비용 감소를 위해 존재해야 한다.

## 5.1 Avoid Premature Abstraction

좋은 예:

```rust
TodoService
UserService
```

나쁜 예:

```rust
trait BaseCrudService<T>
```


## 5.2 Prefer Composition over Hierarchy

좋은 예:

```rust
struct TodoService {
    repository: Arc<dyn TodoRepository>,
}
```

## 5.3 Reduce Cognitive Load

과도한 generic/type-level abstraction을 피한다.

## 5.4 Prefer Small Modules

작고 응집도 높은 module을 선호한다.

## 요약 체크리스트

- abstraction이 과하지 않은가?
- hierarchy가 깊지 않은가?
- composition을 사용하는가?
- cognitive load가 낮은가?
- 단순한 구조를 유지하는가?

---

# 6. Changeability & Refactoring

좋은 구조의 핵심은 미래 변경 비용 감소다.

## 6.1 Stable Boundary

- repository trait
- usecase contract
- API schema

같은 stable boundary를 유지한다.

## 6.2 Separate Business Logic from Framework

framework는 바깥쪽 concern이다.

```text
domain
application
infra
presentation
```

## 6.3 Avoid Framework Coupling

좋은 예:

```rust
execute(command: CreateTodoCommand)
```

나쁜 예:

```rust
execute(Json(req): Json<CreateTodoRequest>)
```

## 6.4 Refactor-Friendly Structure

- small module
- isolated concern
- low coupling
- explicit contract

를 유지한다.

## 요약 체크리스트

- framework coupling이 낮은가?
- stable boundary가 존재하는가?
- concern separation이 되는가?
- 리팩토링 가능한 구조인가?
- business logic이 독립적인가?

---

# 7. Consistency & Predictability

일관성은 cognitive load를 줄이는 가장 강력한 도구다.

## 7.1 Consistent Naming

```rust
find_by_id()
find_all()
save()
delete()
```

## 7.2 Consistent Module Structure

```text
domain/
application/
infra/
presentation/
```

## 7.3 Consistent Error Strategy

```rust
Result<T, AppError>
```

패턴을 프로젝트 전반에서 일관되게 유지한다.

## 7.4 Predictable Behavior

validation/auth/logging 패턴을 일관되게 유지한다.

## 요약 체크리스트

- naming이 일관적인가?
- module structure가 predictable한가?
- error strategy가 통일되어 있는가?
- behavior consistency가 있는가?
- cognitive load를 줄이는가?

---

# 8. Usecase Oriented Design

usecase는 business workflow orchestration layer다.

## 8.1 Thin Controller

controller는:

- request parsing
- validation
- usecase 호출
- response serialization

만 담당한다.

## 8.2 Usecase Owns Workflow

workflow ordering과 transaction boundary를 usecase가 소유한다.

## 8.3 One Usecase, One Intention

좋은 예:

```rust
create_todo()
complete_todo()
```

## 요약 체크리스트

- controller가 thin한가?
- business logic이 controller에 없는가?
- usecase가 workflow를 소유하는가?
- intention-revealing naming을 사용하는가?

---

# 9. Dependency Injection

Rust에서는 explicit DI를 선호한다.

## 9.1 Constructor Injection

```rust
TodoUseCase::new(repositories)
```

장점:

* dependency가 명시적이다
* 테스트가 쉽다
* hidden dependency가 없다
* compile-time validation 가능

## 9.2 Composition Root

Modules가 dependency assembly를 담당한다.

## 9.3 Prefer Trait Boundary

구현보다 trait에 의존한다.

이는:

* 테스트 mock 교체
* 구현 변경
* infra 교체
* feature 확장

을 쉽게 만든다.

## 요약 체크리스트

- dependency가 explicit한가?
- constructor injection을 사용하는가?
- composition root가 존재하는가?
- trait boundary를 사용하는가?

---

# 10. Error Handling

Rust에서는 panic보다 Result 기반 흐름이 기본이다. anyhow + thiserror 조합을 사용한다.

## 10.1 Avoid unwrap/expect Abuse

recoverable error에서 unwrap 사용을 피한다.

## 10.2 Preserve Meaningful Error

에러는 의미를 보존해야 한다.

좋은 예:

```rust
AppError::InvalidJwt
```

나쁜 예:

```rust
anyhow!("something wrong")
```

## 10.3 Error Boundary

layer boundary에서 error abstraction을 변환한다.

예:

* sqlx error → infra
* domain error → usecase
* http error → controller

각 레이어는 자신이 이해 가능한 error abstraction만 알아야 한다.

## 요약 체크리스트

- panic 남용을 피하는가?
- meaningful error를 사용하는가?
- layer별 error boundary가 존재하는가?

---

# 11. Type-Driven Design

Rust에서는 타입 설계가 곧 설계 문서다. trait, generic, enum을 적극 활용한다.

## 11.1 Encode Rule in Type

좋은 예:

```rust
Option<TodoStatus>
```

이는 상태가 없을 수 있다는 의미를 타입으로 표현한다.

나쁜 예:

```rust
String
```
## 11.2 Prefer Enum over String

좋은 예:

```rust
pub enum TodoStatus {
    Todo,
    Doing,
    Done,
}
```

나쁜 예:

```rust
status: String
```

## 11.3 Generic with Restraint

generic은 readability를 해치지 않는 범위까지만 사용한다.

## 요약 체크리스트

- 타입이 business rule을 표현하는가?
- enum을 적극 활용하는가?
- invalid state를 제거하는가?
- generic complexity를 통제하는가?

---

# 12. Async & Concurrency

Rust async는 단순 성능 최적화가 아니라 ownership 기반 concurrency safety를 제공한다.

## 12.1 Shared State with Arc

```rust
Arc<Pool<Postgres>>
```

## 12.2 Minimize Mutable State

immutable flow를 선호한다. mutable state는 concurrency complexity를 급격히 증가시킨다.

## 12.3 Async for I/O Boundary

I/O boundary에만 async를 집중한다.

## 요약 체크리스트

- shared state를 explicit하게 관리하는가?
- mutable state를 최소화했는가?
- async가 I/O 중심인가?

---

# 13. Database & Repository

Repository는 persistence abstraction이다.

## 13.1 Hide Persistence Detail

domain은 SQL을 몰라야 한다.

## 13.2 Infra Owns Query

SQL concern은 infra에 존재한다.

## 13.3 Transaction Boundary in Usecase

workflow transaction을 usecase가 orchestration한다.

## 요약 체크리스트

- persistence detail이 숨겨져 있는가?
- domain이 SQL을 모르는가?
- transaction boundary가 명확한가?

---

# 14. API Design

HTTP API는 public contract다.

## 14.1 Explicit DTO

request/response model을 명시적으로 분리한다.

```rust
#[derive(Deserialize, Debug, Validate)]
pub struct JsonCreateTodo {
    ...
}
```

## 14.2 Validation at Boundary

잘못된 입력이 domain까지 들어가지 않게 한다.

```rust
#[validate(length(min = 1))]
pub title: Option<String>
```

## 14.3 Serialization is Controller Concern

serde model은 controller concern이다. domain model을 그대로 API response로 노출하지 않는다.

## 요약 체크리스트

- DTO가 분리되어 있는가?
- boundary validation을 수행하는가?
- transport concern이 controller에 머무는가?

---

# 15. Authentication & Middleware

인증은 cross-cutting concern이다. 따라서 middleware/AOP 형태로 분리한다.

## 15.1 Middleware-based Authentication

JWT parsing을 middleware로 분리한다. 모든 controller에서 JWT parsing을 반복하지 않는다.

## 15.2 Context Injection

request context에 사용자 정보를 주입한다. 이는 controller를 단순화한다.

```rust
req.extensions_mut().insert(current_user);
```

## 15.3 Explicit Security Failure

인증 실패는 명확하게 처리한다.

좋은 예:

```rust
return Err(InvalidJwt("auth_header not found".to_string()));
```

보안 로직은 “암묵적 fallback”이 없어야 한다.

## 요약 체크리스트

- 인증이 middleware로 분리되어 있는가?
- auth duplication이 없는가?
- security failure가 explicit한가?

---

# 16. Observability & Logging

운영 가능한 시스템은 tracing 가능해야 한다.

## 16.1 Structured Logging

```rust
error!("authorization failed: {:?}", err);
```

## 16.2 Log with Context

- request id
- user id
- correlation id

등 context를 포함한다.

## 16.3 Never Swallow Error

에러를 삼키지 않는다.

```rust
.ok();
```

남용을 피한다.

## 요약 체크리스트

- structured logging을 사용하는가?
- context-rich log를 남기는가?
- error visibility를 유지하는가?

---

# 17. Readability First

좋은 코드는 clever한 코드가 아니라 읽기 쉬운 코드다. Rust에서는 특히 type complexity와 lifetime complexity가 readability를 해치기 쉽다.

## 17.1 Prefer Explicitness

좋은 예:

```rust
match stored_todo {
    Some(st) => Ok(Some(st.try_into()?)),
    None => Ok(None),
}
```

짧은 코드보다 명확한 코드가 우선이다.

## 17.2 Small Scope

변수 scope는 가능한 좁게 유지한다. 이는 ownership/lifetime 문제를 줄인다.

## 17.3 Naming Matters

좋은 이름은 설명 주석보다 강력하다.

좋은 예:

```rust
authorize_current_user
```

나쁜 예:

```rust
handle_auth
process_user
```

handle/process/do 같은 이름은 의미가 약하다.

## 요약 체크리스트

* 짧음보다 명확함을 우선하는가?
* 변수 scope가 작은가?
* 함수명이 intention을 드러내는가?
* generic naming을 피하는가?
* type complexity를 관리하고 있는가?

---

# 18. Testing Philosophy

테스트는 implementation verification이 아니라 business behavior verification이다.

## 18.1 Test Business Rule

좋은 테스트:

```text
완료된 Todo는 다시 완료할 수 없다
```

mock interaction보다 business outcome이 중요하다.

## 18.2 Prefer DI for Testability

DI 구조는 테스트 가능성을 높인다.

## 18.3 Test Proper Layer

| Layer | 테스트 목적 |
|---|---|
| domain | business rule |
| usecase | workflow |
| infra | integration |
| controller | contract |

## 요약 체크리스트

- business behavior를 테스트하는가?
- implementation coupling을 줄였는가?
- 레이어별 테스트 목적이 명확한가?

---

# 19. Documentation & API Schema

문서는 나중 작업이 아니라 설계 일부다. utoipa 기반 OpenAPI 문서화를 사용한다.

## 19.1 Schema as Contract

API schema를 코드와 함께 유지한다.

## 19.2 Documentation Close to Code

문서가 코드 근처에 있어야 drift가 줄어든다.

## 19.3 Self-Describing API

명확한 schema/model naming을 사용한다.

## 요약 체크리스트

- API schema가 유지되는가?
- 문서 drift를 줄이고 있는가?
- self-describing naming을 사용하는가?

---

# 20. AI Coding Style Alignment

이 프로젝트는 AI-assisted coding 시대에도 유지 가능한 구조를 목표로 한다. AI가 코드를 생성하더라도 architecture consistency가 유지되어야 한다.

## 20.1 Enforce Structure over Prompting

"좋은 코드를 써줘"보다:

* layer boundary
* trait abstraction
* DTO separation
* dependency direction

같은 구조적 제약이 더 중요하다.

## 20.2 Make Invalid Architecture Hard

Rust의 강점은 compile-time restriction이다. workspace dependency, trait boundary, ownership system을 활용해 잘못된 구조를 어렵게 만든다.

## 20.3 Readability for Humans and AI

AI도 결국 기존 코드 패턴을 학습한다.

따라서:

* naming consistency
* folder consistency
* explicit boundary
* predictable patterns

이 중요하다. 일관성이 높은 프로젝트일수록 AI assistance 품질도 좋아진다. ([arxiv.org](https://arxiv.org/abs/2403.14986))


## 요약 체크리스트

- architecture consistency가 유지되는가?
- invalid architecture를 어렵게 만들었는가?
- predictable pattern이 존재하는가?
- AI-friendly structure인가?

---

# Final Principles

1. Framework보다 Domain이 우선이다.
2. Runtime보다 Compile-time restriction을 선호한다.
3. 구현보다 의존 구조를 중요하게 본다.
4. Smart code보다 Readable code를 선호한다.
5. 암묵성보다 명시성을 선호한다.
6. 단순성이 유지보수성을 만든다.
7. 변경 용이성이 좋은 설계의 핵심이다.
8. 일관성은 cognitive load를 줄인다.
9. 타입 시스템으로 invalid state를 제거한다.
10. AI 시대일수록 predictable architecture가 중요하다.
