---
name: refactor-rust
description: >
  도메인 중심 진화형 코딩 스타일을 기반으로 운영 중인 Rust 코드를 안전하게 리팩토링할 때 사용하는 스킬.
  도메인 명확성, 변화 용이성, 의도를 드러내는 코드를 목표로 소유권 개선, 에러 처리 체계화,
  Enum 상태 머신, 모듈 구조 재편 등 Rust 전용 리팩토링 전 과정을 가이드한다.
  트리거: "Rust 리팩토링", "Rust 코드 정리", "cargo clippy 개선",
  "소유권 개선", "Rust refactor", "Rust clean up" 등.
---

# REFACTOR SKILL — Rust 운영 코드 리팩토링 가이드

## 개요

이 스킬은 **현재 운영 중인(live) Rust 코드**를 기능 변경 없이 내부 구조를 개선하는 작업을 다룬다.

리팩토링의 목적은 단순한 기술 부채 해소가 아니라 **도메인이 코드에 명확히 드러나고, 변경이 쉬우며, 읽기 좋은 코드**를 만드는 것이다.
Rust의 타입 시스템과 Borrow Checker를 최대한 활용해 **컴파일 타임에 도메인 규칙을 강제**하는 것이 핵심이다.

---

## 핵심 원칙

| 원칙 | 설명 | coding-style.md 근거 |
|------|------|----------------------|
| **기능 동치(Behavioral Equivalence)** | 외부에서 관찰 가능한 동작은 100% 동일하게 유지 | §1.4 점진적 진화 |
| **도메인 중심(Domain-Centric)** | 리팩토링 결과물은 도메인 개념이 코드에 더 잘 드러나야 한다 | §1.3, §2.2 |
| **의도 드러내기(Intent-Revealing)** | 변수·함수·타입명은 구현 방식이 아닌 의도를 표현해야 한다 | §1.2, §4 |
| **소규모 증분(Small Increments)** | 한 번에 하나의 변환만 수행한다 | §1.4 점진적 진화 |
| **항상 그린(Always Green)** | 매 단계마다 `cargo test` 통과 확인 | §3.1 지속적인 리팩토링 |
| **컴파일러를 동반자로(Compiler as Partner)** | 컴파일 오류 메시지를 설계 가이드로 활용 | Rust-specific |
| **되돌릴 수 있게(Reversible)** | Git 커밋 단위를 작게 유지 | §1.4 점진적 진화 |

---

## 리팩토링 트리거

다음 상황 중 하나라도 해당되면 리팩토링을 시작한다 (coding-style.md §3.2):

- **도메인이 코드에 드러나지 않을 때** — primitive 타입(`String`, `u64`)이 도메인 개념을 대신할 때
- **동일한 코드가 3번 이상 반복될 때** — 추상화가 필요하다는 신호 (Rule of Three)
- **이름이 의도를 충분히 설명하지 못할 때** — `data`, `util`, `manager` 같은 모호한 이름
- **함수가 여러 책임을 가지기 시작할 때** — 함수 50줄 초과, 중첩 깊이 3단계 이상
- **잘못된 상태가 타입으로 막히지 않을 때** — `bool` 플래그 조합, 문자열 상태 표현
- **컴파일러 오류를 `.clone()`으로 회피할 때** — 소유권 설계 재검토 신호
- **`.unwrap()`이 라이브러리 코드에 존재할 때** — 패닉 위험

---

## 피드백 작성 가이드라인

리팩토링 피드백은 **코드 냄새 유형 + Before/After 비교 형식**으로 작성한다:

1. **코드 위치**: 파일명과 라인 번호를 명시 (예: `src/domain/order/service.rs:42`)
2. **냄새 유형**: 어떤 카탈로그 항목에 해당하는가? (예: `R-R-02 빈약한 도메인 모델`)
3. **Before/After**: 리팩토링 전후 코드 예시 제공
4. **우선순위**: 각 항목의 우선순위(`🚫`/`⚠️`/`💡`) 명시

| 우선순위 | 의미 | 대응 |
|----------|------|------|
| **🚫 즉시 수정** | 기능 동치 위반, 패닉 위험 | 리팩토링 완료 전 필수 수정 |
| **⚠️ 권장 수정** | 리팩토링 목적 미달성 (도메인 미반영, 중복 잔존, SRP 위반) | 가능하면 이번 PR에 반영 |
| **💡 제안** | 추가 개선 기회 | 향후 고려 |

```
⚠️ 권장 수정 | src/usecase/order.rs:87
R-R-07: 불필요한 clone()이 잔존합니다.

// Before
let name = user.name.clone();
process(&name);

// After
process(&user.name);
```

---

## PHASE 0 — 리팩토링 전 준비

### 0-1. 안전망 확보

```bash
# 기능 동치 기준선 저장
□ cargo test 2>&1 | tee test_before.txt       — 전체 테스트 통과 확인
□ cargo clippy -- -D warnings 2>&1 | tee clippy_before.txt
□ cargo fmt --check                            — 포맷 위반 확인
□ cargo tarpaulin --out Html                   — 커버리지 기준선 (목표: 80% 이상)
□ cargo bench 2>&1 | tee bench_before.txt      — 성능 기준선 (Criterion)
□ git tag refactor/before-start                — 현재 상태 태그 생성
```

**기능 동치 체크리스트** — 리팩토링 완료 후 반드시 확인:

```
□ 외부 동작 불변: 리팩토링 전후로 관찰 가능한 동작이 100% 동일한가?
□ 공개 API 불변: pub fn·pub struct 시그니처가 변경되지 않았는가?
          (파괴적 변경은 별도 PR로 분리)
□ 에러·로그 불변: 에러 메시지·코드·로그 포맷·환경 변수 키가 동일한가?
□ 테스트 통과: cargo test 결과가 before와 동일하거나 개선되었는가?
```

### 0-2. Rust 전용 코드 냄새 탐지 항목

| 냄새 유형 | 증상 | coding-style.md | Clippy Lint |
|-----------|------|-----------------|-------------|
| 의미 없는 이름 | `data`, `util`, 매직 넘버 | §1.2, §4.2 | `clippy::unreadable_literal` |
| Primitive 집착 | `u64`/`String`으로 도메인 개념 표현 | §1.3, §2.2 | 수동 검토 |
| 빈약한 도메인 모델 | 도메인 로직이 서비스에만 있고 엔티티는 데이터만 보유 | §2.2 | 수동 검토 |
| Bool 플래그 남용 | `is_paid + is_shipped` 조합 상태 | §2.4 | 수동 검토 |
| 과도한 중첩 | match/if let 3단계 이상 | §2.1 | `clippy::collapsible_match` |
| 거대 함수 | 함수 50줄 초과 | §2.1 | `clippy::too_many_lines` |
| 중복 코드 | 동일 패턴 3회 이상 반복 | §2.3 | 수동 검토 |
| 암묵적 경계 처리 | `.unwrap()`, 인덱스 무방비 접근 | §5.4 | `clippy::unwrap_used` |
| Clone 남용 | `.clone()`이 컴파일 오류 회피용 | §1.1 | `clippy::clone_on_ref_ptr` |
| 도메인 없는 util | `utils.rs`에 비즈니스 로직 혼재 | §1.3 | 수동 검토 |
| 불필요한 힙 할당 | `String` vs `&str` 혼용 | §1.1 | `clippy::box_default` |
| 블로킹 I/O | async 컨텍스트에서 blocking 호출 | §2.1 | `clippy::async_yields_async` |

### 0-3. 리팩토링 우선순위 매트릭스

```
영향도(Impact) vs 위험도(Risk) 2×2 매트릭스:

         낮은 위험         높은 위험
높은 영향  [1순위]          [3순위]
낮은 영향  [2순위]          [4순위 / 보류]

낮은 위험 변경 (즉시 시작 가능):
  - cargo fmt (포맷만)
  - 변수명 변경 (도메인 의도 반영)       → R-R-01
  - const 추출 (매직 넘버 제거)          → R-R-01
  - unwrap → ? 연산자 변환               → R-R-06
  - Iterator 체인 정리                   → R-R-04

높은 위험 변경 (테스트 안전망 확인 후):
  - 소유권 구조 변경 (참조 → 소유, 반대) → R-R-07
  - 공개 Trait 시그니처 변경             → R-R-05
  - Enum 상태 머신 도입 (bool 필드 제거) → R-R-03
  - 모듈 계층 재편 (pub use 영향)        → R-R-08
  - async/await 도입                     → R-R-04
```

---

## PHASE 1 — 환경 및 도구 설정

### 1-1. Cargo.toml 설정

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"        # 타입화된 에러
anyhow = "1.0"           # 애플리케이션 레벨 에러
async-trait = "0.1"      # async fn in trait
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tokio-test = "0.4"       # 비동기 테스트 유틸리티

[[bench]]
name = "my_bench"
harness = false          # Criterion 사용 시 필수

[profile.release]
lto = true               # Link-Time Optimization
codegen-units = 1        # 최적화 극대화
strip = true             # 심볼 제거 (바이너리 크기 축소)
```

> **주의**: Repository·UseCase·Domain 로직은 실제 DB로 테스트한다. 내부 구현체를 모킹하는 `mockall` 등의 프레임워크는 외부 서비스(결제 API 등) 경계에만 사용한다 (`rust-test-style.md` Mocking Rules 참조).

### 1-2. Clippy 설정 (clippy.toml 또는 소스 상단)

```rust
// src/lib.rs 또는 src/main.rs 최상단
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,          // unwrap() 사용 금지
    clippy::expect_used,          // expect() 사용 제한
    clippy::clone_on_ref_ptr,     // Rc/Arc clone 경고
    clippy::todo,                 // todo!() 잔존 경고
    clippy::dbg_macro,            // dbg!() 잔존 경고
    clippy::print_stdout,         // println!() 라이브러리 사용 경고
    missing_docs,                 // 공개 아이템 문서화 강제
    dead_code,
    unused_imports,
    unused_variables,
)]
```

### 1-3. rustfmt.toml 설정

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

### 1-4. 필수 Cargo 명령어 참조

```bash
cargo clippy -- -D warnings          # 경고를 오류로 처리
cargo clippy --fix --allow-dirty     # 자동 수정 가능한 항목 수정
cargo fmt                            # 포맷 자동 적용
cargo fmt --check                    # 포맷 위반 확인 (CI용)
cargo test                           # 전체 테스트
cargo tarpaulin --out Html --output-dir coverage/
cargo bench                          # 전체 벤치마크
cargo doc --no-deps --open           # 문서 생성 및 열기
cargo audit                          # 보안 취약점 확인
```

---

## PHASE 2 — 리팩토링 카탈로그

> 각 항목은 `coding-style.md`의 해당 섹션에서 직접 도출했다.

| 코드 | 이름 | coding-style.md 근거 | 우선순위 |
|------|------|----------------------|----------|
| R-R-01 | 의도를 드러내는 네이밍 | §1.2, §4 | ⚠️💡 |
| R-R-02 | 빈약한 도메인 모델 개선 | §1.3, §2.2 | ⚠️💡 |
| R-R-03 | 상태 & 제어 흐름 명확화 | §2.4, §2.2 | 🚫⚠️ |
| R-R-04 | 함수 분해 & 단일 책임 | §2.1, §1.4 | ⚠️💡 |
| R-R-05 | 중복 제거 & 적시 추상화 | §2.3, §1.1 | ⚠️💡 |
| R-R-06 | 경계 조건 & 에러 처리 명시화 | §5, §1.2 | 🚫⚠️ |
| R-R-07 | 소유권 & 변경 용이성 | §1.1, §2.1 | ⚠️💡 |
| R-R-08 | 모듈 구조 도메인화 | §1.3, §2.1 | 💡 |

---

### [R-R-01] 의도를 드러내는 네이밍

**coding-style.md 근거**: §1.2 의도를 드러내는 코드, §4 네이밍 규칙

**적용 시점**: 이름만 보고 목적을 알 수 없거나, 의미 불명확한 숫자·문자열 리터럴이 있을 때

```rust
// ❌ Before: 매직 넘버 + 구현 방식을 드러내는 이름
fn check(pw: &str) -> bool {           // "check"가 무엇을 확인하는가?
    pw.len() >= 8 && pw.len() <= 72    // 8, 72가 왜?
}

fn get_data(ids: &[u64]) -> Vec<String> {  // "data"는 무엇인가?
    ids.iter()
        .filter(|&&id| id % 2 == 0)        // 짝수 ID가 왜?
        .map(|id| format!("user_{id}"))
        .collect()
}

// ✅ After: 도메인 용어 + 의미 있는 상수 → 코드가 문서가 된다
/// 최소 비밀번호 길이 (NIST SP 800-63B 권고 기준)
const MIN_PASSWORD_LEN: usize = 8;

/// 최대 비밀번호 길이 (bcrypt 입력 제한)
const MAX_PASSWORD_LEN: usize = 72;

fn is_valid_password_length(password: &str) -> bool {
    (MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&password.len())
}

const ELIGIBLE_USER_MODULUS: u64 = 2;  // 짝수 ID = 파일럿 그룹

fn find_pilot_user_labels(user_ids: &[u64]) -> Vec<String> {
    user_ids.iter()
        .filter(|&&id| id % ELIGIBLE_USER_MODULUS == 0)
        .map(|id| format!("user_{id}"))
        .collect()
}
```

**네이밍 금지 목록** (coding-style.md §4.2):
```
❌ data, util, helper, manager, handler2, tmp, flag
❌ 불필요한 축약: usr, cnt, idx (→ user, count, index)
❌ 구현 방식 표현: StringParser, ListProcessor (→ EmailParser, OrderProcessor)
❌ 동일 개념에 여러 이름: user_id / userId / uid (→ 하나로 통일)
```

---

### [R-R-02] 빈약한 도메인 모델 개선

**coding-style.md 근거**: §1.3 도메인 중심 설계, §2.2 도메인 모델

**적용 시점**: primitive 타입이 도메인 개념을 대신하거나, 도메인 로직이 서비스 레이어에만 분산되어 있을 때

```rust
// ❌ Before-A: Primitive 집착 — u64 두 개, 순서 바뀌어도 컴파일 통과
fn transfer(from: u64, to: u64, amount: i64) { ... }
transfer(to_id, from_id, amount);  // 버그: 계좌 반전, 컴파일 통과

// ✅ After-A: Newtype + Smart Constructor — 도메인 규칙을 생성 시점에 강제
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(u64);

impl AccountId {
    /// 0은 유효하지 않은 계좌 ID
    pub fn new(id: u64) -> Result<Self, DomainError> {
        if id == 0 { return Err(DomainError::InvalidAccountId); }
        Ok(Self(id))
    }
    pub fn value(self) -> u64 { self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmountCents(i64);

impl AmountCents {
    pub fn new(cents: i64) -> Result<Self, DomainError> {
        if cents < 0 { return Err(DomainError::NegativeAmount); }
        Ok(Self(cents))
    }
    pub fn value(self) -> i64 { self.0 }
}

fn transfer(from: AccountId, to: AccountId, amount: AmountCents) { ... }
// transfer(to_id, from_id, amount);  // ✅ 컴파일 오류 — 버그 차단

// ❌ Before-B: 빈약한 도메인 모델 — 로직이 서비스에만 있음
pub struct Order { pub status: String, pub amount: i64 }

impl OrderService {
    fn apply_discount(&self, order: &mut Order, pct: u8) {
        if order.status == "active" {           // 문자열 비교
            order.amount -= order.amount * pct as i64 / 100;
        }
    }
}

// ✅ After-B: 풍부한 도메인 모델 — 데이터와 행동이 함께 (coding-style.md §2.2)
pub struct Order {
    status: OrderStatus,  // private — 외부에서 직접 변경 불가
    amount: AmountCents,
}

impl Order {
    /// 도메인 행동: 할인 적용은 활성 주문에만 가능
    pub fn apply_discount(&mut self, pct: DiscountPercent) -> Result<(), OrderError> {
        if !self.status.is_active() {
            return Err(OrderError::DiscountOnInactiveOrder);
        }
        self.amount = self.amount.apply_discount(pct);
        Ok(())
    }
}
// OrderService는 이제 apply_discount를 직접 계산하지 않고 order.apply_discount()를 호출한다
```

> **핵심**: Newtype 단독으로는 불충분하다. Smart Constructor(`fn new(...) -> Result<Self, E>`)로 invariant를 생성 시점에 강제하고, 필드는 private으로 유지해야 한다.

---

### [R-R-03] 상태 & 제어 흐름 명확화

**coding-style.md 근거**: §2.4 제어 흐름, §2.2 도메인 모델

**적용 시점**: `bool` 플래그 조합이나 문자열로 상태를 표현하거나, 중첩 깊이가 3단계를 초과할 때

```rust
// ❌ Before: bool 플래그 — 불가능한 조합이 타입으로 방지되지 않음
pub struct Order {
    pub is_paid: bool,
    pub is_shipped: bool,
    pub is_cancelled: bool,
}
// is_shipped = true && is_cancelled = true → 불가능하지만 컴파일 통과

// ✅ After-A: Enum 상태 머신 — Make Illegal States Unrepresentable
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Paid { paid_at: DateTime<Utc>, transaction_id: String },
    Shipped { shipped_at: DateTime<Utc>, tracking_number: String },
    Cancelled { reason: CancellationReason },
}

impl Order {
    /// Tell, Don't Ask: 상태 전이 로직을 Order 내부에 캡슐화
    pub fn mark_as_paid(&mut self, tx_id: String) -> Result<(), OrderError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Paid { paid_at: Utc::now(), transaction_id: tx_id };
                Ok(())
            }
            other => Err(OrderError::InvalidTransition { from: format!("{other:?}"), to: "Paid" }),
        }
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::Paid { .. })
    }
}

// ❌ Before: 깊은 중첩 — 흐름 파악이 어려움
fn process(order: &Order) -> Result<Receipt, Error> {
    if order.is_valid() {
        if order.has_items() {
            if order.customer.is_active() {
                // 실제 로직이 3단계 안에
                Ok(Receipt::new(order))
            } else { Err(Error::InactiveCustomer) }
        } else { Err(Error::EmptyOrder) }
    } else { Err(Error::InvalidOrder) }
}

// ✅ After-B: Early Return — 흐름이 선형 (coding-style.md §2.1)
fn process(order: &Order) -> Result<Receipt, Error> {
    if !order.is_valid()          { return Err(Error::InvalidOrder); }
    if !order.has_items()         { return Err(Error::EmptyOrder); }
    if !order.customer.is_active(){ return Err(Error::InactiveCustomer); }

    Ok(Receipt::new(order))
}
```

---

### [R-R-04] 함수 분해 & 단일 책임

**coding-style.md 근거**: §2.1 구조, §1.4 점진적 진화

**적용 시점**: 함수가 50줄을 초과하거나, 여러 책임을 가지거나, 명령형 루프에 filter+map 패턴이 보일 때

```rust
// ❌ Before: 한 함수가 집계·필터·변환·포맷팅을 모두 처리
fn generate_report(orders: &[Order]) -> String {
    let mut active = Vec::new();
    for o in orders {
        if o.status == OrderStatus::Active { active.push(o); }
    }
    let mut total = 0i64;
    for o in &active { total += o.amount_cents; }
    let mut ids = Vec::new();
    for o in &active {
        if !ids.contains(&o.customer_id) { ids.push(o.customer_id); }
    }
    format!("count={}, total={}, customers={}", active.len(), total, ids.len())
}

// ✅ After: 단일 책임 함수 분해 + Iterator 체인 (코드 길이 절반)
use std::collections::HashSet;

struct ActiveOrderSummary {
    count: usize,
    total_cents: i64,
    unique_customers: usize,
}

fn summarize_active_orders(orders: &[Order]) -> ActiveOrderSummary {
    let (count, total_cents, customers) = orders.iter()
        .filter(|o| o.status == OrderStatus::Active)
        .fold(
            (0usize, 0i64, HashSet::new()),
            |(count, total, mut customers), o| {
                customers.insert(o.customer_id);
                (count + 1, total + o.amount_cents, customers)
            },
        );

    ActiveOrderSummary { count, total_cents, unique_customers: customers.len() }
}

fn format_report(summary: &ActiveOrderSummary) -> String {
    format!(
        "count={}, total={}, customers={}",
        summary.count, summary.total_cents, summary.unique_customers,
    )
}

// 자주 쓰는 Iterator 패턴 참고
fn iterator_patterns_reference(items: &[Item]) {
    let first   = items.iter().find(|i| i.is_valid());
    let all_ok  = items.iter().all(|i| i.is_valid());
    let any_bad = items.iter().any(|i| i.is_expired());
    let ids: HashSet<_> = items.iter().map(|i| i.id).collect();
    let tags: Vec<_>    = items.iter().flat_map(|i| i.tags.iter()).collect();
}
```

---

### [R-R-05] 중복 제거 & 적시 추상화

**coding-style.md 근거**: §2.3 추상화, §1.1 변화 용이성 우선

**적용 시점**: 동일한 패턴이 3번 이상 반복될 때 (Rule of Three). 2번 이하라면 중복을 허용한다.

```rust
// ❌ Before: 동일 Repository 패턴이 Order·User·Product 세 곳에 각각 존재
//           → Rule of Three 충족, 추상화 시점
impl OrderService {
    async fn get_order(&self, id: OrderId) -> Result<Order, ServiceError> {
        self.db.find_order_by_id(id).await?
            .ok_or(ServiceError::NotFound)
    }
}
impl UserService {
    async fn get_user(&self, id: UserId) -> Result<User, ServiceError> {
        self.db.find_user_by_id(id).await?
            .ok_or(ServiceError::NotFound)  // 동일 패턴
    }
}
// Product도 동일 패턴 → 3번째 반복 → 추상화 도입

// ✅ After-A: Repository Trait으로 패턴 추출 (Rule of Three 적용)
#[async_trait::async_trait]
pub trait Repository<T, Id>: Send + Sync {
    async fn find_by_id(&self, id: Id) -> Result<Option<T>, RepositoryError>;
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;
}

// 각 도메인 별칭 (명시성 유지)
pub trait OrderRepository: Repository<Order, OrderId> {}
pub trait UserRepository: Repository<User, UserId> {}

// ❌ Before: 성급한 추상화 — 단 한 곳에서만 쓰이는 Trait
pub trait Formatter {
    fn format(&self, data: &str) -> String;
}
pub struct JsonFormatter;
impl Formatter for JsonFormatter { ... }  // JsonFormatter만 존재

// ✅ After-B: 추상화 제거, 직접 구현 (YAGNI, coding-style.md §2.3)
fn format_as_json(data: &str) -> String { ... }
// Trait이 필요한 두 번째 구현체가 생기면 그때 Trait 도입
```

> **규칙**: 추상화는 **최소 3번의 반복 이후**에 도입한다. 코드가 두 곳에 있으면 복사를 유지한다. 세 곳이 되면 그때 추상화한다 (coding-style.md §1.1).

---

### [R-R-06] 경계 조건 & 에러 처리 명시화

**coding-style.md 근거**: §5 에지 케이스 & 경계 조건, §1.2 의도를 드러내는 코드

**적용 시점**: `.unwrap()` / 인덱스 직접 접근 / `unwrap_or_default()`로 경계 조건을 암묵적으로 처리할 때

```rust
// ❌ Before: 침묵하는 실패·암묵적 경계 처리 (coding-style.md §5.4)
fn get_first_admin(users: &[User]) -> String {
    users[0].name.clone()               // 빈 슬라이스면 패닉
}

fn parse_config(raw: &str) -> Config {
    serde_json::from_str(raw).unwrap()  // 파싱 실패 시 패닉
}

fn find_active_user(users: &[User], id: u64) -> User {
    users.iter()
        .find(|u| u.id == id)
        .cloned()
        .unwrap_or_default()            // 실패가 침묵 — 기본값이 의미 있는가?
}

// ✅ After: 경계 조건을 도메인 로직으로 명시 (coding-style.md §5.2)
fn get_first_admin(users: &[User]) -> Result<&User, DomainError> {
    users.iter()
        .find(|u| u.is_admin())
        .ok_or(DomainError::NoAdminFound)   // 명시적: "어드민 없음"은 도메인의 일부
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("설정 파싱 실패")]           // 외부 노출용 — 내부 정보 제외
    ParseFailed(#[source] serde_json::Error),
    #[error("필수 설정 누락: {field}")]
    MissingField { field: &'static str },
}

fn parse_config(raw: &str) -> Result<Config, ConfigError> {
    let config: Config = serde_json::from_str(raw)
        .map_err(ConfigError::ParseFailed)?;

    if config.api_key.is_empty() {
        return Err(ConfigError::MissingField { field: "api_key" });
    }
    Ok(config)
}

fn find_active_user(users: &[User], id: UserId) -> Result<&User, DomainError> {
    users.iter()
        .find(|u| u.id == id && u.is_active())
        .ok_or(DomainError::UserNotFound(id))  // "없음"이 명시적 에러
}
```

> **에러 메시지 보안**: `#[error("...")]` 외부 노출 메시지에 DB 쿼리·파일 경로·스택 정보를 포함하지 않는다. `#[source]`로 내부 원인을 분리해 로그에만 기록한다 (`security.md §5 에러&로그` + `security-rust.md §6 에러 처리` 참조).

---

### [R-R-07] 소유권 & 변경 용이성

**coding-style.md 근거**: §1.1 변화 용이성 우선, §2.1 구조

**적용 시점**: 컴파일러 오류를 `.clone()`으로 회피하거나, 함수 파라미터에 `String`/`Vec<T>`를 소유권 이전 없이 사용할 때

```rust
// ❌ Before: 불필요한 clone — 소유권 이해 없이 컴파일 오류 회피
fn process_orders(orders: Vec<Order>) -> Vec<String> {
    orders.clone()                              // Vec 전체 복사
        .into_iter()
        .filter(|o| !o.id.clone().is_empty())   // 불필요 복사
        .map(|o| o.description.clone())         // 또 복사
        .collect()
}

fn get_label(name: String) -> String {          // String 소유권 이전 강제
    format!("Label: {name}")
}

// ✅ After: 참조 우선 — 호출자가 소유권 선택
fn process_orders(orders: &[Order]) -> Vec<&str> {  // &[T]·&str 우선
    orders.iter()
        .filter(|o| !o.id.is_empty())
        .map(|o| o.description.as_str())            // 복사 없음
        .collect()
}

fn get_label(name: &str) -> String {                // &str 파라미터
    format!("Label: {name}")
}

// Clone이 진짜 필요한 경우:
// - Arc<T> 참조 카운트 증가 (cheap, O(1))
// - 데이터를 여러 스레드에 독립적으로 전달
// - 원본 보존 + 변경된 복사본 필요 (immutable 설계 원칙)

// ✅ 소유권이 필요한 경우와 참조로 충분한 경우 기준:
// 파라미터: &str / &[T] 우선 (호출자 선택)
// 구조체 필드: String / Vec<T> (소유 필요)
// 반환값: 가능하면 &str / &[T] — 호출자가 .to_string() 선택
// async 컨텍스트: 'static 필요 시 Arc<T> 또는 소유형

// ❌ 라이프타임 오류를 clone으로 회피 — 설계 재검토 신호
fn find_name<'a>(items: &'a [Item], key: &str) -> Option<&'a str> {
    // ❌: .map(|i| i.name.clone())  — 복사 없이 참조 반환 가능
    items.iter().find(|i| i.key == key).map(|i| i.name.as_str())
}
```

---

### [R-R-08] 모듈 구조 도메인화

**coding-style.md 근거**: §1.3 도메인 중심 설계, §2.1 구조

**적용 시점**: `src/` 직하 파일이 10개 이상이거나, `utils.rs`·`models.rs`·`services.rs` 처럼 기능 단위로 flat하게 구성되어 있을 때

```
❌ Before: 기능 단위 flat 구조 — 도메인 경계 없음
src/
├── main.rs
├── db.rs          ← DB 로직 전부
├── models.rs      ← 모든 도메인 모델
├── handlers.rs    ← 모든 HTTP 핸들러
├── services.rs    ← 모든 비즈니스 로직
└── utils.rs       ← 잡다한 유틸리티 (도메인 없는 코드)

✅ After: 도메인 단위 계층 구조 — 경계가 코드 구조에 드러남
src/
├── main.rs                          ← 앱 초기화, DI 조립
├── domain/                          ← 순수 비즈니스 로직 (외부 의존 없음)
│   ├── order/
│   │   ├── model.rs                 ← Order, OrderItem, OrderStatus
│   │   ├── repository.rs            ← OrderRepository trait
│   │   ├── service.rs               ← OrderService
│   │   └── error.rs                 ← OrderError
│   └── user/
│       ├── model.rs                 ← User, Email
│       ├── repository.rs            ← UserRepository trait
│       └── service.rs               ← UserService
├── infra/                           ← 외부 시스템 구현체
│   ├── db/
│   │   ├── order_repository.rs      ← impl OrderRepository
│   │   └── user_repository.rs
│   └── http/
│       ├── router.rs
│       ├── order_handler.rs
│       └── middleware.rs
└── shared/                          ← 크로스 컷팅 관심사
    ├── errors.rs                    ← 공통 에러 타입
    ├── types.rs                     ← OrderId, UserId (Newtype)
    └── config.rs                    ← AppConfig
```

```rust
// src/domain/order/mod.rs — pub 노출 최소화
mod error;
mod model;
mod repository;
mod service;

pub use error::OrderError;
pub use model::{Order, OrderId, OrderStatus};
pub use repository::OrderRepository;
pub use service::OrderService;
// 내부 구현 세부사항은 외부에 노출하지 않음

// 공개 범위 원칙 (coding-style.md §2.1):
// pub(super) ← 부모 모듈만 접근
// pub(crate) ← 크레이트 내부만 접근
// pub        ← 외부 공개 (최소화 — 실제로 필요한 경우만)
```

> **utils.rs 처리**: `utils.rs`에서 도메인 로직을 발견하면 해당 도메인 모듈로 이동한다. 도메인 개념이 없는 순수 기술 유틸리티(날짜 포맷터, HTTP 클라이언트 래퍼)만 `shared/`에 남긴다.

---

## PHASE 3 — 리팩토링 실행 워크플로우

### 3-1. 단계별 실행 순서

```bash
# Step 1: 기준선 측정 (PHASE 0 체크리스트 완료 후)
# Step 2: 포맷 통일 (가장 안전 — 로직 변경 없음)
cargo fmt
git commit -m "style: cargo fmt 전체 적용"

# Step 3: Clippy 자동 수정
cargo clippy --fix --allow-dirty
cargo test
git commit -m "refactor: clippy 자동 수정 적용"

# Step 4: 수동 리팩토링 (카탈로그 항목 순서대로)
# → 각 항목 완료 후 즉시 검증:
cargo test
cargo clippy -- -D warnings

# Step 5: 성능 회귀 확인
cargo bench 2>&1 | tee bench_after.txt

# Step 6: 최종 커버리지 확인
cargo tarpaulin --out Html --output-dir coverage/

# Step 7: 최종 커밋 + PR
git commit -m "refactor(order): [R-R-03] OrderStatus bool 플래그 → Enum 상태 머신"
```

### 3-2. 커밋 메시지 컨벤션

```
형식: refactor(<scope>): [R-R-XX] <50자 이내 요약>

예시:
refactor(order):   [R-R-01] 매직 넘버 상수 추출, 함수명 도메인 용어로 변경
refactor(user):    [R-R-02] UserId Newtype + Smart Constructor 도입
refactor(order):   [R-R-03] OrderStatus bool 플래그 → Enum 상태 머신
refactor(report):  [R-R-04] generate_report 단일 책임 함수로 분해
refactor(domain):  [R-R-05] OrderRepository Trait 추출 (Rule of Three 충족)
refactor(config):  [R-R-06] unwrap 제거, ConfigError 도메인 에러 타입 도입
refactor(user):    [R-R-07] UserList 처리에서 불필요한 clone() 제거
refactor(arch):    [R-R-08] flat src/ → domain/infra/shared 계층 분리
```

### 3-3. PR 체크리스트

```
기능 동치:
□ cargo test — before와 동일하거나 개선
□ 공개 API 시그니처 변경 없음 (파괴적 변경은 별도 PR)
□ 에러·로그 동작 불변

코드 품질:
□ cargo clippy -- -D warnings — 경고 0건
□ cargo fmt --check — 포맷 위반 없음
□ 커버리지 기준선(80%) 유지 또는 향상
□ 도메인 개념이 리팩토링 전보다 명확히 드러나는가?

안전성:
□ unsafe 블록 추가 없음 (불가피한 경우 // SAFETY: 주석 필수)
□ unwrap/expect 추가 없음 (라이브러리 코드)
□ 데이터 레이스 가능성 없음 (Send/Sync 경계 확인)

성능:
□ cargo bench 기준선 ±5% 이내
□ 불필요한 힙 할당 미도입
□ async 컨텍스트에서 blocking 코드 없음

설계:
□ pub 노출 최소화 (pub(crate) / pub(super) 우선)
□ pub 아이템 rustdoc 주석 업데이트
```

---

## PHASE 4 — 대규모 Rust 리팩토링 전략

### 4-1. Strangler Fig Pattern — 크레이트 단위 점진적 교체

```
레거시 크레이트와 신규 크레이트 병존:

workspace/
├── Cargo.toml       ← workspace 정의
├── legacy-core/     ← 기존 구현 (건드리지 않음)
├── core-v2/         ← 신규 구현
└── app/             ← Feature Flag로 v1/v2 전환

// app/src/main.rs
#[cfg(feature = "new-core")]
use core_v2::OrderService;

#[cfg(not(feature = "new-core"))]
use legacy_core::OrderService;
```

### 4-2. Branch by Abstraction — Trait로 구현체 교체

```rust
// 1단계: 기존 구현 위에 Trait 추출
pub trait StorageBackend: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), StorageError>;
}

// 2단계: 기존 구현체에 Trait 구현
impl StorageBackend for LegacyRocksDb { ... }

// 3단계: 신규 구현체 작성
impl StorageBackend for NewSledDb { ... }

// 4단계: Cargo.toml feature로 전환
// [features]
// default = ["new-storage"]
// new-storage = []
```

### 4-3. Mikado Method — 컴파일 오류 기반 의존성 정리

```
Rust 컴파일러가 Mikado 다이어그램을 자동 생성:

1. 목표 변경 시도 → cargo build 실패
2. 오류 메시지 목록 = 선행 조건 목록
3. error[E0308] / error[E0277] 등 오류 코드로 분류
4. 가장 깊은 의존 오류부터 수정
5. 각 성공마다: cargo test && git commit

팁: cargo check가 cargo build보다 빠름 — 의존성 탐색 시 활용
```

---

## PHASE 5 — 금지 패턴 (Anti-Patterns)

coding-style.md §9 안티 패턴의 Rust 구현 기준:

```
❌ 성급한 추상화 (coding-style.md §9)
   → 동일 패턴이 3번 나타나기 전에 추상화하지 않는다 (Rule of Three).

❌ 빈약한 도메인 모델 (coding-style.md §9)
   → 데이터만 있는 struct 금지. 행동을 엔티티 내부에 캡슐화한다 (R-R-02).

❌ Primitive 집착 — u64, String으로 도메인 개념 표현
   → Newtype + Smart Constructor 사용 (R-R-02).

❌ 깊은 중첩 구조 (coding-style.md §9)
   → 중첩 3단계 이상 금지. Early Return / exhaustive match 사용 (R-R-03).

❌ 암묵적 경계 처리 (coding-style.md §9)
   → .unwrap(), 인덱스 무방비 접근, unwrap_or_default() 남용 금지 (R-R-06).

❌ 테스트 없는 핵심 로직 (coding-style.md §9)
   → 도메인 행동 변경 리팩토링에는 반드시 테스트가 선행된다.

❌ 도메인 없는 util 모듈 (coding-style.md §9)
   → utils.rs에 비즈니스 로직 금지. 도메인 모듈로 이동한다 (R-R-08).

❌ 라이프타임 오류를 .clone()으로 회피
   → 원인 분석 후 소유권 설계 재검토 (R-R-07).

❌ panic!() / unwrap()을 에러 처리 대신 사용
   → 라이브러리: 절대 금지. 바이너리 main(): 허용 (R-R-06).

❌ Arc<Mutex<T>>로 모든 공유 상태 처리
   → 대안: 메시지 패싱(tokio::mpsc), DashMap, RwLock.

❌ Box<dyn Trait> 남용 (동적 디스패치)
   → 대안: 제네릭 <T: Trait> (정적 디스패치, 컴파일 타임 단형화).

❌ 에러 타입 없이 Box<dyn Error>로 통일
   → thiserror로 도메인 의미를 가진 구체적 에러 타입 정의 (R-R-06).

❌ 내부 Repository·UseCase를 mockall로 모킹
   → 실제 DB 통합 테스트로 대체. 모킹은 외부 API 경계에서만 (rust-test-style.md).

❌ 기능 추가와 리팩토링 동시 진행 (coding-style.md §9)
   → 한 PR은 하나의 의도만 담는다.
```

---

## 참고 자료

| 항목 | 도구/자료 |
|------|-----------|
| 정적 분석 | Clippy (`cargo clippy`) |
| 코드 포맷 | rustfmt (`cargo fmt`) |
| 테스트 커버리지 | cargo-tarpaulin |
| 벤치마킹 | Criterion (`cargo bench`) |
| 에러 처리 | thiserror, anyhow |
| 비동기 | tokio, async-trait |
| 병렬 처리 | rayon, tokio |
| IDE | rust-analyzer (VS Code, CLion, Helix) |
| 핵심 서적 | *Rust for Rustaceans* — Jon Gjengset |
| 핵심 서적 | *Programming Rust* — Blandy & Orendorff (2nd Ed.) |
| 패턴 참고 | rust-unofficial.github.io/patterns |
| API 가이드 | rust-lang.github.io/api-guidelines |
| 코딩 스타일 | `.claude/rules/coding-style.md` |
| 테스트 규칙 | `.claude/rules/rust-test-style.md` |
| 보안 규칙 (공통) | `.claude/rules/security.md` |
| 보안 규칙 (Rust 전용) | `.claude/rules/security-rust.md` |
