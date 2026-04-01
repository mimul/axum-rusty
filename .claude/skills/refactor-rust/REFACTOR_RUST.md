---
name: refactor-rust
description: >
  운영 중인 Rust 코드를 안전하고 체계적으로 전체 리팩토링할 때 사용하는 스킬.
  소유권(Ownership) 개선, 에러 처리 체계화, Trait 추상화, 모듈 구조 재편 등
  Rust 전용 리팩토링 전 과정을 가이드한다.
  트리거: "Rust 리팩토링", "Rust 코드 정리", "cargo clippy 개선",
  "소유권 개선", "Rust refactor", "Rust clean up" 등.
---

# REFACTOR SKILL — Rust 운영 코드 리팩토링 가이드

## 개요

이 스킬은 **현재 운영 중인(live) Rust 코드**를 기능 변경 없이 내부 구조를 개선하는 작업을 다룬다.
Rust 리팩토링은 타입 시스템과 Borrow Checker를 최대한 활용해 **컴파일 타임에 버그를 제거**하는 것이 핵심 목표다.

### 핵심 원칙
- **기능 동치(Behavioral Equivalence)**: 외부 동작은 100% 동일하게 유지
- **소규모 증분(Small Increments)**: 한 번에 하나의 변환만 수행
- **항상 그린(Always Green)**: 매 단계마다 `cargo test` 통과 확인
- **컴파일러를 동반자로(Compiler as Partner)**: 컴파일 오류 메시지를 설계 가이드로 활용
- **되돌릴 수 있게(Reversible)**: Git 커밋 단위를 작게 유지

---

## PHASE 0 — 리팩토링 전 준비

### 0-1. 안전망 확보

```bash
체크리스트:
□ cargo test — 전체 테스트 통과 확인 및 결과 저장
□ cargo clippy -- -D warnings — 현재 경고 목록 저장
□ cargo fmt --check — 포맷 위반 확인
□ cargo tarpaulin --out Html — 커버리지 기준선 측정 (목표: 80% 이상)
□ cargo bench — 성능 기준선 측정 (Criterion 사용)
□ git tag refactor/before-start — 현재 상태 태그 생성
```

### 0-2. Rust 전용 코드 냄새 탐지 항목

| 냄새 유형 | 증상 | Clippy Lint |
|-----------|------|-------------|
| Clone 남용 | `.clone()` 이 불필요하게 많음 | `clippy::clone_on_ref_ptr` |
| unwrap 남용 | `.unwrap()` / `.expect()` 가 라이브러리 코드에 존재 | `clippy::unwrap_used` |
| 원시 타입 집착 | `u64`/`String`으로 도메인 개념 표현 | 수동 검토 |
| 거대 함수 | 함수 50줄 초과 | `clippy::too_many_lines` |
| 중복 코드 | 동일 이터레이터 체인 반복 | 수동 검토 |
| 블로킹 I/O | `std::thread::sleep`, blocking reqwest | `clippy::async_yields_async` |
| 불필요한 힙 할당 | `Box<T>` 과용, `String` vs `&str` 혼용 | `clippy::box_default` |
| 잘못된 공유 | `Rc<T>` 멀티스레드 환경 사용 | 컴파일 오류 |
| 매직 넘버 | 의미 불명확한 숫자 리터럴 | `clippy::unreadable_literal` |
| 과도한 중첩 | match/if let 3단계 이상 중첩 | `clippy::collapsible_match` |

### 0-3. 리팩토링 우선순위 매트릭스

```
영향도(Impact) vs 위험도(Risk) 2×2 매트릭스:

         낮은 위험         높은 위험
높은 영향  [1순위]          [3순위]
낮은 영향  [2순위]          [4순위 / 보류]

Rust에서 낮은 위험 변경 예시:
  - cargo fmt (포맷만)
  - 변수명 변경
  - unwrap → expect (메시지 추가)
  - 이터레이터 체인 정리

Rust에서 높은 위험 변경 예시:
  - 소유권 구조 변경 (참조 → 소유, 반대)
  - async/await 도입
  - 공개 Trait 시그니처 변경
  - 모듈 계층 재편 (pub use 영향)
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
once_cell = "1.19"       # 정적 초기화

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
mockall = "0.12"         # Trait Mock 생성
tokio-test = "0.4"       # 비동기 테스트 유틸리티

[[bench]]
name = "my_bench"
harness = false          # Criterion 사용 시 필수

[profile.release]
lto = true               # Link-Time Optimization
codegen-units = 1        # 최적화 극대화
strip = true             # 심볼 제거 (바이너리 크기 축소)

[profile.dev]
opt-level = 0
debug = true
```

### 1-2. Clippy 설정 (clippy.toml 또는 소스 상단)

```rust
// src/lib.rs 또는 src/main.rs 최상단
#![warn(
    // 기본 Clippy lints
    clippy::all,
    clippy::pedantic,
    // 중요 개별 lints
    clippy::unwrap_used,          // unwrap() 사용 금지
    clippy::expect_used,          // expect() 사용 제한
    clippy::clone_on_ref_ptr,     // Rc/Arc clone 경고
    clippy::todo,                 // todo!() 잔존 경고
    clippy::unimplemented,        // unimplemented!() 잔존 경고
    clippy::dbg_macro,            // dbg!() 잔존 경고
    clippy::print_stdout,         // println!() 라이브러리 사용 경고
    // 코드 품질
    missing_docs,                 // 공개 아이템 문서화 강제
    dead_code,                    // 미사용 코드 경고
    unused_imports,               // 미사용 임포트 경고
    unused_variables,             // 미사용 변수 경고
)]
```

### 1-3. rustfmt.toml 설정

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
imports_granularity = "Module"   # use 구문 자동 정리
group_imports = "StdExternalCrate"
```

### 1-4. 필수 Cargo 명령어 참조

```bash
# 분석
cargo clippy -- -D warnings          # 경고를 오류로 처리
cargo clippy --fix --allow-dirty     # 자동 수정 가능한 항목 수정
cargo fmt                            # 포맷 자동 적용
cargo fmt --check                    # 포맷 위반 확인만 (CI용)

# 테스트
cargo test                           # 전체 테스트
cargo test -- --nocapture            # println! 출력 표시
cargo test order::tests              # 특정 모듈 테스트

# 커버리지 (cargo-tarpaulin 설치 필요)
cargo tarpaulin --out Html --output-dir coverage/

# 벤치마킹
cargo bench                          # 전체 벤치마크
cargo bench --bench my_bench         # 특정 벤치마크
cargo bench -- --baseline before     # 기준선 비교

# 문서
cargo doc --no-deps --open           # 문서 생성 및 열기

# 의존성 감사
cargo audit                          # 보안 취약점 확인
cargo outdated                       # 오래된 의존성 확인
```

---

## PHASE 2 — Rust 리팩토링 카탈로그

### [R-R-01] Ownership 명확화 — Clone 남용 제거

**적용 시점**: `.clone()` 이 컴파일 오류를 피하려고 무분별하게 사용될 때

```rust
// ❌ Before: clone() 남발로 불필요한 힙 할당 발생
fn process_users(users: Vec<User>) -> Vec<String> {
    users.clone()           // Vec 전체 복사
        .into_iter()
        .filter(|u| !u.name.clone().is_empty())  // String 복사
        .map(|u| u.email.clone().to_lowercase())  // 또 복사
        .collect()
}

// ✅ After-A: 슬라이스 참조 + 문자열 슬라이스 활용
fn process_users(users: &[User]) -> Vec<String> {
    users.iter()
        .filter(|u| !u.name.is_empty())       // &str — 복사 없음
        .map(|u| u.email.to_lowercase())       // 한 번만 String 생성
        .collect()
}

// ✅ After-B: 소유권이 실제로 필요한 경우 (호출자가 더 이상 사용 안 함)
fn process_users_owned(users: Vec<User>) -> Vec<String> {
    users.into_iter()                          // 소유권 이전 (복사 없음)
        .filter(|u| !u.name.is_empty())
        .map(|u| u.email.to_lowercase())       // u 소유권 사용
        .collect()
}

// Clone이 진짜 필요한 경우:
// - Rc<T> / Arc<T> 의 참조 카운트 증가 (cheap)
// - 데이터를 여러 스레드에 독립적으로 전달할 때
// - 원본을 보존하면서 변경된 복사본이 필요할 때
```

---

### [R-R-02] Error Handling 체계화 — unwrap/expect 제거

**적용 시점**: 라이브러리 코드 또는 운영 코드에 `.unwrap()` / `.expect()` 가 존재할 때

```rust
// ❌ Before: panic 유발 — 운영 환경에서 프로세스 종료
fn load_user_config(path: &str) -> UserConfig {
    let content = std::fs::read_to_string(path).unwrap();
    let config: UserConfig = serde_json::from_str(&content).expect("파싱 실패");
    config
}

// ✅ After: thiserror로 타입화된 에러 + ? 연산자

// 1. 에러 타입 정의
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("설정 파일 읽기 실패 ({path}): {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("설정 JSON 파싱 실패: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("필수 설정 누락: {field}")]
    MissingField { field: &'static str },
}

// 2. ? 연산자로 에러 전파
fn load_user_config(path: &str) -> Result<UserConfig, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::FileRead {
            path: path.to_string(),
            source: e,
        })?;

    let config: UserConfig = serde_json::from_str(&content)?; // From 자동 변환

    if config.api_key.is_empty() {
        return Err(ConfigError::MissingField { field: "api_key" });
    }

    Ok(config)
}

// 3. 애플리케이션 진입점: anyhow로 다양한 에러 통합
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_user_config("config.json")
        .context("앱 초기화 중 설정 로드 실패")?;

    run_app(config)?;
    Ok(())
}

// 4. 테스트에서 명확한 에러 타입 검증
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_returns_file_read_error() {
        let result = load_user_config("nonexistent.json");
        assert!(matches!(result, Err(ConfigError::FileRead { .. })));
    }
}
```

---

### [R-R-03] Newtype Pattern — 원시 타입 도메인화

**적용 시점**: 동일 타입의 파라미터가 여러 개 나열되거나, 컴파일러가 의미 오류를 잡지 못할 때

```rust
// ❌ Before: u64 두 개 — 순서 바뀌어도 컴파일 통과
fn transfer_funds(from_account: u64, to_account: u64, amount_cents: i64) { ... }
transfer_funds(to_id, from_id, amount);  // 버그: 출금/입금 계좌 반전

// ✅ After: Newtype으로 컴파일 타임 안전성 확보
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(u64);

impl AccountId {
    pub fn new(id: u64) -> Self { Self(id) }
    pub fn value(self) -> u64 { self.0 }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Account({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AmountCents(i64);

impl AmountCents {
    pub fn new(cents: i64) -> Result<Self, &'static str> {
        if cents < 0 {
            Err("금액은 0 이상이어야 합니다")
        } else {
            Ok(Self(cents))
        }
    }

    pub fn value(self) -> i64 { self.0 }

    pub fn from_won(won: i64) -> Result<Self, &'static str> {
        Self::new(won)  // 원화 = 센트 단위 (필요 시 환율 변환 추가)
    }
}

// 이제 순서 바꾸면 컴파일 오류 발생
fn transfer_funds(from: AccountId, to: AccountId, amount: AmountCents)
    -> Result<(), TransferError> { ... }

// transfer_funds(to_id, from_id, amount);  // ✅ 컴파일 오류로 버그 차단
transfer_funds(from_id, to_id, amount);     // 정상
```

---

### [R-R-04] Trait 기반 추상화 — 구체 타입 의존 제거

**적용 시점**: 구체 타입이 직접 사용되어 단위 테스트 시 실제 DB/외부 서비스 필요할 때

```rust
// ❌ Before: 구체 타입에 직접 의존
pub struct OrderService {
    db: PostgresOrderDb,  // 테스트 시 실제 Postgres 필요
}

// ✅ After: Trait 정의 + 제네릭 의존성 주입

// 1. Repository Trait 정의
#[async_trait::async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, RepositoryError>;
    async fn save(&self, order: &Order) -> Result<(), RepositoryError>;
    async fn delete(&self, id: OrderId) -> Result<(), RepositoryError>;
}

// 2. 실제 구현체
pub struct PostgresOrderRepository {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, RepositoryError> {
        sqlx::query_as!(Order, "SELECT * FROM orders WHERE id = $1", id.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::from)
    }

    async fn save(&self, order: &Order) -> Result<(), RepositoryError> {
        // upsert 구현
        todo!()
    }

    async fn delete(&self, id: OrderId) -> Result<(), RepositoryError> {
        todo!()
    }
}

// 3. 제네릭 서비스 — 컴파일 타임 단형화(monomorphization)로 런타임 오버헤드 없음
pub struct OrderService<R: OrderRepository> {
    repository: R,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_order(&self, id: OrderId) -> Result<Order, ServiceError> {
        self.repository
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(id))
    }
}

// 4. 테스트: mockall로 Mock 자동 생성
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    mockall::mock! {
        pub OrderRepositoryMock {}

        #[async_trait::async_trait]
        impl OrderRepository for OrderRepositoryMock {
            async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, RepositoryError>;
            async fn save(&self, order: &Order) -> Result<(), RepositoryError>;
            async fn delete(&self, id: OrderId) -> Result<(), RepositoryError>;
        }
    }

    #[tokio::test]
    async fn get_order_returns_not_found_when_missing() {
        let mut mock = MockOrderRepositoryMock::new();
        let target_id = OrderId::new(999);

        mock.expect_find_by_id()
            .with(eq(target_id))
            .returning(|_| Ok(None));

        let service = OrderService::new(mock);
        let result = service.get_order(target_id).await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }
}
```

---

### [R-R-05] Iterator Adapter 체인 — 명령형 루프 → 함수형

**적용 시점**: 명령형 for 루프에 filter + map + collect 패턴이 보일 때

```rust
// ❌ Before: 명령형 루프 + 중간 컬렉션 반복 생성
fn summarize_active_orders(orders: &[Order]) -> OrderSummary {
    let mut active = Vec::new();
    for order in orders {
        if order.status == OrderStatus::Active {
            active.push(order);
        }
    }

    let mut total = 0i64;
    for order in &active {
        total += order.amount_cents;
    }

    let mut customer_ids = Vec::new();
    for order in &active {
        if !customer_ids.contains(&order.customer_id) {
            customer_ids.push(order.customer_id);
        }
    }

    OrderSummary { count: active.len(), total_cents: total, unique_customers: customer_ids.len() }
}

// ✅ After: 단일 패스 이터레이터 — 중간 컬렉션 없음
use std::collections::HashSet;

fn summarize_active_orders(orders: &[Order]) -> OrderSummary {
    let (count, total_cents, unique_customers) = orders.iter()
        .filter(|o| o.status == OrderStatus::Active)
        .fold(
            (0usize, 0i64, HashSet::new()),
            |(count, total, mut customers), order| {
                customers.insert(order.customer_id);
                (count + 1, total + order.amount_cents, customers)
            },
        );

    // 또는 분리된 집계 (가독성 vs 성능 트레이드오프)
    let active: Vec<_> = orders.iter()
        .filter(|o| o.status == OrderStatus::Active)
        .collect();

    OrderSummary {
        count,
        total_cents,
        unique_customers: unique_customers.len(),
    }
}

// 자주 쓰는 이터레이터 패턴 모음
fn iterator_patterns(items: &[Item]) {
    // 첫 번째 매칭 항목
    let first = items.iter().find(|i| i.is_valid());

    // 모든 항목이 조건 만족하는지
    let all_valid = items.iter().all(|i| i.is_valid());

    // 어느 항목이라도 조건 만족하는지
    let any_urgent = items.iter().any(|i| i.is_urgent());

    // 그룹화 (itertools 크레이트)
    // use itertools::Itertools;
    // let by_status = items.iter().group_by(|i| i.status);

    // 중복 제거
    let unique_ids: HashSet<_> = items.iter().map(|i| i.id).collect();

    // 평탄화
    let all_tags: Vec<_> = items.iter().flat_map(|i| i.tags.iter()).collect();

    // 인덱스와 함께
    items.iter().enumerate().for_each(|(idx, item)| {
        println!("[{idx}] {item:?}");
    });
}
```

---

### [R-R-06] State Machine with Enum — Bool 플래그 → 타입 안전 상태

**적용 시점**: 여러 bool 필드나 문자열로 상태를 표현하고, 불가능한 상태 조합이 존재할 때

```rust
// ❌ Before: bool 플래그 조합 — 논리적 불가능 상태가 타입으로 방지되지 않음
pub struct Order {
    pub id: u64,
    pub is_paid: bool,
    pub is_shipped: bool,
    pub is_delivered: bool,
    pub is_cancelled: bool,
    pub status: String,  // "pending" | "paid" | ... — 문자열과 bool 중복
}
// is_shipped = true && is_cancelled = true → 불가능하지만 타입으로 막을 수 없음

// ✅ After: Enum 상태 머신 — 불가능한 상태를 타입으로 배제 (Make Illegal States Unrepresentable)
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Paid {
        paid_at: DateTime<Utc>,
        transaction_id: String,
    },
    Shipped {
        shipped_at: DateTime<Utc>,
        tracking_number: String,
        courier: String,
    },
    Delivered {
        delivered_at: DateTime<Utc>,
    },
    Cancelled {
        cancelled_at: DateTime<Utc>,
        reason: CancellationReason,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CancellationReason {
    CustomerRequest,
    OutOfStock,
    PaymentFailed,
    Other(String),
}

pub struct Order {
    pub id: OrderId,
    pub customer_id: CustomerId,
    pub status: OrderStatus,
    pub items: Vec<OrderItem>,
}

impl Order {
    /// 상태 전이: Pending → Paid
    /// 유효하지 않은 전이는 컴파일 타임이 아닌 Result로 처리
    pub fn mark_as_paid(
        &mut self,
        transaction_id: String,
    ) -> Result<(), OrderError> {
        match &self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Paid {
                    paid_at: Utc::now(),
                    transaction_id,
                };
                Ok(())
            }
            other => Err(OrderError::InvalidTransition {
                from: format!("{other:?}"),
                to: "Paid".to_string(),
            }),
        }
    }

    /// 상태 전이: Paid → Shipped
    pub fn mark_as_shipped(
        &mut self,
        tracking_number: String,
        courier: String,
    ) -> Result<(), OrderError> {
        match &self.status {
            OrderStatus::Paid { .. } => {
                self.status = OrderStatus::Shipped {
                    shipped_at: Utc::now(),
                    tracking_number,
                    courier,
                };
                Ok(())
            }
            other => Err(OrderError::InvalidTransition {
                from: format!("{other:?}"),
                to: "Shipped".to_string(),
            }),
        }
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::Paid { .. })
    }
}

// 상태 기반 디스플레이
impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending              => write!(f, "결제 대기"),
            Self::Paid { .. }          => write!(f, "결제 완료"),
            Self::Shipped { tracking_number, .. } => write!(f, "배송 중 ({tracking_number})"),
            Self::Delivered { .. }     => write!(f, "배송 완료"),
            Self::Cancelled { reason, .. } => write!(f, "취소됨 ({reason:?})"),
        }
    }
}
```

---

### [R-R-07] Async/Await 도입 — Blocking I/O 제거

**적용 시점**: `std::thread::sleep`, `blocking` HTTP 클라이언트, 동기 파일 I/O가 핫 경로에 존재할 때

```rust
// ❌ Before: 스레드 블로킹 — 동시 요청 수 = 스레드 수에 제한
use std::time::Duration;

fn fetch_user_profile(id: u64) -> Result<UserProfile, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new(); // 매 호출마다 클라이언트 생성
    let resp = client
        .get(format!("https://api.example.com/users/{id}"))
        .timeout(Duration::from_secs(5))
        .send()?
        .json::<UserProfile>()?;
    Ok(resp)
}

fn process_users(ids: Vec<u64>) -> Vec<UserProfile> {
    ids.iter()
        .filter_map(|id| fetch_user_profile(*id).ok()) // 순차 실행
        .collect()
}

// ✅ After: async/await + 공유 클라이언트 + 병렬 실행

// 1. 공유 HTTP 클라이언트 (전역 초기화)
use once_cell::sync::Lazy;
use reqwest::Client;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(20)
        .build()
        .expect("HTTP 클라이언트 초기화 실패")
});

// 2. 비동기 개별 요청
async fn fetch_user_profile(id: u64) -> Result<UserProfile, reqwest::Error> {
    HTTP_CLIENT
        .get(format!("https://api.example.com/users/{id}"))
        .send()
        .await?
        .json::<UserProfile>()
        .await
}

// 3. 병렬 실행 (N개 동시 요청)
use futures::future::join_all;

async fn process_users(ids: Vec<u64>) -> Vec<UserProfile> {
    let futures = ids.iter().map(|&id| fetch_user_profile(id));
    let results = join_all(futures).await;

    results.into_iter()
        .filter_map(|r| r.ok())
        .collect()
}

// 4. 동시성 제한이 필요한 경우 (rate limiting)
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn process_users_limited(ids: Vec<u64>, concurrency: usize) -> Vec<UserProfile> {
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let futures = ids.into_iter().map(|id| {
        let sem = Arc::clone(&semaphore);
        async move {
            let _permit = sem.acquire().await.unwrap();
            fetch_user_profile(id).await
        }
    });

    join_all(futures).await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect()
}
```

---

### [R-R-08] Module 구조 재편 — Flat → 도메인 계층

**적용 시점**: `src/` 바로 아래 파일이 10개 이상이거나, 파일 간 순환 참조 경향이 보일 때

```
❌ Before: 기능별 flat 구조 — 책임 혼재
src/
├── main.rs
├── db.rs          ← DB 로직 전부 (연결 풀, 쿼리, 마이그레이션)
├── models.rs      ← 모든 도메인 모델
├── handlers.rs    ← 모든 HTTP 핸들러
├── services.rs    ← 모든 비즈니스 로직
└── utils.rs       ← 잡다한 유틸리티

✅ After: 도메인 중심 계층 구조
src/
├── main.rs                          ← 앱 초기화, DI 조립
├── domain/                          ← 순수 비즈니스 로직 (외부 의존 없음)
│   ├── mod.rs
│   ├── order/
│   │   ├── mod.rs
│   │   ├── model.rs                 ← Order, OrderItem, OrderStatus
│   │   ├── repository.rs            ← OrderRepository trait
│   │   ├── service.rs               ← OrderService<R: OrderRepository>
│   │   └── error.rs                 ← OrderError
│   └── user/
│       ├── mod.rs
│       ├── model.rs                 ← User, Email, PhoneNumber
│       ├── repository.rs            ← UserRepository trait
│       └── service.rs               ← UserService<R: UserRepository>
├── infrastructure/                  ← 외부 시스템 구현체
│   ├── mod.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── connection.rs            ← PgPool 초기화
│   │   ├── order_repository.rs      ← impl OrderRepository for PgOrderRepo
│   │   └── user_repository.rs
│   ├── http/
│   │   ├── mod.rs
│   │   ├── router.rs                ← Axum/Actix 라우터 설정
│   │   ├── order_handler.rs         ← HTTP 핸들러
│   │   └── middleware.rs            ← 인증, 로깅
│   └── cache/
│       ├── mod.rs
│       └── redis_client.rs
└── shared/                          ← 크로스 컷팅 관심사
    ├── mod.rs
    ├── errors.rs                    ← 공통 에러 타입
    ├── types.rs                     ← OrderId, UserId 등 공통 Newtype
    └── config.rs                    ← AppConfig

모듈 공개 범위 원칙:
pub(super)   ← 부모 모듈만 접근
pub(crate)   ← 크레이트 내부만 접근
pub          ← 외부 공개 (최소화)
```

```rust
// src/domain/order/mod.rs — 공개 API 제어
mod error;
mod model;
mod repository;
mod service;

pub use error::OrderError;
pub use model::{Order, OrderId, OrderItem, OrderStatus};
pub use repository::OrderRepository;
pub use service::OrderService;
// service의 내부 구현은 외부에 노출하지 않음
```

---

### [R-R-09] Lifetime 명확화 — 불필요한 복사 제거

**적용 시점**: String을 &str 대신 사용하거나, 라이프타임 오류를 clone으로 회피할 때

```rust
// ❌ Before: String 반환 — 항상 힙 할당 발생
struct Config {
    database_url: String,
    api_key: String,
}

impl Config {
    fn database_url(&self) -> String {
        self.database_url.clone()  // 불필요한 복사
    }
}

fn find_config_value<'a>(configs: &'a [Config], key: &str) -> String {
    configs.iter()
        .find(|c| c.api_key == key)
        .map(|c| c.api_key.clone())  // 또 복사
        .unwrap_or_default()
}

// ✅ After: &str 반환으로 불필요한 힙 할당 제거
impl Config {
    fn database_url(&self) -> &str {
        &self.database_url   // 라이프타임: Config가 살아있는 동안 유효
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }
}

// 라이프타임 명시: 반환값은 configs의 라이프타임에 묶임
fn find_config_value<'a>(configs: &'a [Config], key: &str) -> Option<&'a str> {
    configs.iter()
        .find(|c| c.api_key == key)
        .map(|c| c.api_key.as_str())  // 복사 없이 슬라이스 반환
}

// 소유권이 필요한 경우와 참조로 충분한 경우 구분:
// - 함수 파라미터: &str 우선 (호출자 선택)
// - 구조체 필드: String 필요 (소유 필요)
// - 반환값: 가능하면 &str (호출자가 필요 시 .to_string())
```

---

### [R-R-10] Const / Static 분리 — 매직 넘버/문자열 제거

**적용 시점**: 의미 불명확한 숫자 리터럴이나 문자열이 코드 곳곳에 하드코딩될 때

```rust
// ❌ Before: 매직 넘버와 하드코딩 문자열
fn validate_password(password: &str) -> bool {
    password.len() >= 8 && password.len() <= 72  // 72가 왜?
}

fn retry<F: Fn() -> Result<(), Error>>(f: F) -> Result<(), Error> {
    for _ in 0..3 {  // 3이 왜?
        if f().is_ok() { return Ok(()); }
        std::thread::sleep(std::time::Duration::from_millis(500));  // 500ms가 왜?
    }
    f()
}

// ✅ After: 의미 있는 상수로 자기 문서화(Self-Documenting Code)

/// 최소 비밀번호 길이 (NIST SP 800-63B 권고 기준)
const MIN_PASSWORD_LEN: usize = 8;

/// 최대 비밀번호 길이 (bcrypt 입력 제한)
const MAX_PASSWORD_LEN: usize = 72;

/// 외부 API 최대 재시도 횟수
const MAX_RETRY_COUNT: u32 = 3;

/// 재시도 간격 (지수 백오프 기준 초기값)
const RETRY_BASE_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

fn validate_password(password: &str) -> bool {
    (MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&password.len())
}

fn retry<F, E>(f: F) -> Result<(), E>
where
    F: Fn() -> Result<(), E>,
{
    for attempt in 0..MAX_RETRY_COUNT {
        match f() {
            Ok(_) => return Ok(()),
            Err(_) if attempt < MAX_RETRY_COUNT - 1 => {
                let delay = RETRY_BASE_DELAY * 2u32.pow(attempt); // 지수 백오프
                std::thread::sleep(delay);
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

---

## PHASE 3 — 리팩토링 실행 워크플로우

### 3-1. 단계별 실행 순서

```bash
# Step 1: 기준선 측정 및 저장
cargo clippy 2>&1 | tee clippy_before.txt
cargo test    2>&1 | tee test_before.txt
cargo bench        2>&1 | tee bench_before.txt

# Step 2: 포맷 통일 (가장 안전 — 로직 변경 없음)
cargo fmt
git commit -m "style: cargo fmt 전체 적용"

# Step 3: Clippy 자동 수정
cargo clippy --fix --allow-dirty
cargo test  # 즉시 검증
git commit -m "refactor: clippy 자동 수정 적용"

# Step 4: 수동 리팩토링 (카탈로그 항목 순서대로)
# → 각 항목 완료 후 즉시:
cargo test
cargo clippy -- -D warnings

# Step 5: 성능 회귀 확인
cargo bench 2>&1 | tee bench_after.txt
# 비교:
cargo bench -- --baseline before  # Criterion 기준선 비교

# Step 6: 문서 생성
cargo doc --no-deps --open        # rustdoc 확인

# Step 7: 최종 커버리지 확인
cargo tarpaulin --out Html --output-dir coverage/

# Step 8: 최종 커밋
git commit -m "refactor(order): [R-R-06] OrderStatus bool 플래그 → Enum 상태 머신 전환"
```

### 3-2. 커밋 메시지 컨벤션

```
형식: refactor(<scope>): [R-R-XX] <50자 이내 요약>

예시:
refactor(user):    [R-R-01] UserList 처리에서 불필요한 clone() 제거
refactor(config):  [R-R-02] unwrap 제거, ConfigError 타입 도입
refactor(order):   [R-R-03] AccountId, AmountCents Newtype 도입
refactor(db):      [R-R-04] OrderRepository Trait 추상화 + mockall 테스트
refactor(report):  [R-R-05] 집계 루프 단일 패스 이터레이터 체인으로 전환
refactor(order):   [R-R-06] OrderStatus bool 플래그 → Enum 상태 머신
refactor(api):     [R-R-07] blocking HTTP → async reqwest + join_all 병렬화
refactor(arch):    [R-R-08] flat src/ → domain/infrastructure/shared 계층 분리
```

### 3-3. PR 체크리스트

```
코드 품질:
□ cargo test — 전체 통과
□ cargo clippy -- -D warnings — 경고 0건
□ cargo fmt --check — 포맷 위반 없음
□ 커버리지 기준선(80%) 유지 또는 향상

안전성:
□ unsafe 블록 추가 없음 (불가피한 경우 SAFETY 주석 필수)
□ unwrap/expect 추가 없음 (라이브러리 코드)
□ 데이터 레이스 가능성 없음 (Send/Sync 경계 확인)

성능:
□ cargo bench 기준선 ±5% 이내
□ 불필요한 힙 할당 미도입
□ async 컨텍스트에서 blocking 코드 없음

설계:
□ 공개 Trait/구조체 시그니처 변경 없음 (파괴적 변경 시 버전 업)
□ pub 노출 최소화 (pub(crate) / pub(super) 우선)
□ Clippy pedantic lints 통과

문서:
□ pub 아이템 rustdoc 주석 업데이트
□ CHANGELOG 항목 추가
□ 아키텍처 변경 시 README 업데이트

운영 안전:
□ 에러 메시지/코드 변경 없음 (모니터링 연계 확인)
□ 로그 포맷 변경 없음
□ 환경 변수 키 변경 없음
□ 직렬화 형식 변경 없음 (serde 필드명 등)
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
// legacy-storage = []
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

팁: cargo check 가 cargo build보다 빠름 — 의존성 탐색 시 활용
```

---

## PHASE 5 — Rust 리팩토링 금지 패턴

```
❌ unsafe 블록을 "성능"을 이유로 근거 없이 도입
   → 대안: 알고리즘 개선, SIMD 크레이트(packed_simd), Rayon 병렬화

❌ Arc<Mutex<T>>로 모든 공유 상태 처리
   → 대안: 메시지 패싱(tokio::mpsc), DashMap, RwLock

❌ Box<dyn Trait> 남용 (동적 디스패치)
   → 대안: 제네릭 <T: Trait> (정적 디스패치, 컴파일 타임 단형화)

❌ 라이프타임 오류를 .to_string() / .clone()으로 회피
   → 원인 분석 후 소유권 설계 재검토

❌ panic!() / unwrap() 을 에러 처리 대신 사용
   → 라이브러리: 절대 금지. 바이너리: main()에서만 허용

❌ Rc<T>를 멀티스레드 환경에서 사용
   → Arc<T> 사용 (컴파일러가 Send 위반으로 잡아줌)

❌ 에러 타입 없이 Box<dyn Error>로 통일
   → thiserror로 구체적 에러 타입 정의

❌ async fn을 sync 컨텍스트에서 .block_on()으로 호출
   → 전체 스택을 async로 전환하거나 tokio::spawn 활용

❌ 리팩토링 이름으로 크레이트 교체 (별도 PR로 분리)
❌ 기능 추가와 리팩토링 동시 진행
```

---

## 참고 자료

| 항목 | 도구/자료 |
|------|-----------|
| 정적 분석 | Clippy (`cargo clippy`) |
| 코드 포맷 | rustfmt (`cargo fmt`) |
| 테스트 커버리지 | cargo-tarpaulin |
| 벤치마킹 | Criterion (`cargo bench`) |
| 모킹 | mockall |
| 에러 처리 | thiserror, anyhow |
| 비동기 | tokio, async-trait |
| 병렬 처리 | rayon, tokio |
| IDE | rust-analyzer (VS Code, CLion, Helix) |
| 핵심 서적 | *Rust for Rustaceans* — Jon Gjengset |
| 핵심 서적 | *Programming Rust* — Blandy & Orendorff (2nd Ed.) |
| 패턴 참고 | rust-unofficial.github.io/patterns |
| API 가이드 | rust-lang.github.io/api-guidelines |
