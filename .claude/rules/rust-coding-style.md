---
name: rust-coding-style
description: Rust 도메인 중심 코딩 원칙. 소유권·네이밍·설계·경계 조건·성능 등 Rust 고유 관례를 포함한 코딩 스타일 가이드. /refactor-rust·/code-review-rust 스킬의 1차 판단 기준.
---

단순히 Axum으로 API를 만든 프로젝트가 아니라, Rust의 타입 시스템을 활용해 Clean Architecture 사상을 코드의 구조(계층)에 반영하고 DDD는 도메인 모델링과 비즈니스 로직에 집중하여 서로의 약점을 보완해 유지보수성과 확장성이 뛰어난 시스템을 구축하는 것이 목표다.

특정 문법 스타일만 정의하는 문서가 아니라, 왜 이런 구조를 선택했는지, 어떤 사고 흐름으로 코드를 작성해야 하는지까지 포함한다.

---

# 1. Architecture First

Rust의 타입 시스템을 활용해 Clean Architecture의 의존 방향을 컴파일 타임에 강제하려는 프로젝트다. 핵심은 “기능 구현”보다 "의존 구조 유지"가 우선이라는 점이다.

## 1.1 Layer Responsibility

레이어는 역할 중심으로 나눈다.

| Layer      | 책임                                    |
| ---------- | ------------------------------------- |
| controller | HTTP 요청/응답, validation, serialization |
| usecase    | 애플리케이션 유스케이스 orchestration            |
| domain     | 핵심 도메인 모델과 비즈니스 규칙                    |
| infra      | DB/외부 시스템 구현                          |

각 레이어는 자신보다 상위 관심사를 몰라야 한다.

예:

* domain은 axum을 몰라야 한다.
* infra는 HTTP 응답 타입을 몰라야 한다.
* usecase는 SQL을 직접 몰라야 한다.

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

위 나쁜 예는 usecase가 infra 구현(sqlx)에 직접 의존한다.

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

핵심은 domain이 “구현”이 아니라 “계약(contract)”을 소유한다는 점이다.

## 1.3 Cargo Workspace as Architecture Boundary

Rust workspace는 단순 build 분리가 아니라 architectural boundary 역할을 한다.

```toml
[dependencies]
todo-domain = { path = "../todo-domain" }
todo-infra = { path = "../todo-infra" }
```

usecase crate가 controller crate를 dependency에 추가하지 않았다면, 컴파일 레벨에서 의존 위반이 불가능하다.

## 요약 체크리스트

* 레이어 책임이 명확한가?
* domain이 framework를 모르고 있는가?
* usecase가 infra 구현에 직접 의존하지 않는가?
* Cargo workspace로 의존 방향이 강제되는가?
* trait는 domain에, 구현은 infra에 있는가?

---

# 2. Domain First

핵심 철학은 “DB 모델”이 아니라 “도메인 모델” 중심 설계다. DB schema를 그대로 서비스 구조로 사용하는 순간, 비즈니스 정책이 persistence 구조에 종속된다. 따라서 domain layer는 DB row 구조가 아니라 business meaning을 표현해야 한다.

## 2.1 Domain Model != Database Model

좋은 예:

```rust
pub struct Todo {
    pub id: Id<Todo>,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
}
```

나쁜 예:

```rust
pub struct Todo {
    pub status_id: String,
    pub created_at: DateTime<Utc>,
}
```

`status_id`는 DB concern이다. 도메인은 상태 의미를 가져야 한다.

## 2.2 Explicit Conversion

레이어 간 객체 전달은 명시적으로 변환한다.

```rust
impl TryFrom<StoredTodo> for Todo {
    type Error = anyhow::Error;

    fn try_from(source: StoredTodo) -> Result<Self, Self::Error> {
        ...
    }
}
```

장점:

* 레이어 경계가 명확해진다.
* 암묵적 coupling이 줄어든다.
* 변환 시 validation을 넣을 수 있다.
* persistence 변경 영향이 줄어든다.

## 2.3 Rich Domain over Primitive Obsession

문자열(String) 남용을 줄인다.

좋은 예:

```rust
pub struct Id<T> {
    pub value: Uuid,
    _marker: PhantomData<T>,
}
```

나쁜 예:

```rust
pub id: String
```

강한 타입은 컴파일 타임 안정성을 높인다. 특히 Rust에서는 타입 설계가 곧 validation 전략이다.

## 요약 체크리스트

* domain model이 business meaning을 표현하는가?
* DB schema가 domain으로 새어 나오지 않는가?
* 레이어 간 변환이 명시적인가?
* primitive obsession을 줄였는가?
* 타입 시스템으로 invalid state를 줄이고 있는가?

---

# 3. Usecase Oriented Design

usecase는 단순 service class가 아니다. 하나의 business flow를 orchestration하는 application layer다. controller에는 비즈니스 로직이 없어야 한다.

## 3.1 Thin Controller

좋은 예:

```rust
pub async fn create_todo(
    modules: State<Arc<Modules>>,
    Json(body): Json<JsonCreateTodo>,
) -> Result<impl IntoResponse, AppError> {
    let result = modules
        .todo_use_case()
        .create(body.try_into()?)
        .await?;

    Ok(Json(result))
}
```

controller 역할:

* request parsing
* validation
* usecase 호출
* response serialization

여기서 business rule 판단을 하면 안 된다.

## 3.2 Usecase Owns Workflow

좋은 예:

```rust
pub async fn create(&self, source: NewTodo) -> anyhow::Result<Todo> {
    self.repositories
        .todo_repository()
        .insert(source)
        .await
}
```

유스케이스는:

* transaction boundary
* workflow ordering
* domain collaboration
* external dependency orchestration

를 담당한다.

## 3.3 One Usecase, One Intention

usecase 함수는 “행동(intent)” 중심이어야 한다.

좋은 예:

```rust
create_todo()
complete_todo()
assign_user()
```

나쁜 예:

```rust
save_todo()
process_data()
handle_request()
```

동사가 구체적이어야 business meaning이 드러난다.

## 요약 체크리스트

* controller가 thin한가?
* business logic이 controller에 없는가?
* usecase가 workflow를 소유하는가?
* 함수명이 business intention을 드러내는가?
* generic한 process/save 이름을 남용하지 않는가?

---

# 4. Dependency Injection

생성자 기반 DI를 사용한다. Rust에서는 런타임 reflection 기반 DI보다 명시적 의존 전달이 훨씬 자연스럽다.

## 4.1 Constructor Injection

좋은 예:

```rust
impl<R: RepositoriesModuleExt> TodoUseCase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }
}
```

장점:

* dependency가 명시적이다
* 테스트가 쉽다
* hidden dependency가 없다
* compile-time validation 가능

## 4.2 Modules as Composition Root

Modules는 dependency composition root다.

```rust
impl Modules {
    pub async fn new() -> Self {
        let db = Db::new().await;
        let repositories_module = Arc::new(RepositoriesModule::new(db.clone()));

        let todo_use_case = TodoUseCase::new(repositories_module.clone());

        Self {
            todo_use_case,
        }
    }
}
```

객체 생성은 한 곳에서 조립한다. 비즈니스 코드 내부에서 객체를 직접 생성하지 않는다.

나쁜 예:

```rust
let repository = TodoRepositoryImpl::new();
```

이런 코드는 dependency inversion을 깨뜨린다.

## 4.3 Prefer Trait Boundary

구현보다 trait에 의존한다.

```rust
pub struct TodoUseCase<R: RepositoriesModuleExt>
```

이는:

* 테스트 mock 교체
* 구현 변경
* infra 교체
* feature 확장

을 쉽게 만든다.

## 요약 체크리스트

* constructor injection을 사용하는가?
* dependency가 명시적인가?
* composition root가 존재하는가?
* business code 내부에서 new 하지 않는가?
* 구현보다 trait에 의존하는가?

---

# 5. Error Handling

Rust에서는 panic보다 Result 기반 흐름이 기본이다. anyhow + thiserror 조합을 사용한다.

## 5.1 Recoverable Error vs Panic

panic은 시스템 불변식이 깨졌을 때만 사용한다.

좋은 예:

```rust
pub async fn get(&self, id: &Id<Todo>) -> anyhow::Result<Option<Todo>>
```

나쁜 예:

```rust
.unwrap()
.expect("todo not found")
```

HTTP API에서는 대부분 recoverable error다.

## 5.2 Domain Error Mapping

에러는 의미를 보존해야 한다.

좋은 예:

```rust
Err(AppError::InvalidJwt(err.to_string()))
```

나쁜 예:

```rust
Err(anyhow!("something wrong"))
```

generic error message는 debugging과 observability를 망친다.

## 5.3 Error Boundary

에러 변환은 layer boundary에서 처리한다.

예:

* sqlx error → infra
* domain error → usecase
* http error → controller

각 레이어는 자신이 이해 가능한 error abstraction만 알아야 한다.

## 요약 체크리스트

* unwrap/expect 남용을 피하는가?
* panic을 exceptional case에만 사용하는가?
* 에러 의미가 보존되는가?
* 레이어별 error boundary가 존재하는가?
* generic message 대신 domain 의미를 표현하는가?

---

# 6. Type-Driven Design

Rust에서는 타입 설계가 곧 설계 문서다. trait, generic, enum을 적극 활용한다.

## 6.1 Encode Rules in Types

좋은 예:

```rust
Option<TodoStatus>
```

이는 상태가 없을 수 있다는 의미를 타입으로 표현한다.

나쁜 예:

```rust
String
```

문자열만 보면 nullable인지, enum인지, uuid인지 알 수 없다.

## 6.2 Prefer Enum over String

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

enum은:

* invalid state 제거
* exhaustive match 보장
* compiler assistance 제공

효과가 있다.

## 6.3 Generic Boundary

좋은 예:

```rust
pub struct DatabaseRepositoryImpl<T>
```

generic abstraction은 중복을 줄이되, domain meaning을 숨기지 않는 수준까지만 사용한다. generic이 과도하면 readability가 급격히 떨어진다.

## 요약 체크리스트

* 타입이 비즈니스 규칙을 표현하는가?
* String 남용을 줄였는가?
* enum으로 invalid state를 제거했는가?
* Option 의미가 명확한가?
* generic이 readability를 해치지 않는가?

---

# 7. Async & Concurrency

Rust async는 단순 성능 최적화가 아니라 ownership 기반 concurrency safety를 제공한다.

## 7.1 Shared State with Arc

좋은 예:

```rust
#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Postgres>>);
```

Arc를 통해 thread-safe shared ownership을 명시한다.

## 7.2 Minimize Mutable State

mutable state는 concurrency complexity를 급격히 증가시킨다.

가능하면:

* immutable data
* pure transformation
* ownership transfer

를 우선 사용한다.

## 7.3 Async Boundary Clarity

async는 I/O boundary에 집중한다.

좋은 예:

```rust
repository.get(id).await
```

나쁜 예:

```rust
complex_cpu_calculation().await
```

CPU 작업을 무분별하게 async로 만들면 오히려 구조가 복잡해진다.

## 요약 체크리스트

* shared state를 명시적으로 관리하는가?
* mutable state를 최소화했는가?
* async가 I/O 중심으로 사용되는가?
* ownership 흐름이 명확한가?
* concurrency safety를 타입으로 보장하는가?

---

# 8. Database & Repository

Repository는 단순 CRUD wrapper가 아니다. 도메인 persistence abstraction이다.

## 8.1 Repository Hides Persistence Detail

좋은 예:

```rust
async fn find(&self, status: Option<TodoStatus>)
```

나쁜 예:

```rust
async fn find_by_sql(sql: &str)
```

domain layer는 SQL을 몰라야 한다.

## 8.2 Query Responsibility

infra만 SQL을 가진다.

```rust
let sql = r#"
select ...
"#;
```

SQL 최적화 concern은 infra에 존재한다.

## 8.3 Transaction Boundary in Usecase

트랜잭션은 usecase에서 orchestration한다.

```rust
executor: impl PostgresAcquire<'_>
```

이 구조는:

* Pool
* Transaction

모두를 동일 abstraction으로 처리 가능하게 만든다.

## 요약 체크리스트

* repository가 persistence detail을 숨기는가?
* domain이 SQL을 모르고 있는가?
* transaction boundary가 usecase에 있는가?
* infra가 DB concern을 소유하는가?
* CRUD abstraction이 business meaning을 유지하는가?

---

# 9. API Design

HTTP API는 transport protocol이 아니라 public contract다.

## 9.1 Explicit Request/Response Model

좋은 예:

```rust
#[derive(Deserialize, Debug, Validate)]
pub struct JsonCreateTodo {
    ...
}
```

request model을 명시적으로 분리한다.

## 9.2 Validation at Boundary

validation은 가능한 boundary 가까이 수행한다.

```rust
#[validate(length(min = 1))]
pub title: Option<String>
```

잘못된 입력은 domain까지 들어가지 않게 한다.

## 9.3 Serialization is Controller Concern

serde model은 controller concern이다. domain model을 그대로 API response로 노출하지 않는다.

이유:

* API evolution
* security
* backward compatibility
* transport independence

를 위해서다.

## 요약 체크리스트

* request/response model이 분리되어 있는가?
* boundary에서 validation 하는가?
* domain model을 직접 노출하지 않는가?
* API contract가 명시적인가?
* transport concern이 controller에 머무는가?

---

# 10. Authentication & Middleware

인증은 business logic이 아니라 cross-cutting concern이다. 따라서 middleware/AOP 형태로 분리한다.

## 10.1 Authentication as Middleware

좋은 예:

```rust
pub async fn auth(
    modules: State<Arc<Modules>>,
    mut req: Request,
    next: Next,
)
```

모든 controller에서 JWT parsing을 반복하지 않는다.

## 10.2 Context Injection

인증 완료 후:

```rust
req.extensions_mut().insert(current_user);
```

request context에 사용자 정보를 주입한다. 이는 controller를 단순화한다.

## 10.3 Security Boundary

인증 실패는 명확하게 처리한다.

좋은 예:

```rust
return Err(InvalidJwt("auth_header not found".to_string()));
```

보안 로직은 “암묵적 fallback”이 없어야 한다.

## 요약 체크리스트

* 인증이 middleware로 분리되어 있는가?
* controller에서 인증 중복이 없는가?
* request context를 사용하는가?
* 인증 실패가 명확한가?
* 보안 로직에 암묵적 fallback이 없는가?

---

# 11. Observability & Logging

운영 가능한 시스템은 tracing 가능해야 한다. tracing 기반 logging을 사용한다.

## 11.1 Structured Logging

좋은 예:

```rust
error!("error authorizing user: {:?}", err);
```

단순 println!은 production observability에 적합하지 않다.

## 11.2 Log with Context

로그에는:

* request id
* user id
* correlation id
* operation

같은 context가 포함되어야 한다.

## 11.3 Error Visibility

에러를 삼키지 않는다.

나쁜 예:

```rust
.ok();
```

이 패턴은 debugging을 어렵게 만든다. 오류를 intentional하게 무시하는 경우만 사용한다.

## 요약 체크리스트

* structured logging을 사용하는가?
* 로그에 context가 포함되는가?
* println! 대신 tracing을 사용하는가?
* 에러를 삼키지 않는가?
* 운영 환경 debugging이 가능한가?

---

# 12. Readability First

좋은 코드는 clever한 코드가 아니라 읽기 쉬운 코드다. Rust에서는 특히 type complexity와 lifetime complexity가 readability를 해치기 쉽다.

## 12.1 Prefer Explicitness

좋은 예:

```rust
match stored_todo {
    Some(st) => Ok(Some(st.try_into()?)),
    None => Ok(None),
}
```

짧은 코드보다 명확한 코드가 우선이다.

## 12.2 Small Scope

변수 scope는 가능한 좁게 유지한다. 이는 ownership/lifetime 문제를 줄인다.

## 12.3 Naming Matters

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

# 13. Testing Philosophy

테스트는 implementation verification이 아니라 business behavior verification이다.

## 13.1 Test Business Rule

좋은 테스트:

```text
"완료된 Todo는 다시 완료 처리할 수 없다"
```

나쁜 테스트:

```text
"repository.save()가 호출되었다"
```

mock interaction보다 business outcome이 중요하다.

## 13.2 Prefer Dependency Injection

DI 구조는 테스트 가능성을 높인다.

```rust
TodoUseCase<R: RepositoriesModuleExt>
```

mock repository를 쉽게 교체할 수 있다.

## 13.3 Test at Proper Layer

| Layer      | 테스트 포인트       |
| ---------- | ------------- |
| domain     | business rule |
| usecase    | workflow      |
| infra      | integration   |
| controller | contract      |

모든 테스트를 e2e로 만들지 않는다.

## 요약 체크리스트

* business behavior를 테스트하는가?
* implementation detail 테스트를 줄였는가?
* DI 구조가 테스트 가능성을 높이는가?
* 레이어별 테스트 목적이 명확한가?
* mock interaction에 과도하게 의존하지 않는가?

---

# 14. Documentation & API Schema

문서는 나중 작업이 아니라 설계 일부다. utoipa 기반 OpenAPI 문서화를 사용한다.

## 14.1 Schema as Contract

좋은 예:

```rust
#[derive(utoipa::OpenApi)]
```

API schema를 코드와 함께 유지한다.

## 14.2 Documentation Close to Code

좋은 예:

```rust
#[utoipa::path(
    get,
    path = "/v1/todos/{id}",
)]
```

문서가 코드 근처에 있어야 drift가 줄어든다.

## 14.3 Self-Describing API

명확한 schema/model naming을 사용한다.

```rust
JsonCreateTodo
JsonUpdateTodoContents
```

API는 문서 없이도 어느 정도 읽혀야 한다.

## 요약 체크리스트

* API schema가 코드와 함께 관리되는가?
* 문서가 코드 근처에 존재하는가?
* OpenAPI contract가 유지되는가?
* naming이 self-describing한가?
* 문서 drift를 줄이고 있는가?

---

# 15. AI Coding Style Alignment

이 프로젝트는 AI-assisted coding 시대에도 유지 가능한 구조를 목표로 한다. AI가 코드를 생성하더라도 architecture consistency가 유지되어야 한다.

## 15.1 Enforce Structure over Prompting

"좋은 코드를 써줘"보다:

* layer boundary
* trait abstraction
* DTO separation
* dependency direction

같은 구조적 제약이 더 중요하다.

## 15.2 Make Invalid Architecture Hard

Rust의 강점은 compile-time restriction이다. workspace dependency, trait boundary, ownership system을 활용해 잘못된 구조를 어렵게 만든다.

## 15.3 Readability for Humans and AI

AI도 결국 기존 코드 패턴을 학습한다.

따라서:

* naming consistency
* folder consistency
* explicit boundary
* predictable patterns

이 중요하다. 일관성이 높은 프로젝트일수록 AI assistance 품질도 좋아진다. ([arxiv.org](https://arxiv.org/abs/2403.14986))

## 요약 체크리스트

* architecture consistency가 유지되는가?
* invalid architecture를 어렵게 만들었는가?
* AI가 학습하기 쉬운 패턴인가?
* naming/folder structure가 일관적인가?
* 구조적 제약이 존재하는가?

---

# Final Principles

axum-rusty 스타일의 핵심은 다음과 같다.

1. Framework보다 Domain이 우선이다.
2. Runtime 규칙보다 Compile-time 제약을 선호한다.
3. 구현보다 의존 구조를 더 중요하게 본다.
4. Smart code보다 Readable code를 선호한다.
5. 암묵성보다 명시성을 선호한다.
6. DB 중심이 아니라 Business 중심으로 모델링한다.
7. 타입 시스템을 적극 활용해 invalid state를 제거한다.
8. 테스트 가능성은 설계 품질의 결과다.
9. AI 시대일수록 구조 일관성이 더 중요하다.
10. 유지보수 비용을 줄이는 것이 최우선 목표다.

이 문서는 단순 코드 스타일 규칙집이 아니라, 프로젝트 전체의 architectural thinking을 공유하기 위한 기준 문서다.

## 참고 문서

- [언어에 의존하지 않는 도메인 중심 코딩 원칙과 실천법](https://www.mimul.com/blog/ai-coding-style/)
- `.claude/rules/rust-security-style.md` — 보안 원칙
- `.claude/rules/rust-test-style.md` — 테스트 원칙
