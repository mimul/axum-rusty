# Rust 도메인 중심 코딩 원칙

> 기원: [언어에 의존하지 않는 도메인 중심 코딩 원칙과 실천법](https://www.mimul.com/blog/ai-coding-style/)을 Rust에 맞게 적용한 문서.
> Rust의 소유권, 타입 시스템, 트레이트 기반 설계를 활용하여 원칙을 더욱 강하게 실현한다.

---

## 철학적 뿌리

세 사람의 사상이 이 원칙의 근간을 이룬다.

**Kent Beck**: "변경하기 쉬운 상태를 항상 유지하는 것"이 소프트웨어 설계의 목표다.  
**Martin Fowler**: 리팩토링은 개발의 "리듬"으로서 자연스럽게 녹아있어야 한다.  
**Eric Evans**: "코드가 비즈니스 도메인의 언어를 반영해야 한다." — Rust의 타입 시스템은 이 원칙을 컴파일 타임에 강제한다.

**핵심 통찰**: "코드는 읽히고, 이해되고, 변경될 것이다. Rust에서는 그 변경이 안전하게 이루어지도록 컴파일러가 함께 설계에 참여한다."

---

## 핵심 원칙 (6가지)

### 1. 변화 용이성을 최우선으로

"3개월 후 누군가가 이 코드를 쉽게 수정할 수 있을까?"가 모든 설계 결정의 기준이다.

**Rule of Three**: 동일한 패턴이 세 번 반복되기 전까지 추상화(트레이트, 제네릭)를 도입하지 않는다.

```rust
// ❌ 조기 추상화 — 사용처가 하나뿐인데 트레이트를 꺼냄
trait Processor {
    fn process(&self, input: &str) -> String;
}
struct EmailProcessor;
impl Processor for EmailProcessor {
    fn process(&self, input: &str) -> String { todo!() }
}

// ✅ 중복이 드러날 때까지 단순하게
fn process_email(input: &str) -> String { todo!() }
```

성능은 실제 병목이 `cargo bench` 또는 `perf`로 측정된 이후에 고민한다.

---

### 2. 코드는 의도를 명확하게 말해야 한다

좋은 코드는 "무엇을 하는지"가 아니라 "왜 존재하는지"를 전달한다.

```rust
// ❌ 숫자와 상태 코드 직접 비교 — 의도 불명
if user.role == 1 && user.status == 0 {
    send_notification(&user);
}

// ✅ 도메인 언어로 의도를 드러냄
if user.is_eligible_for_notification() {
    send_notification(&user);
}
```

**매직 넘버를 상수 또는 enum으로 제거**:

```rust
// ❌ 숫자 리터럴 직접 사용
fn mile_to_metre(miles: f64) -> f64 {
    miles * 1609.344
}

// ✅ 의미를 가진 상수
const METRES_PER_MILE: f64 = 1609.344;

fn mile_to_metre(miles: f64) -> f64 {
    miles * METRES_PER_MILE
}
```

**반환 타입에 의도를 명시**:

```rust
// ✅ Result로 실패 이유까지 타입에서 전달
fn find_user(id: UserId) -> Result<User, UserError> {
    self.store.get(&id).cloned().ok_or(UserError::NotFound(id))
}
```

**주석 원칙**: 주석은 "왜(why)"를 적는다. 코드로 표현할 수 없는 숨겨진 제약, 비즈니스 의사결정 맥락만 남긴다.

```rust
// ❌ 코드가 이미 말하는 것을 반복
// discount_rate가 0.05이면 5% 할인 적용
let discounted = total * (1.0 - 0.05);

// ✅ 코드만으로는 알 수 없는 "왜"를 설명
// 2018년 VIP 캠페인 약정 (기획서 #1234): 1만원 이상 주문에 5% 할인
let discounted = total * (1.0 - VIP_DISCOUNT_RATE);
```

---

### 3. 도메인 언어를 코드에 반영한다

도메인 전문가가 "주문 취소"라고 말하면 코드에서도 `cancel_order`다. 번역 없이 일대일로 대응해야 한다.

```rust
// ❌ 기술 용어로 도메인을 덮음
fn update_order_flag(order_id: i64, flag: i32) { }

// ✅ 도메인 언어 그대로
fn cancel_order(order_id: OrderId) -> Result<(), OrderError> { todo!() }
fn ship_order(order_id: OrderId, address: ShippingAddress) -> Result<(), OrderError> { todo!() }
```

Rust의 `enum`은 도메인 상태를 표현하는 가장 강력한 도구다:

```rust
// ❌ 상태를 정수로 표현 — 도메인 의미 소실
struct Order {
    status: i32, // 0=pending, 1=paid, 2=shipped, 3=cancelled
}

// ✅ enum으로 도메인 상태와 데이터를 함께 명확하게 표현
enum OrderStatus {
    Pending,
    Paid { paid_at: DateTime<Utc> },
    Shipped { tracking_number: TrackingNumber },
    Cancelled { reason: CancellationReason },
}
```

---

### 4. 작고 되돌릴 수 있는 단위로 변경한다

커밋 하나에 하나의 명확한 의도만 담는다. 큰 리팩토링도 항상 `cargo test`가 통과하는 상태를 유지하며 진행한다.

```
# 좋은 커밋 예시
git commit -m "refactor(order): [R-R-02] OrderId에 Newtype 패턴 적용"
git commit -m "refactor(order): [R-R-06] unwrap() 제거 및 ? 연산자로 교체"
```

중간 상태가 컴파일·테스트를 통과해야만 언제든 되돌릴 수 있다.

---

### 5. 기존 컨텍스트의 관례를 따른다

기존 코드베이스에 합류할 때는 파일과 프로젝트의 관례가 개인 취향보다 우선이다. 스타일이 혼재하는 것은 어느 한쪽이 나쁜 스타일인 것보다 더 큰 문제다.

`rustfmt`와 `clippy`를 CI 게이트로 강제하면 스타일 논의를 자동으로 없앨 수 있다:

```bash
cargo fmt --check          # 포맷 위반 시 CI 실패
cargo clippy -- -D warnings  # 경고를 에러로 처리
```

---

### 6. 불확실할 때는 질문한다

요구사항이 모호하면 추측하지 않는다. 추측으로 만들어진 타입 설계는 나중에 큰 수정 비용을 유발한다.

---

## 설계 원칙 (6가지)

### 1. 작고 조합 가능한 구조

함수는 하나의 책임만 가진다. 중첩 깊이는 최대 2단계.

**조기 반환(Early Return)으로 중첩 제거**:

```rust
// ❌ 깊은 중첩 — 핵심 로직이 깊은 곳에 숨어있다
fn process_order(order: &Order) -> Result<Receipt, OrderError> {
    if order.is_paid() {
        if let Some(items) = order.items() {
            if !items.is_empty() {
                return Ok(generate_receipt(items));
            }
        }
    }
    Err(OrderError::InvalidState)
}

// ✅ 조기 반환으로 조건을 위로 올림 — 핵심 로직이 마지막에 명확하게
fn process_order(order: &Order) -> Result<Receipt, OrderError> {
    if !order.is_paid() {
        return Err(OrderError::NotPaid);
    }
    let items = order.items().ok_or(OrderError::MissingItems)?;
    if items.is_empty() {
        return Err(OrderError::EmptyOrder);
    }
    Ok(generate_receipt(items))
}
```

**Tell, Don't Ask** — 상태를 물어 외부에서 결정하지 말고 객체에 행동을 위임:

```rust
// ❌ Ask: 외부에서 상태를 물어 결정
if order.status == OrderStatus::Pending && !order.items.is_empty() {
    order.status = OrderStatus::Confirmed;
}

// ✅ Tell: 객체에 행동을 위임
order.confirm()?;
```

**복잡한 조건식은 의미 있는 변수로 분해**:

```rust
// ❌ 한 줄에 모든 조건 압축
if user.role == UserRole::Admin && (user.last_login_at > cutoff || user.is_superuser) {
    grant_access();
}

// ✅ 의도가 드러나는 임시 변수로 분해
let is_admin = user.role == UserRole::Admin;
let has_recent_activity = user.last_login_at > cutoff;
let can_access = is_admin && (has_recent_activity || user.is_superuser);
if can_access {
    grant_access();
}
```

---

### 2. 도메인 모델에 행동을 담는다

Rust에서 빈약한 도메인 모델은 `impl` 블록 없이 필드만 있는 struct다.

```rust
// ❌ 빈약한 모델: 데이터만 있고 행동이 없음 — 로직이 서비스에 흩어짐
struct Order {
    total: Money,
    status: OrderStatus,
}

impl OrderService {
    fn apply_discount(&self, order: &mut Order, rate: f64) {
        let discount = order.total.amount() as f64 * rate;
        order.total = Money::new((order.total.amount() as f64 - discount) as i64);
    }
}

// ✅ 풍부한 도메인 모델: 도메인 불변식을 struct 자신이 보호
impl Order {
    pub fn apply_discount(&mut self, rate: f64) -> Result<(), DomainError> {
        if !(0.0..=MAX_DISCOUNT_RATE).contains(&rate) {
            return Err(DomainError::InvalidDiscountRate(rate));
        }
        let discount = (self.total.amount() as f64 * rate) as i64;
        self.total = self.total.subtract(Money::new(discount))?;
        Ok(())
    }

    pub fn cancel(&mut self, reason: CancellationReason) -> Result<(), DomainError> {
        if !matches!(self.status, OrderStatus::Pending | OrderStatus::Paid { .. }) {
            return Err(DomainError::InvalidStatusTransition);
        }
        self.status = OrderStatus::Cancelled { reason };
        Ok(())
    }
}
```

**Newtype 패턴으로 도메인 식별자 보호**:

```rust
// ❌ 원시 타입: user_id와 order_id를 컴파일러가 구분할 수 없음
fn get_order(user_id: i64, order_id: i64) -> Result<Order, Error> { todo!() }

// ✅ Newtype: 컴파일 타임에 인수 혼동을 원천 차단
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct OrderId(i64);

fn get_order(user_id: UserId, order_id: OrderId) -> Result<Order, Error> { todo!() }
```

---

### 3. 추상화는 중복이 드러난 이후에

YAGNI — 지금 필요하지 않은 트레이트, 제네릭, 레이어를 만들지 않는다.

```rust
// ❌ 사용처가 하나뿐인데 조기에 만든 트레이트
pub trait UserRepository {
    fn find_by_id(&self, id: UserId) -> Option<User>;
    fn save(&self, user: User) -> Result<(), Error>;
}

// ✅ 두 번째 구현체(e.g., InMemoryUserRepository)가 필요해졌을 때 트레이트 도입
pub struct PostgresUserRepository { pool: PgPool }

impl PostgresUserRepository {
    pub async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Error> { todo!() }
    pub async fn save(&self, user: &User) -> Result<(), Error> { todo!() }
}
```

트레이트는 기본적으로 봉인 상태로 설계한다. 외부 크레이트에서 구현을 허용하는 것은 API 공개와 같으므로, 필요가 명확해진 후에만 연다.

---

### 4. 접근 권한은 최소한으로 정의한다

새 아이템은 항상 가장 좁은 범위부터 시작한다.

```rust
// ❌ 불필요하게 넓은 공개 범위
pub struct OrderRepository {
    pub pool: PgPool,    // 외부에서 pool을 직접 건드릴 이유 없음
}

// ✅ 필요한 최소 범위
pub struct OrderRepository {
    pool: PgPool,        // 비공개 — 기본값
}
```

| 범위 | 선언 | 사용 시점 |
|------|------|-----------|
| 비공개 | (기본) | 기본값 — 항상 여기서 시작 |
| 크레이트 공유 | `pub(crate)` | 같은 크레이트 내 모듈 간 |
| 상위 모듈 공유 | `pub(super)` | 부모 모듈까지만 |
| 공개 API | `pub` | 외부 크레이트에 실제로 필요할 때만 |

---

### 5. 트레이트 경계로 추상 타입을 참조한다

```rust
// ❌ 구체 타입을 파라미터로 받아 구현 교체 시 시그니처도 바꿔야 함
fn send_notification(sender: &SmtpSender, message: &str) { todo!() }

// ✅ 트레이트 경계: 구현을 교체해도 호출부 변경 없음
fn send_notification(sender: &impl NotificationSender, message: &str) { todo!() }
```

반환 타입에서도 구체 타입 대신 `impl Trait`를 우선한다:

```rust
// ✅ 호출자가 구현 세부 사항에 의존하지 않음
fn active_users(users: &[User]) -> impl Iterator<Item = &User> {
    users.iter().filter(|u| u.is_active())
}
```

---

### 6. 변수는 사용 직전에 선언하고 스코프를 최소화한다

```rust
// ❌ 선언과 사용이 멀다
let result;
let items = fetch_items().await?;
// ... 다른 처리들 ...
result = calculate_total(&items);

// ✅ 필요한 순간에 선언 + 체이닝으로 스코프 최소화
let total = fetch_items().await?
    .iter()
    .filter(|i| i.is_active())
    .map(|i| i.price)
    .sum::<Money>();
```

블록으로 스코프를 명시적으로 제한한다:

```rust
// 임시 변수가 블록 밖으로 노출되지 않도록
let config = {
    let raw = std::env::var("APP_CONFIG")?;
    parse_config(&raw)?
};
// raw는 이 시점에서 이미 스코프 밖
```

---

## 리팩토링은 개발의 리듬이다

리팩토링은 기능 추가와 교대로 이루어진다. 새 기능을 추가하기 어려운 구조라면, 기능을 넣기 전에 먼저 구조를 개선한다.

### 리팩토링이 필요하다는 신호

- 동일한 코드가 3번 이상 반복될 때 (Rule of Three)
- 변수나 함수 이름이 도메인 의도를 충분히 설명하지 못할 때
- `impl` 블록 하나가 너무 많은 책임을 가질 때
- `clippy`가 같은 패턴을 반복적으로 경고할 때
- `match` 분기가 계속 늘어나며 새 variant 추가가 무서워질 때
- 서비스 레이어에 도메인 조건문이 점점 쌓일 때

### 죽은 코드는 과감하게 제거한다

주석 처리된 코드, `#[allow(dead_code)]`로 숨겨진 미사용 함수, 오래된 설정은 즉시 삭제한다. git이 모든 이력을 보관한다.

```bash
cargo check 2>&1 | grep "unused"
cargo clippy -- -D dead_code
```

### 실천하는 태도

리팩토링은 항상 작은 단위로, `cargo test`가 통과하는 상태를 유지하며 진행한다.

```
# ❌ 의도를 알 수 없는 커밋
git commit -m "코드 정리"

# ✅ 의도가 명확한 커밋
git commit -m "refactor(payment): [R-R-01] clone() 제거 — Arc<T>로 공유 소유권 전환"
git commit -m "refactor(user): [R-R-02] UserId Newtype 적용 — user_id: i64 혼동 방지"
```

---

## 네이밍은 도메인의 번역이다

### 피해야 할 이름들

`data`, `util`, `helper`, `manager`, `processor` — 역할과 책임이 불명확하다는 신호다.

```rust
// ❌ 이름이 아무것도 말하지 않는다
struct DataHelper;
fn process_data(data: &Data) -> Result<Data, Error> { todo!() }

// ✅ 도메인 역할이 이름에 드러난다
struct InvoiceCalculator;
fn apply_discount(order: &mut Order, rate: f64) -> Result<(), DomainError> { todo!() }
```

### Rust 네이밍 관례 (RFC 430)

| 아이템 | 관례 | 예시 |
|--------|------|------|
| 타입, 트레이트, enum variant | `UpperCamelCase` | `OrderStatus`, `Repository` |
| 함수, 메서드, 변수, 필드 | `snake_case` | `find_user`, `order_id` |
| 상수, 정적 변수 | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT` |
| 라이프타임 | `'lowercase` | `'a`, `'static` |
| 제네릭 타입 파라미터 | 단일 대문자 또는 `UpperCamelCase` | `T`, `E`, `Item` |
| 모듈 | `snake_case` | `order_service`, `infra` |

### 축약어 최소화

`usr`, `cfg`, `cnt`, `idx` 대신 `user`, `config`, `count`, `index`. 타이핑 비용은 작고, 읽기 비용은 크다.

### 구현이 아닌 의도를 드러내는 이름

```rust
// ❌ 구현 방식이 이름에 드러남
fn get_user_from_db(id: UserId) -> Option<User> { todo!() }
fn loop_and_sum_prices(items: &[Item]) -> Money { todo!() }

// ✅ 의도가 이름에 드러남
fn find_user(id: UserId) -> Option<User> { todo!() }
fn calculate_order_total(items: &[Item]) -> Money { todo!() }
```

### 동사 일관성

같은 개념에 `fetch`, `get`, `retrieve`, `load`를 혼용하지 않는다.

| 의미 | 권장 동사 |
|------|-----------|
| 저장소에서 단일 조회 | `find_` (없으면 `None` / `Err`) |
| 저장소에서 목록 조회 | `list_` |
| 계산 결과 반환 | `calculate_` |
| 상태 전환 요청 | 도메인 동사 (`confirm`, `cancel`, `ship`) |
| 도메인 불변식 검증 | `is_`, `has_`, `can_` |

---

## 경계 조건은 도메인의 일부다

"재고 없음", "미인증 사용자", "결제 실패" — 이것들은 예외가 아니라 도메인이 정상적으로 다뤄야 할 유효한 상태다.

### `Option<T>`와 `Result<T, E>`로 경계 조건을 명시적으로 표현

```rust
// ❌ null 반환으로 호출자에게 null 체크 부담 전가 (Rust에서는 raw pointer 또는 unwrap 강요)
fn find_order(id: OrderId) -> *const Order { todo!() }

// ✅ Option/Result로 "없음"과 "실패"를 타입에서 강제
fn find_order(id: OrderId) -> Option<Order> { todo!() }
fn create_order(cmd: CreateOrderCommand) -> Result<Order, DomainError> { todo!() }
```

### Invalid State를 타입 수준에서 제거

```rust
// ❌ 상호 배타적 Option 필드들 — 유효하지 않은 상태 조합이 가능
struct Order {
    status: OrderStatus,
    tracking_number: Option<String>,   // shipped일 때만 의미 있음
    paid_at: Option<DateTime<Utc>>,    // paid일 때만 의미 있음
}

// ✅ 상태별로 유효한 데이터만 포함하도록 enum에 데이터 부착
enum OrderStatus {
    Pending,
    Paid { paid_at: DateTime<Utc> },
    Shipped {
        paid_at: DateTime<Utc>,
        tracking_number: TrackingNumber,
    },
    Cancelled { reason: CancellationReason },
}
```

### 컬렉션은 `None` 대신 빈 컬렉션을 반환한다

```rust
// ❌ None 반환 — 호출자가 항상 처리를 강요받음
fn find_active_users(&self) -> Option<Vec<User>> {
    if self.users.is_empty() { return None; }
    Some(self.users.iter().filter(|u| u.is_active()).cloned().collect())
}

// ✅ 빈 Vec 반환 — 호출자가 안전하게 순회 (for 루프, map, filter 등)
fn find_active_users(&self) -> Vec<User> {
    self.users.iter().filter(|u| u.is_active()).cloned().collect()
}
```

### `None`과 빈 값은 분명히 구분한다

```rust
struct Product {
    stock_count: Option<u32>, // None=아직 미조회, Some(0)=재고 0개
}

// ❌ None과 0을 같은 의미로 취급 — 두 상태의 의미 소실
if product.stock_count.unwrap_or(0) == 0 {
    fetch_inventory().await?;
}

// ✅ 두 상태를 명시적으로 구분
match product.stock_count {
    None => fetch_inventory().await?,
    Some(0) => notify_out_of_stock().await?,
    Some(n) => reserve(n).await?,
}
```

### `unwrap()`과 `expect()`는 라이브러리·핸들러에서 절대 금지

```rust
// ❌ 핸들러에서 panic 유발 — 서버 crash 또는 DoS로 이어질 수 있음
async fn handler(state: State<AppState>) -> Json<User> {
    let user = state.repo.find_user(UserId(1)).await.unwrap(); // panic!
    Json(user)
}

// ✅ ? 연산자로 에러를 호출 스택 위로 전파
async fn handler(
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = state.repo.find_user(UserId(1)).await?;
    Ok(Json(user))
}
```

`expect()`는 컴파일 타임에 유효성이 보장되는 리터럴에만 허용하며, 이유를 주석으로 명시한다:

```rust
// 허용: 컴파일 타임 리터럴은 항상 유효
static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .expect("날짜 정규식 리터럴 — 컴파일 타임에 유효성 보장")
});
```

### `match`는 완전성을 유지한다

```rust
// ❌ wildcard로 새 variant를 조용히 무시 — 추가 시 컴파일은 되지만 버그
match status {
    OrderStatus::Pending => enable_edit(),
    OrderStatus::Paid { .. } => show_receipt(),
    _ => {} // Shipped, Cancelled 무시
}

// ✅ 모든 variant를 명시 — 새 variant 추가 시 컴파일 오류로 알려줌
match status {
    OrderStatus::Pending => enable_edit(),
    OrderStatus::Paid { .. } => show_receipt(),
    OrderStatus::Shipped { tracking_number, .. } => show_tracking(tracking_number),
    OrderStatus::Cancelled { reason } => show_cancellation(reason),
}
```

### 에러를 제어 흐름으로 사용하지 않는다

```rust
// ❌ 정상 분기를 에러로 표현
fn find_or_guest(user_id: UserId) -> User {
    match find_user(user_id) {
        Ok(user) => user,
        Err(_) => create_guest_user(), // 에러가 아닌 정상 분기
    }
}

// ✅ 조건문 또는 Option 조합자로 정상 흐름 표현
async fn find_or_guest(&self, user_id: UserId) -> Result<User, AppError> {
    if let Some(user) = self.repo.find_by_id(user_id).await? {
        return Ok(user);
    }
    Ok(self.create_guest_user())
}
```

---

## 성능은 측정이 먼저다

> "Premature optimization is the root of all evil." — Donald Knuth

### 올바른 순서

1. 올바르게 동작하게 만든다
2. `cargo bench` / `perf` / `flamegraph`로 실제 병목을 측정한다
3. 측정된 병목만 선택적으로 최적화한다

### 실무에서 지키는 원칙들

**`clone()`은 소유권 이전이 실제로 필요한 경우에만**:

```rust
// ❌ 불필요한 clone — 참조로 충분
fn greet(name: String) -> String {
    format!("Hello, {}", name.clone())
}

// ✅ &str / &[T] 우선
fn greet(name: &str) -> String {
    format!("Hello, {}", name)
}
```

**N+1 문제는 가장 흔하고 위험한 패턴**:

```rust
// ❌ 반복문 내 DB 조회 — N번 쿼리
for order in &orders {
    let items = repo.find_items_by_order(order.id).await?;
    process(order, &items);
}

// ✅ 배치 조회
let ids: Vec<OrderId> = orders.iter().map(|o| o.id).collect();
let items_map = repo.find_items_by_orders(&ids).await?;
for order in &orders {
    let items = items_map.get(&order.id).map(Vec::as_slice).unwrap_or(&[]);
    process(order, items);
}
```

**async 컨텍스트에서는 `tokio::sync::Mutex` / `RwLock` 사용**:

```rust
// ❌ std::sync::Mutex — async 컨텍스트에서 deadlock 위험
struct AppState {
    cache: std::sync::Mutex<HashMap<String, String>>,
}

// ✅ tokio의 비동기 동기화 프리미티브 사용
struct AppState {
    cache: tokio::sync::RwLock<HashMap<String, String>>,
}
```

**리소스는 RAII로 자동 해제**:

Rust의 `Drop` 트레이트가 스코프 종료 시 자동으로 리소스를 정리한다. 수동 정리 코드는 구현 버그의 원인이다.

**알고리즘 복잡도는 의식적으로 선택**:

O(n²) 이상의 복잡도가 들어갈 때는 반드시 명확한 이유가 있어야 한다. 특히 대규모 데이터를 다룰 때 `Vec<T>`를 반복 검색하는 대신 `HashMap<K, V>`을 활용하는 것을 검토한다.

---

## Rust 고유 관례

### 에러 타입 설계 — 레이어별 분리

```rust
// ❌ 모든 에러를 하나의 enum으로 — 레이어 경계가 사라짐
enum AppError {
    Database(sqlx::Error),
    NotFound,
    InvalidDiscountRate(f64),
    Unauthorized,
}

// ✅ 레이어별 에러 타입 분리 (thiserror 활용)
// domain/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("유효하지 않은 할인율: {0}")]
    InvalidDiscountRate(f64),
    #[error("잘못된 상태 전환")]
    InvalidStatusTransition,
}

// controller/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("데이터베이스 오류")]
    Database(#[from] sqlx::Error),
    #[error("도메인 오류: {0}")]
    Domain(#[from] DomainError),
    #[error("찾을 수 없음")]
    NotFound,
}
```

### `?` 연산자를 최대한 활용한다

```rust
// ❌ 장황한 match — 노이즈만 증가
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(ConfigError::Io(e)),
    };
    match serde_json::from_str(&content) {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(ConfigError::Parse(e)),
    }
}

// ✅ ? 연산자 체이닝
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}
```

### `unsafe` 블록은 최소화하고 반드시 이유를 명시

```rust
// ❌ 이유 없는 unsafe
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}

// ✅ SAFETY 주석으로 불변식 증명
// SAFETY: src와 dst는 각각 len 바이트의 유효한 메모리를 가리키며,
//         호출자가 두 범위가 겹치지 않음을 보장했다.
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

### 공개 API에는 `///` 문서 주석 필수

```rust
/// 주문에 할인율을 적용한다.
///
/// # Errors
/// - `DomainError::InvalidDiscountRate`: `rate`가 `0.0..=MAX_DISCOUNT_RATE` 범위를 벗어난 경우
pub fn apply_discount(&mut self, rate: f64) -> Result<(), DomainError> { todo!() }
```

### 파라미터에서 소유가 아닌 참조를 우선한다

```rust
// ❌ 소유권을 가져가면 호출자가 clone을 강요받음
fn validate(email: String) -> bool { todo!() }

// ✅ &str, &[T]로 참조를 받아 호출자의 유연성 보장
fn validate(email: &str) -> bool { todo!() }
fn process(items: &[Item]) -> Money { todo!() }
```

---

## 안티 패턴 요약

| 안티 패턴 | Rust에서의 증상 | 해결 방향 |
|---------|----------------|---------|
| 성급한 추상화 | 사용처 없는 트레이트·제네릭 범람 | 3회 반복 후 추상화 |
| 빈약한 도메인 모델 | `impl` 없는 struct, 서비스에 로직 집중 | 도메인 객체에 행동 부여 |
| 깊은 중첩 구조 | 4단계 이상 if/match 중첩 | 조기 반환, `?` 연산자 |
| 암묵적 경계 처리 | `unwrap()` / `expect()` 남용 | `Option`/`Result` + `?` 전파 |
| 불필요한 clone() | `clone()` 과다 사용 | `&str`, `&[T]`, `Arc<T>` 활용 |
| Invalid State 허용 | struct에 상호 배타적 Option 필드 | enum에 데이터 부착 |
| 원시 타입 도메인 ID | `i64`로 UserId/OrderId 혼용 | Newtype 패턴 |
| N+1 쿼리 | 반복문 내 DB 조회 | 배치 조회, JOIN 활용 |
| std::Mutex in async | tokio 없는 blocking Mutex | `tokio::sync::Mutex` / `RwLock` |
| 에러를 제어 흐름으로 | `Err`로 정상 분기 표현 | 조건문, `unwrap_or_else` |
| wildcard match 남용 | `_ => {}` 로 새 variant 무시 | 모든 variant 명시적 처리 |
| 죽은 코드 방치 | `#[allow(dead_code)]` 남용 | 즉시 삭제, git이 이력 보관 |
| 매직 넘버 | 의미 불명의 리터럴 직접 등장 | 이름 있는 상수로 대체 |
| 스타일 혼재 | 한 파일에 두 가지 관례 공존 | `rustfmt` CI 강제 |
| String 파라미터 | 소유권 강요로 불필요한 clone | `&str` / `&[T]` 우선 |

---

## 참고 문서

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust RFC 430 — Naming conventions](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md)
- [언어에 의존하지 않는 도메인 중심 코딩 원칙과 실천법](https://www.mimul.com/blog/ai-coding-style/)
- `.claude/rules/rust-security-style.md` — 보안 원칙
- `.claude/rules/rust-test-style.md` — 테스트 원칙
