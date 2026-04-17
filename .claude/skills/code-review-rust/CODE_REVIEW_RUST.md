---
name: code-review-rust
description: >
  Rust 코드 리뷰 체크리스트 레퍼런스 문서.
  /code-review-rust 커맨드의 SKILL.md가 이 문서를 참조하여 10개 카테고리(C-CR-01~C-CR-10) 기준으로 코드를 검토한다.
reference: SKILL.md
---

# CODE REVIEW RUST — 10개 카테고리 체크리스트

## 카테고리 코드 맵

| 코드 | 카테고리 | 핵심 위험 |
|------|----------|-----------|
| **C-CR-01** | 에러 처리 | unwrap/panic, ? 연산자 미활용 |
| **C-CR-02** | 소유권·차용 | 불필요한 clone, 잘못된 참조 |
| **C-CR-03** | 에지 케이스 | 빈 컬렉션 패닉, 정수 오버플로우 |
| **C-CR-04** | 타입 설계 | bool 파라미터, 원시 타입 남용 |
| **C-CR-05** | 동시성·스레드 안전 | 데이터 레이스, 잘못된 Mutex |
| **C-CR-06** | 비동기(Async/Await) | 블로킹 I/O, 누락된 await |
| **C-CR-07** | unsafe 코드 | 불필요한 unsafe, SAFETY 주석 누락 |
| **C-CR-08** | 코드 품질 | 단일 책임 위반, 매직 넘버, 문서 누락 |
| **C-CR-09** | Rust 관용 표현 | 수동 루프, 장황한 match |
| **C-CR-10** | 테스트 | 에러 케이스 누락, clippy 미통과 |

---

## 리뷰 결과 분류 체계

리뷰 결과는 다음 4가지 카테고리로 분류한다:

| 분류 | 의미 | 대응 |
|------|------|------|
| **🚫 Blocking Issues** | 반드시 수정이 필요한 항목 (보안, 버그, 아키텍처 위반) | 머지 전 필수 수정 |
| **⚠️ Recommended Changes** | 권장 개선 사항 (성능, 가독성, 베스트 프랙티스) | 가능하면 이번 PR에 반영 |
| **💡 Suggestions** | 선택적 개선 아이디어 (리팩토링, 최적화 기회) | 향후 고려 |
| **📝 Tech Debt** | 향후 개선이 필요한 기술 부채 | 별도 이슈로 추적 |

---

## 공통 리뷰 항목 (언어 공통)

Rust 특유의 체크리스트(C-CR-01~C-CR-10) 이전에, 언어에 무관하게 적용되는 일반 원칙을 먼저 확인한다.

### 코드 품질 & 설계 원칙

- **SRP (Single Responsibility Principle)**: 각 모듈·함수가 하나의 명확한 책임만 가지는가?
  - 함수가 "그리고(and)"로 설명되어야 한다면 분리 대상
- **DRY (Don't Repeat Yourself)**: 중복 코드가 없고 재사용 가능한 추상화가 있는가?
  - 유사한 로직이 2곳 이상에 존재하면 공통 함수나 트레이트로 추출
- **KISS (Keep It Simple)**: 불필요한 복잡성 없이 단순하고 명확한가?
  - 현재 요구사항을 충족하는 가장 단순한 구현을 선택
  - 미래를 위한 과도한 추상화(YAGNI) 지양

### 가독성 & 유지보수성

- **네이밍 컨벤션**: 변수·함수·타입명이 의미를 명확히 전달하는가?
  - 이름만 보고 역할을 추론할 수 있어야 한다
- **함수 길이**: 함수가 적절한 길이인가? (기준: 50줄 이하)
  - 50줄 초과 시 단일 책임 원칙 위반 신호로 보고 책임 단위로 분리 검토
- **매직 넘버·문자열**: 하드코딩된 값이 상수로 추출되었는가?
  - 의미를 알 수 없는 숫자·문자열은 이름 있는 상수로 대체
- **공개 API 문서화**: 공개된 함수·타입에 사용법과 에러 조건을 설명하는 문서가 있는가?
  - 파라미터의 사전조건(precondition)과 반환값의 의미를 명시한다

### 에지 케이스 & 경계 조건

- **경계값 처리**: 빈 컬렉션, 0, 최대값 등 경계값에서 올바르게 동작하는가?
  - 빈 입력에 "첫 번째 원소 접근" 같은 코드가 패닉을 일으키지 않는가
- **정수 오버플로우·언더플로우**: 산술 연산에서 오버플로우 가능성이 있는가?
  - 입력 범위가 보장되지 않는 경우 안전한 연산(포화, 검사)을 사용하는가
- **0 나누기**: 제수가 0인 경우에 대처가 있는가?
  - 조건 체크 없이 나누기를 수행하지 않는가

### 테스트

- **에러·에지 케이스 커버**: 단위 테스트가 정상계뿐만 아니라 에러·에지 케이스를 커버하는가?
- **테스트 이름**: 테스트 이름이 의도를 명확히 전달하는가?

### 보안 기본 원칙

- **입력 검증**: 모든 외부 입력 (HTTP 파라미터·헤더·바디·환경변수)에 대한 검증이 충분한가?
- **민감 정보 보호**: 비밀번호·토큰 등 민감 정보가 로그나 응답에 노출되지 않는가?
- **인젝션 방지**: SQL 인젝션·커맨드 인젝션 등의 취약점이 없는가?

---

## 피드백 작성 가이드라인

리뷰 코멘트는 **구체적이고 실행 가능한** 형식으로 작성한다:

1. **코드 위치**: 파일명과 라인 번호를 명시 (예: `src/infra/user.rs:42`)
2. **문제 설명**: 무엇이 문제인지 명확히 설명
3. **개선 방안**: 개선된 코드 예시 제공
4. **우선순위**: 각 항목의 우선순위(`🚫`/`⚠️`/`💡`/`📝`) 명시

```
🚫 Blocking | src/usecase/order.rs:87
unwrap() 사용으로 패닉 위험이 있습니다.

// Before
let user = find_user(id).unwrap();

// After
let user = find_user(id).ok_or(AppError::NotFound)?;
```

---

## [C-CR-01] 에러 처리

### 체크 항목

- `unwrap()` / `expect()`를 프로덕션 코드에서 무분별하게 사용하고 있지 않은가?
- `?` 연산자를 사용할 수 있는 곳에 `match`로 장황하게 처리하고 있지 않은가?
- 커스텀 에러 타입은 `std::error::Error`를 구현하고 있는가?
- `thiserror` / `anyhow` 크레이트를 상황에 맞게 사용하는가?
  - 라이브러리: `thiserror`로 구체적인 커스텀 에러 타입 정의
  - 애플리케이션: `anyhow`로 간편하게 에러 전파
- `panic!`을 에러 처리 수단으로 사용하고 있지 않은가?
- 에러 응답이 내부 구현 정보(스택 트레이스, DB 오류 상세, 파일 경로 등)를 외부에 노출하지 않는가?

### 판정 기준 예시

```rust
// ❌ 위험 — panic 유발
let val = some_option.unwrap();
let file = File::open("data.txt").unwrap();

let result = match some_fn() {
    Ok(v) => v,
    Err(e) => return Err(e),  // ? 연산자로 대체 가능
};

// ✅ 올바른 패턴
let val = some_option.ok_or(AppError::NotFound)?;
let file = File::open("data.txt")?;
let result = some_fn()?;

// ✅ 커스텀 에러 타입 (thiserror 사용)
use thiserror::Error;
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}
```

---

## [C-CR-02] 소유권·차용(Ownership & Borrowing)

### 체크 항목

- 불필요한 `.clone()`을 남용하여 성능 손실을 일으키고 있지 않은가?
- 소유권 이동(move)이 필요한 곳과 참조(`&`)로 충분한 곳을 구분하는가?
- 불변 참조(`&T`)로 충분한 곳에 가변 참조(`&mut T`)를 사용하고 있지 않은가?
- 라이프타임 어노테이션(`'a`)을 명확하지 않은 경우 적절히 표기하는가?
- `Rc<T>` / `Arc<T>`의 순환 참조로 메모리 누수가 발생하지 않는가?

### 판정 기준 예시

```rust
// ❌ 위험 — 불필요한 clone
fn print_name(name: String) { println!("{}", name); }
print_name(user.name.clone());

// ✅ 올바른 패턴 — 참조 사용
fn print_name(name: &str) { println!("{}", name); }
print_name(&user.name);

// ✅ 라이프타임 어노테이션
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}

// ✅ 순환 참조 방지
use std::rc::{Rc, Weak};
struct Node { parent: Weak<Node>, children: Vec<Rc<Node>> }
```

---

## [C-CR-03] 에지 케이스

> 경계값(빈 컬렉션, 0, 오버플로우)에 대한 **언어 공통** 원칙은 공통 리뷰 항목 — 에지 케이스 & 경계 조건을 참조한다.
> 여기서는 해당 원칙을 Rust에서 올바르게 구현하는 방법을 검사한다.

### 체크 항목

- `Option<T>` 처리 시 `None` 케이스를 빠뜨리지 않았는가?
- 슬라이스 접근에 인덱스(`[idx]`) 대신 `.get(idx)`을 사용하는가?
- 정수 오버플로우에 `checked_add()` / `saturating_add()` / `wrapping_add()`를 의도에 맞게 선택하는가?
- `as` 캐스팅 시 타입 축소(truncation) 위험이 없는가? (`u32 as u8` 등)
- 에지 케이스에 `unwrap_or` / `unwrap_or_else` / `unwrap_or_default`로 안전한 기본값을 제공하는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
fn first_element(v: &[i32]) -> i32 { v[0] }          // 빈 슬라이스 패닉
let item = items[idx];                                 // 범위 초과 패닉
let small = big_num as u8;                            // silently truncates
let result = a + b;                                    // release 모드에서 wrap

// ✅ Rust 관용 패턴
fn first_element(v: &[i32]) -> Option<i32> { v.first().copied() }
let item = items.get(idx).ok_or(AppError::OutOfRange)?;
let small = u8::try_from(big_num).map_err(|_| AppError::Overflow)?;
let result = a.checked_add(b).ok_or(AppError::Overflow)?;
// 또는 포화: a.saturating_add(b)

// ✅ unwrap_or 계열로 기본값 제공
let name = config.get("name").unwrap_or("unknown");
let count = parse_count(s).unwrap_or_default();       // 0
```

---

## [C-CR-04] 타입 설계

> `bool` 파라미터 기피(불명확한 파라미터 타입)는 공통 리뷰 항목 — 코드 품질 & 설계 원칙을 참조한다.
> 여기서는 Rust 고유의 타입 시스템 활용을 검사한다.

### 체크 항목

- `bool` 파라미터 대신 의미 있는 `enum`을 사용하는가?
  - `connect(host, true)` 대신 `connect(host, TlsMode::Enabled)` 형식 사용
- 원시 타입(`i32`, `String`) 대신 newtype 패턴으로 의미를 부여하는가?
  - 같은 타입의 파라미터 여럿이 순서를 혼동할 수 있는 경우 특히 중요
- `String`을 받을 수 있는 곳에 `&str`을 사용하여 불필요한 할당을 피하는가?
- 반환 타입으로 `Vec<T>` 대신 `impl Iterator<Item = T>`를 활용하는가?
- 재귀 타입이나 큰 enum variant를 `Box<T>`로 감싸 스택 오버플로우를 방지하는가?
- 외부 입력은 `Newtype` 또는 `TryFrom`을 이용해 도메인 타입 생성 시점에서 검증하는가?
  - 검증되지 않은 입력을 DB 쿼리나 파일 경로에 직접 사용하지 않는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
fn get_order(user_id: u64, order_id: u64) { ... }     // 인수 순서 혼동 가능
fn list_users() -> Vec<User> { ... }                   // 중간 소비자가 collect 비용 부담

// ✅ 올바른 패턴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderId(u64);
fn get_order(user_id: UserId, order_id: OrderId) { ... }

fn list_users() -> impl Iterator<Item = User> { ... } // 지연 평가, 유연성

// ✅ 큰 enum variant Box 처리
enum Expr {
    Lit(i64),
    Add(Box<Expr>, Box<Expr>),   // Box 없으면 무한 크기
}
```

---

## [C-CR-05] 동시성·스레드 안전

### 체크 항목

- 공유 상태는 `Mutex<T>` / `RwLock<T>`으로 보호되어 있는가?
- `Mutex` 락을 잡은 채로 오래 걸리는 작업(I/O, 네트워크)을 수행하지 않는가?
- `RwLock` 사용 시 읽기/쓰기 비율을 고려하여 선택했는가?
- 비동기(async) 코드에서 `std::sync::Mutex` 대신 `tokio::sync::Mutex`를 사용하는가?
- `Send` / `Sync` 트레이트 경계가 스레드 안전을 보장하도록 설정되어 있는가?
- 동일한 `Mutex`를 중첩으로 잠그는 데드락 패턴이 없는가?
- 단순 카운터·플래그에 `Arc<Mutex<T>>` 대신 `AtomicUsize` / `AtomicBool` 등 atomic 타입을 활용하는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
static mut COUNTER: u64 = 0;                          // 데이터 레이스

// async 컨텍스트에서 std Mutex를 .await 경계에 걸쳐 사용
let guard = std::sync::Mutex::new(0).lock().unwrap();
some_async_fn().await;                                 // 데드락 위험

// ✅ 올바른 패턴
let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

// async 컨텍스트
let mutex = tokio::sync::Mutex::new(0);
let guard = mutex.lock().await;
some_async_fn().await;                                 // 안전

// ✅ 단순 카운터에는 atomic 타입 선호
use std::sync::atomic::{AtomicUsize, Ordering};
static REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0);
REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
```

---

## [C-CR-06] 비동기(Async/Await)

### 체크 항목

- `async fn` 내에서 블로킹 I/O(`std::fs`, `std::net`)를 직접 호출하지 않는가?
- `Future`를 생성만 하고 `.await`하지 않아 실행되지 않는 경우가 없는가?
- `tokio::spawn`으로 태스크를 분리할 때 에러 처리를 빠뜨리지 않았는가?
- 불필요하게 `async`로 감싼 함수가 없는가? (동기 로직은 동기 함수 유지)
- `tokio::select!` 사용 시 취소 안전(cancellation-safe)하지 않은 Future가 포함되어 있지 않은가?
- `.await` 지점을 넘어 `!Send` 타입(`Rc<T>`, `MutexGuard<T>` 등)을 보유하지 않는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
async fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()             // 런타임 스레드 블로킹
}

let _fut = some_async_fn();                            // await 없이 생성만 함 — 실행 안 됨

tokio::spawn(async { some_task().await });             // spawn 결과 무시

async fn just_compute(x: i32) -> i32 { x * 2 }       // 동기 로직에 불필요한 async

// .await를 넘는 !Send 타입
let guard = std::sync::Mutex::lock(&m).unwrap();
some_async_fn().await;                                 // guard가 .await를 넘음 → !Send

// ✅ 올바른 패턴
async fn read_file(path: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(path).await              // 비동기 I/O 사용
}

let handle = tokio::spawn(async { some_task().await });
handle.await??;                                        // JoinError + 내부 에러 처리

fn just_compute(x: i32) -> i32 { x * 2 }             // 동기 함수로 유지

// guard를 .await 전에 명시적으로 drop
drop(guard);
some_async_fn().await;

// select!에서 취소 안전성 확인 — tokio::io::AsyncReadExt::read_buf 등은 취소 안전
tokio::select! {
    result = cancel_safe_fn() => { ... }
    _ = timeout_fut => { ... }
}
```

---

## [C-CR-07] unsafe 코드

### 체크 항목

- `unsafe` 블록의 사용이 최소화되어 있으며, 반드시 필요한 경우에만 사용하는가?
- `unsafe` 블록마다 안전 불변식(safety invariants)을 `// SAFETY:` 주석으로 명시하는가?
- 원시 포인터(`*const T`, `*mut T`) 역참조 시 null 체크를 수행하는가?
- FFI 경계에서 데이터 타입의 ABI 호환성을 검증했는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
unsafe { let val = &*ptr; }                            // SAFETY 주석 없음
unsafe { println!("{}", *ptr); }                       // null 체크 없음

// ✅ 올바른 패턴
// SAFETY: ptr은 호출자가 유효한 포인터임을 보장하고,
//         이 함수 호출 동안 다른 가변 참조가 없음이 보장된다.
unsafe {
    let val = &*ptr;
}

if !ptr.is_null() {
    // SAFETY: null 체크 완료, 단독 접근 보장
    unsafe { println!("{}", *ptr); }
}
```

---

## [C-CR-08] 코드 품질

> 공개 API 문서화 원칙은 공통 리뷰 항목 — 가독성 & 유지보수성을 참조한다.
> 여기서는 Rust 고유의 코드 품질 패턴을 검사한다.

### 체크 항목

- 공개 API(`pub fn`, `pub struct`, `pub trait`)에 `///` 문서 주석이 작성되어 있는가? (`# Errors`, `# Panics`, `# Examples` 포함)
- Rust 네이밍 관례를 따르는가?
  - `snake_case`: 변수·함수·모듈
  - `CamelCase`: 타입·트레이트·열거형 variant
  - `SCREAMING_SNAKE_CASE`: 상수·static
  - 축약어(`d`, `tmp`, `val`, `s`) 사용 자제, 도메인 언어에 맞는 이름 사용
- `#[derive]`로 자동 구현 가능한 트레이트를 수동으로 구현하고 있지 않은가?
- 불필요한 `pub` 공개 범위를 최소화하고 있는가? (`pub(crate)`, `pub(super)` 활용)
- `todo!()` / `unimplemented!()` 매크로가 프로덕션 코드에 남아 있지 않은가?
- `#[allow(dead_code)]`, `#[allow(unused)]` 등 lint 억제 어노테이션이 남용되고 있지 않은가?
- 과도하게 깊은 중첩(4레벨 초과)이 없는가? (early return / guard clause로 평탄화)
- 민감 정보(`Display` 구현 시 마스킹 처리, `#[derive(Debug)]` 시 민감 필드 노출 여부 확인)를 로그·응답에서 보호하는가?
  - `tracing`/`log` 매크로에 비밀번호·토큰 등 민감 값을 직접 포함하지 않는가?
- `sqlx` 사용 시 파라미터 바인딩을 사용하는가? (`format!`으로 쿼리를 문자열 조합하는 것 금지)
  - 외부 커맨드 실행 시 인수를 직접 보간하지 않는가?

### 판정 기준 예시

```rust
// ✅ 문서 주석 형식
/// 사용자 ID로 사용자를 조회합니다.
///
/// # Errors
/// 사용자를 찾지 못하면 [`AppError::NotFound`]를 반환합니다.
///
/// # Examples
/// ```
/// let user = find_user(UserId(42))?;
/// ```
pub fn find_user(id: UserId) -> Result<User, AppError> { ... }

// ✅ 공개 범위 최소화
pub(crate) fn internal_helper() { ... }   // pub 대신 pub(crate)
pub(super) fn module_helper() { ... }     // 상위 모듈까지만 공개

// ❌ 깊은 중첩
fn process(req: &Request) {
    if req.is_valid() {
        if let Some(user) = req.user() {
            if user.is_active() {
                // 로직...
            }
        }
    }
}

// ✅ early return으로 평탄화
fn process(req: &Request) -> Result<(), AppError> {
    if !req.is_valid() { return Err(AppError::InvalidRequest); }
    let user = req.user().ok_or(AppError::Unauthorized)?;
    if !user.is_active() { return Err(AppError::Forbidden); }
    // 로직...
    Ok(())
}

// ✅ 민감 정보 마스킹
#[derive(Debug)]
pub struct ApiKey(String);

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey(****)")  // 실제 값 노출 금지
    }
}

// ❌ 인젝션 위험
let query = format!("SELECT * FROM users WHERE id = '{id}'");  // SQL 인젝션

// ✅ sqlx 파라미터 바인딩
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_optional(&pool).await?;
```

---

## [C-CR-09] Rust 관용 표현 (Idiomatic Rust)

### 체크 항목

- `Iterator` 어댑터(`map`, `filter`, `fold`)를 활용할 수 있는 곳에 수동 루프를 작성하지 않는가?
- `if let` / `while let`으로 간결하게 표현할 수 있는 `match`를 장황하게 작성하지 않는가?
- `Option`과 `Result`의 조합에 `and_then`, `map`, `unwrap_or_else`를 활용하는가?
- `String` 포맷팅에 `format!` 매크로를 적절히 활용하는가?
- 구조체 초기화 시 struct update syntax(`..Default::default()`)를 활용하는가?
- `impl Trait` 파라미터를 활용하여 제네릭을 간결하게 표현하는가?

### 판정 기준 예시

```rust
// ❌ 장황한 패턴
let mut result = Vec::new();
for item in &items {
    if item.value > 0 { result.push(item.value * 2); }
}

match opt {
    Some(val) => println!("{}", val),
    None => {},
}

fn print_items<T: Display>(items: &[T]) { ... }

// ✅ 관용적 패턴
let result: Vec<_> = items.iter()
    .filter(|item| item.value > 0)
    .map(|item| item.value * 2)
    .collect();

if let Some(val) = opt { println!("{}", val); }

fn print_items(items: &[impl Display]) { ... }

// struct update syntax
let config = Config { timeout: 30, ..Config::default() };
```

---

## [C-CR-10] 테스트

> 에러·에지 케이스 커버, 테스트 이름 형식은 공통 리뷰 항목 — 테스트를 참조한다.
> 여기서는 Rust 고유의 테스트 품질 패턴을 검사한다.

### 체크 항목

- 테스트 이름이 `[대상]_[조건]_[기대결과]` (`snake_case`) 형식인가?
  - `test1`, `test_order` 대신 `create_order_with_empty_items_returns_error`
- `Result`를 반환하는 함수마다 최소 1개 이상의 에러 케이스 테스트가 있는가?
- `#[should_panic]`보다 `Result`를 반환하는 테스트가 더 적절한 경우는 없는가?
- `cargo test` 외에 `cargo clippy`와 `cargo fmt --check`가 통과하는가?
- `assert!(result.is_ok())` 대신 `unwrap()` 또는 `assert!(matches!(...), "{result:?}")` 형식으로 실패 원인을 명확히 드러내는가?
- `assert_eq!` 실패 시 디버그 정보를 제공하는가? (복잡한 값은 메시지 추가)

### 판정 기준 예시

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ❌ 실패 원인을 알 수 없는 패턴
    #[test]
    fn test_create_user() {
        let result = create_user("alice");
        assert!(result.is_ok());                 // 실패해도 왜 실패했는지 알 수 없음
    }

    // ✅ 실패 원인을 드러내는 패턴
    #[test]
    fn create_user_with_valid_name_returns_user() {
        let user = create_user("alice").unwrap(); // 또는 .expect("create_user failed")
        assert_eq!(user.name, "alice", "user.name mismatch: {user:?}");
    }

    // ✅ 에러 타입까지 검증
    #[test]
    fn find_user_when_missing_returns_not_found() {
        let result = find_user(UserId(9999));
        assert!(matches!(result, Err(AppError::NotFound(_))), "{result:?}");
    }

    // ✅ Result 반환 테스트 (should_panic 대신)
    #[test]
    fn parse_valid_input_returns_value() -> Result<(), AppError> {
        let val = parse("42")?;
        assert_eq!(val, 42);
        Ok(())
    }
}
```

---

## CI 파이프라인 연동 정보

이 체크리스트는 두 가지 경로로 실행된다:

| 실행 경로 | 트리거 | 목적 |
|-----------|--------|------|
| **GitHub Actions** (`claude-review.yml`) | PR에 `ci-passed` 라벨 부착 시 자동 | 모든 PR에 일관된 품질 게이트 적용 |
| **로컬 Claude Code** (`/code-review-rust`) | 개발자가 수동 실행 | PR 올리기 전 자가 점검 |

### GitHub Actions에서의 자동 수정 범위 (`auto_fixable`)

CI에서 Claude가 **자동으로 코드를 수정하고 커밋**하는 이슈:

| 카테고리 | 자동 수정 가능 | 이유 |
|----------|---------------|------|
| C-CR-01 unwrap→? | ✅ 단순 패턴 | 로직 변경 없이 기계적 변환 가능 |
| C-CR-02 clone 제거 | ✅ 참조 변환 | &str/&[T]로 서명 변경, 컴파일로 검증 |
| C-CR-05 std→tokio Mutex | ✅ import 교체 | 타입 교체만으로 해결 가능 |
| 공통 매직 넘버→const | ✅ 명명 추출 | 의미 변경 없는 리팩토링 |
| C-CR-09 루프→Iterator | ✅ 기계적 변환 | 결과 동일, 컴파일로 검증 |
| C-CR-10 fmt/clippy 위반 | ✅ 도구 자동화 | cargo fmt/clippy --fix |

Claude가 **자동 수정하지 않고 코멘트만** 남기는 이슈:

| 카테고리 | 수동 처리 이유 |
|----------|---------------|
| C-CR-03 에지 케이스 | 비즈니스 의도 파악 필요 |
| C-CR-04 타입 설계 | API 시그니처 변경 → 영향 범위 큼 |
| C-CR-06 async 구조 | 런타임 아키텍처 변경 |
| C-CR-07 unsafe | 안전 불변식 인간 검토 필수 |

### 인간 리뷰어가 반드시 확인해야 할 체크포인트

GitHub Actions의 Claude 리뷰는 코드 품질 패턴만 검사한다. 아래 항목은 **반드시 인간 리뷰어가 직접 판단**해야 한다:

```
✅ 인간 리뷰어 전용 체크포인트
─────────────────────────────────────────────────────
① 비즈니스 로직 정확성
   - 도메인 규칙(결제, 재고, 권한 등)에 맞게 동작하는가?
   - 상태 전이가 비즈니스 흐름에 부합하는가?

② 요구사항 충족 여부
   - 티켓/스펙의 AC(Acceptance Criteria)를 모두 만족하는가?
   - 엣지 케이스(동시 요청, 타임아웃, 대용량)를 처리하는가?

③ 버그·성능·보안 개선 여지
   - N+1 쿼리, 불필요한 락 경합, 메모리 누수 등
   - 인증·권한·입력 검증의 새로운 취약점

④ 과도한 리팩토링 여부
   - PR 목적과 무관한 변경이 포함되어 있는가?
   - 단일 PR의 변경 범위가 리뷰하기 적절한가?

⑤ AI 자동 수정 코드 검토
   - Claude가 자동 적용한 수정이 의도에 맞는가?
   - 자동 수정이 다른 기능에 부작용을 일으키지 않는가?
   - fix(ai): [Claude] 커밋을 커밋 히스토리에서 확인
─────────────────────────────────────────────────────
```
