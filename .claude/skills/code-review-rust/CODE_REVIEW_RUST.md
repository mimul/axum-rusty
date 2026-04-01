---
name: code-review-rust
description: >
  Rust 코드 리뷰 체크리스트 레퍼런스 문서.
  /code-review-rust 커맨드의 SKILL.md가 이 문서를 참조하여
  10개 카테고리(C-CR-01~C-CR-10) 기준으로 코드를 검토한다.
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

## [C-CR-01] 에러 처리

### 체크 항목

- `unwrap()` / `expect()`를 프로덕션 코드에서 무분별하게 사용하고 있지 않은가?
- `?` 연산자를 사용할 수 있는 곳에 `match`로 장황하게 처리하고 있지 않은가?
- 커스텀 에러 타입은 `std::error::Error`를 구현하고 있는가?
- `thiserror` / `anyhow` 크레이트를 상황에 맞게 사용하는가?
  - 라이브러리: `thiserror`로 구체적인 커스텀 에러 타입 정의
  - 애플리케이션: `anyhow`로 간편하게 에러 전파
- `panic!`을 에러 처리 수단으로 사용하고 있지 않은가?

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

### 체크 항목

- 빈 `Vec`, `HashMap`, 문자열 슬라이스(`""`)를 전달했을 때 패닉이 없는가?
- `Option<T>` 처리 시 `None` 케이스를 빠뜨리지 않았는가?
- 0으로 나누는 경우에 대처가 있는가?
- 정수 오버플로우가 발생할 수 있는 연산에 `checked_add()` 등을 사용하는가?
- 슬라이스 인덱스 접근 시 범위 체크가 되어 있는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
fn first_element(v: &[i32]) -> i32 { v[0] }          // 빈 Vec 패닉
let ratio = success / total;                           // 0 나누기 패닉
let item = items[idx];                                 // 범위 초과 패닉
let result = a + b;                                    // 오버플로우 (release 모드 wrap)

// ✅ 올바른 패턴
fn first_element(v: &[i32]) -> Option<i32> { v.first().copied() }
let ratio = if total > 0 { success as f64 / total as f64 } else { 0.0 };
let item = items.get(idx).ok_or(AppError::OutOfRange)?;
let result = a.checked_add(b).ok_or(AppError::Overflow)?;
// 또는: let result = a.saturating_add(b);
```

---

## [C-CR-04] 타입 설계

### 체크 항목

- `bool` 파라미터 대신 의미 있는 `enum`을 사용하는가?
- 원시 타입(`i32`, `String`) 대신 newtype 패턴으로 의미를 부여하는가?
- `String`을 받을 수 있는 곳에 `&str`을 사용하여 불필요한 할당을 피하는가?
- 반환 타입으로 `Vec<T>` 대신 `impl Iterator<Item = T>`를 활용하는가?

### 판정 기준 예시

```rust
// ❌ 위험 패턴
fn connect(host: &str, use_tls: bool) { ... }         // bool 의미 불명확
fn get_order(user_id: u64, order_id: u64) { ... }     // 혼동 가능

// ✅ 올바른 패턴
#[derive(Debug, Clone, Copy)]
enum TlsMode { Enabled, Disabled }
fn connect(host: &str, tls: TlsMode) { ... }
connect("example.com", TlsMode::Enabled);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderId(u64);
fn get_order(user_id: UserId, order_id: OrderId) { ... }
```

---

## [C-CR-05] 동시성·스레드 안전

### 체크 항목

- 공유 상태는 `Mutex<T>` / `RwLock<T>`으로 보호되어 있는가?
- `Mutex` 락을 잡은 채로 오래 걸리는 작업(I/O, 네트워크)을 수행하지 않는가?
- `RwLock` 사용 시 읽기/쓰기 비율을 고려하여 선택했는가?
- 비동기(async) 코드에서 `std::sync::Mutex` 대신 `tokio::sync::Mutex`를 사용하는가?
- `Send` / `Sync` 트레이트 경계가 스레드 안전을 보장하도록 설정되어 있는가?

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
```

---

## [C-CR-06] 비동기(Async/Await)

### 체크 항목

- `async fn` 내에서 블로킹 I/O(`std::fs`, `std::net`)를 직접 호출하지 않는가?
- `Future`를 생성만 하고 `.await`하지 않아 실행되지 않는 경우가 없는가?
- `tokio::spawn`으로 태스크를 분리할 때 에러 처리를 빠뜨리지 않았는가?
- 불필요하게 `async`로 감싼 함수가 없는가? (동기 로직은 동기 함수 유지)

### 판정 기준 예시

```rust
// ❌ 위험 패턴
async fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()             // 런타임 스레드 블로킹
}

tokio::spawn(async { some_task().await });             // spawn 결과 무시

// ✅ 올바른 패턴
async fn read_file(path: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(path).await
}

let handle = tokio::spawn(async { some_task().await });
handle.await??;                                        // JoinError + 내부 에러 처리
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

### 체크 항목

- 하나의 함수가 둘 이상의 책임을 갖고 있지 않은가? (단일 책임 원칙)
- 매직 넘버와 매직 문자열은 `const` 또는 `enum`으로 정의되어 있는가?
- 공개 API(`pub fn`, `pub struct`)에 `///` 문서 주석이 작성되어 있는가?
- `#[derive]`로 자동 구현 가능한 트레이트를 수동으로 구현하고 있지 않은가?
- 불필요한 `pub` 공개 범위를 최소화하고 있는가? (`pub(crate)`, `pub(super)` 활용)

### 판정 기준 예시

```rust
// ❌ 위험 패턴
if retry_count > 3 {                                   // 매직 넘버
    std::thread::sleep(Duration::from_secs(30));
}

// ✅ 올바른 패턴
const MAX_RETRY: u32 = 3;
const RETRY_WAIT_SECS: u64 = 30;

if retry_count > MAX_RETRY {
    std::thread::sleep(Duration::from_secs(RETRY_WAIT_SECS));
}

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

### 체크 항목

- 단위 테스트가 정상계뿐만 아니라 에러 케이스도 커버하는가?
- `#[should_panic]`보다 `Result`를 반환하는 테스트가 더 적절한 경우는 없는가?
- `cargo test` 외에 `cargo clippy`와 `cargo fmt --check`가 통과하는가?

### 판정 기준 예시

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ✅ 정상계 + 에러 케이스 + 에지 케이스 모두 커버
    #[test]
    fn test_average_normal() {
        assert_eq!(average(&[1, 2, 3]), 2.0);
    }

    #[test]
    fn test_average_empty() {
        assert_eq!(average(&[]), 0.0);           // 에지 케이스
    }

    #[test]
    fn test_find_user_not_found() {
        assert!(matches!(find_user(UserId(9999)), Err(AppError::NotFound(_))));
    }

    // ✅ Result 반환 테스트 (should_panic 대신)
    #[test]
    fn test_parse() -> Result<(), AppError> {
        let val = parse("42")?;
        assert_eq!(val, 42);
        Ok(())
    }
}
```

---

## 심각도 기준

| 심각도 | 정의 | 즉시 수정 필요 |
|--------|------|----------------|
| 🔴 **Critical** | 패닉·데이터 레이스·메모리 위험 | 필수 |
| 🟠 **High** | 런타임 오류 가능, unsafe 오남용 | 필수 |
| 🟡 **Medium** | 성능 저하, 관용 표현 위반 | 권장 |
| 🔵 **Low** | 문서 누락, 스타일 개선 여지 | 선택 |
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
| C-CR-08 매직 넘버→const | ✅ 명명 추출 | 의미 변경 없는 리팩토링 |
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

GitHub Actions의 Claude 리뷰는 코드 품질 패턴만 검사한다.
아래 항목은 **반드시 인간 리뷰어가 직접 판단**해야 한다:

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
