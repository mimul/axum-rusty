# 테스트 규칙 (Test Rules)

Claude가 이 프로젝트에서 테스트를 작성하거나 리뷰할 때
반드시 준수해야 하는 테스트 규칙이다.

---

## 1. 테스트 구조

### 파일 배치 원칙

```
src/
├── domain/order/service.rs      ← 단위 테스트는 같은 파일 내 #[cfg(test)] 모듈
└── ...

tests/
├── order_integration_test.rs    ← 통합 테스트 (외부 DB, HTTP 포함)
└── common/
    └── mod.rs                   ← 공통 테스트 헬퍼
```

### 모듈 구조

```rust
// src/domain/order/service.rs
pub struct OrderService { ... }

impl OrderService {
    pub fn calculate_total(&self, order: &Order) -> Money { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_total_applies_coupon_discount() { ... }

    #[test]
    fn calculate_total_returns_zero_for_empty_order() { ... }
}
```

---

## 2. 테스트 네이밍

**형식**: `[테스트_대상]_[조건]_[기대_결과]`

```rust
// ✅ 의도가 명확한 이름
#[test]
fn calculate_total_with_10pct_coupon_applies_discount() { ... }

#[test]
fn find_user_when_not_exists_returns_not_found_error() { ... }

#[test]
fn create_order_with_empty_items_returns_validation_error() { ... }

// ❌ 의미 없는 이름
#[test]
fn test1() { ... }

#[test]
fn test_order() { ... }
```

---

## 3. 테스트 커버리지 기준

| 구분 | 최소 커버리지 | 비고 |
|------|--------------|------|
| 전체 | **80%** | CI에서 강제 (`cargo tarpaulin`) |
| `controller/` | **80%** | 라우터와 서버 구동 부분을 구현. 요청/응답 전처리, 에러 모델 정의, JSON의 직렬화 및 역직렬화를 처리 |
| `usecase/` | 80% | 어플리케이션을 처리하기 위해 필요한 비즈니스 로직을 구현. 여러 리포지토리를 통해 애플리케이션에 필요한 데이터 구조를 반환 |
| `domain/` | 80% | 도메인 모델의 생성이나, repository 구현 |
| `infra/` | 85% | 외부 서비스와의 연계 레이어, DB 접속이나 쿼리 로직을 구현 |
| `common/` | 80% | 설정 피일 로드, 로그 설정, 인증 쿠키, 인증 헤더 처리 함수들을 구현 |

```bash
# 커버리지 측정
cargo tarpaulin --out Html --output-dir coverage/
open coverage/tarpaulin-report.html
```

---

## 4. 테스트 종류별 작성 기준

### 단위 테스트 (Unit Test)

- 외부 의존성(DB, HTTP) 없이 순수 로직만 검증
- `mockall`로 Repository/외부 서비스 Mock 생성
- 정상 케이스 + **에러 케이스 + 에지 케이스** 모두 작성

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    mockall::mock! {
        OrderRepositoryMock {}
        #[async_trait::async_trait]
        impl OrderRepository for OrderRepositoryMock {
            async fn find_by_id(&self, id: OrderId)
                -> Result<Option<Order>, RepositoryError>;
        }
    }

    #[tokio::test]
    async fn get_order_returns_not_found_when_missing() {
        let mut mock = MockOrderRepositoryMock::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(None));

        let service = OrderService::new(mock);
        let result = service.get_order(OrderId::new(999)).await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }
}
```

### 통합 테스트 (Integration Test)

- 실제 DB 연결 (테스트 전용 DB 사용)
- 각 테스트 시작 전 트랜잭션 시작, 종료 후 롤백 (격리 보장)
- `tests/common/` 에 DB 설정·픽스처 헬퍼 모음

```rust
// tests/common/mod.rs
pub async fn setup_test_db() -> sqlx::PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL 환경변수 필요");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    pool
}

// tests/order_integration_test.rs
#[tokio::test]
async fn create_order_persists_to_database() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    // 테스트 실행
    let repo = PgOrderRepository::new(&mut tx);
    let order = Order::new(...);
    repo.save(&order).await.unwrap();

    let found = repo.find_by_id(order.id).await.unwrap();
    assert_eq!(found, Some(order));

    tx.rollback().await.unwrap();  // 항상 롤백 → DB 상태 오염 없음
}
```

### 프로퍼티 기반 테스트 (Property-Based Test)

복잡한 도메인 로직에는 `proptest` 또는 `quickcheck` 활용:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn calculate_total_never_negative(
        prices in prop::collection::vec(0.0f64..10000.0, 0..100),
        discount in 0.0f64..1.0
    ) {
        let items: Vec<OrderItem> = prices.into_iter()
            .map(|p| OrderItem::new(p, 1))
            .collect();
        let total = calculate_total(&items, discount);
        prop_assert!(total >= 0.0);
    }
}
```

---

## 5. 에러 케이스 테스트 필수 목록

모든 `Result` 반환 함수에 대해 아래 케이스를 반드시 작성한다:

```rust
// ✅ 에러 케이스 테스트 예시
#[test]
fn parse_user_id_with_invalid_uuid_returns_error() {
    let result = UserId::parse("not-a-uuid");
    assert!(matches!(result, Err(AppError::InvalidUserId)));
}

#[test]
fn create_order_with_empty_items_returns_validation_error() {
    let result = Order::new(CustomerId::new(1), vec![]);
    assert!(matches!(result, Err(AppError::EmptyOrderItems)));
}

#[test]
fn divide_by_zero_returns_overflow_error() {
    let result = safe_divide(10, 0);
    assert!(matches!(result, Err(AppError::DivisionByZero)));
}
```

---

## 6. 테스트 금지 패턴

```
🚫 #[ignore] 무단 추가 (리뷰어 승인 없이)
🚫 테스트 삭제 (코드 변경 시 테스트도 함께 수정)
🚫 불필요한 sleep() 사용 (비동기 테스트는 tokio::test 사용)
🚫 테스트 간 공유 상태 (각 테스트는 독립적으로 실행 가능해야 함)
🚫 assert!(result.is_ok()) — 실패 시 원인을 알 수 없음
    → unwrap() 또는 assert!(matches!(result, Ok(_)), "{result:?}") 사용
🚫 프로덕션 DB에 테스트 데이터 삽입
```

---

## 7. Result 반환 테스트 권장

`#[should_panic]` 대신 `Result` 반환 테스트를 선호한다:

```rust
// ❌ should_panic — 어떤 패닉인지 구분 어려움
#[test]
#[should_panic]
fn parse_invalid_id_panics() {
    UserId::parse("bad").unwrap();
}

// ✅ Result 반환 — 에러 타입까지 명확히 검증
#[test]
fn parse_invalid_id_returns_error() -> Result<(), AppError> {
    let result = UserId::parse("bad");
    assert!(matches!(result, Err(AppError::InvalidUserId)));
    Ok(())
}
```

---

## 8. 벤치마크 작성 기준

성능 민감 경로(쿼리 집계, 직렬화 등)에는 Criterion 벤치마크를 작성한다:

```rust
// benches/order_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_calculate_total(c: &mut Criterion) {
    let items = generate_test_items(1000);
    c.bench_function("calculate_total_1000_items", |b| {
        b.iter(|| calculate_total(black_box(&items), 0.0))
    });
}

criterion_group!(benches, bench_calculate_total);
criterion_main!(benches);
```

```bash
# 로컬 벤치마크 실행
cargo bench

# PR에서 성능 회귀 확인 (perf-check 라벨 필요)
# → pr-ci.yml Job 6 자동 실행
```

---

## CI 테스트 자동화

PR CI에서 아래 순서로 자동 실행된다 (`pr-ci.yml` 참조):

```
Job 4: cargo test --all           단위 + 통합 테스트
Job 5: cargo tarpaulin            커버리지 측정 (80% 목표)
Job 6: cargo bench (선택)        성능 회귀 확인
```

```bash
# 로컬 사전 확인 (PR 올리기 전)
cargo test --all
cargo tarpaulin --out Html
```
