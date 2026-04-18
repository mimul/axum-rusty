# 테스트 규칙 (Test Rules)

Claude가 이 프로젝트에서 테스트를 작성하거나 리뷰할 때 반드시 준수해야 하는 테스트 규칙이다.

## 테스트 철학

- 구현이 아닌 동작을 테스트하세요. 순수 리팩토링은 테스트를 깨뜨리면 안됩니다.
 예) 공개 API를 통해 유스케이스 동작을 테스트합니다(usecase/handler).
- 시스템 경계에서만 Mocking한다. 시스템 내부의 모든 것은 실제입니다.
- 엄격한 TDD보다 Classicst 테스트를 선호합니다. TDD를 맹목적으로 적용하지 말고 실용적으로 사용하세요.
- 의미 있는 테스트를 지향해주세요. 단위 테스트보다 통합 테스트를 선호합니다.
- 테스트는 빠르고, 격리되어 있으며, deterministic(트랜젝션, 롤백) 해야합니다.

## Mocking Rules

**Mocking 해야하는 경우**
- 타사 HTTP API(결제, OAuth 등)
- 파일 시스템, 시계, 난수 생성, 네트워크
- 제어할 수 없는 프로세스 경계를 넘나드는 모든 것

**Mocking 하지말아야 하는 경우**
- 데이터베이스/ORM (테스트 컨테이너와 함께 실제 데이터베이스 사용)
- 내부 모듈(DI) 및 Repository 구현체, UseCase, Domain 로직 
- 순수한 기능이나 유틸리티

## Assertion Rules

- 반환 값과 관찰 가능한 상태(DB, API 응답, 이벤트) 모두에 대해 Assertion을 수행해야 합니다.
- 상호 작용 검증보다 상태 검증을 우선해야 합니다.(Spy 테스트 우선 방식 금지).
- 객체 전체 비교는 결정적이고 안정적인 경우에만 수행하십시오. 그렇지 않은 경우에는 모든 필드가 아닌 의미 있는 필드에 대해서만 Assertion을 수행해야 합니다.
- 비결정적 출력(타임스탬프, ID, LLM, 순서가 지정되지 않은 집합)은 스냅샷으로 저장하면 안됩니다.
- 올바른 추상화 수준에서 어설션을 수행하십시오(구현 세부 정보 유출 금지).
- 부작용(DB 쓰기, 상태 변경)은 항상 명시적으로 검증하십시오.

## Naming Rules

- 테스트 이름은 구현 방식이 아닌 관찰 가능한 동작을 설명해야 합니다.
- 주어 중심의 명명 방식보다는 동작 중심의 명명 방식을 선호합니다.

Template:
<action>_<expected_behavior>_when_<condition>

Examples:
- create_todo_succeeds_when_input_is_valid
- create_todo_fails_when_title_is_empty
- login_rejects_when_password_is_incorrect
- get_user_returns_none_when_not_found

Rules:
- 일관된 동사 사용(create, get, update, delete, returns, fails, rejects)
- 모호한 용어 사용 자제 (works, handles, processes)
- 조건을 명확하고 구체적으로 작성
- 간결하면서도 설명적인 이름 유지


## Structure Rules

| Layer | Purpose | Budget |
|---|---|---|
| Unit | Pure logic (algorithms, domain rules) | Many (only when non-trivial) |
| Integration | Usecase + real DB/cache/queue | 1–3 per usecase |
| E2E | Critical user journeys (API level) | 1 per journey |
| Regression | Reproduce past bugs | 1 per incident |

Rules:

- 통합 테스트는 계층이 아닌 유스케이스를 대상으로 합니다.
- 항상 실제 DB를 사용합니다(리포지토리에 대한 모킹은 금지).
- E2E 테스트는 핵심 흐름만 다룹니다.
- 단위 테스트는 복잡한 로직에 대해서만 수행합니다(게터, DI, 연결 코드는 금지).
- 단위 테스트는 소스 코드와 함께 배치하고, 통합/E2E 테스트는 분리합니다.

Execution:

- 모든 DB 기반 테스트는 기본적으로 실행됩니다.
- 외부 시스템(타사 API)만 환경 변수 플래그로 접근을 제한합니다.

Performance:

- 테스트는 빨라야 합니다(통합 테스트당 100ms 미만).
- 가능한 경우 컨테이너를 재사용합니다.
- 전체 데이터베이스 초기화보다는 트랜잭션 롤백을 선호합니다.

Quality:

- 테스트는 결정적이고 격리되어야 합니다.
- 각 테스트는 독립적이어야 합니다.
- 모든 운영 환경 버그에 대해 회귀 테스트를 추가합니다.

## Domain Entity Rules

다음 중 **하나라도** 참이면 도메인 엔티티를 추출합니다.
- 비즈니스 로직이 동일한 데이터를 사용하는 2개 이상의 서비스에 분산되어 있는 경우
- 서비스가 일반 DB 행에 대해 산술 연산이나 상태 전환을 수행하는 경우
- 실제 로직은 아니지만, 테스트를 위해 별도의 DB를 생성해야 하는 경우

```
# Before — logic in the service, tied to ORM
user.hunger = user.hunger - EAT * 2
user.energy = user.energy + SLEEP * 2
db.user.update(user)

# After — logic in the entity, service only persists
user.eat()
user.sleep()
user_repo.save(user)
```

이렇게 하면 `User.eat()`는 순수 메모리 기반 단위 테스트가 됩니다. 밀리초 단위의 시간 내에 실행되고, 모의 객체도 필요 없으며, 드리프트도 없습니다.

## Property-Based Testing

명확한 불변 조건을 가진 순수 로직에는 속성 기반 테스트를 사용하세요.
- validators
- domain rules
- parsers / mappers
- state machines
- calculations

Rule:
- 동일한 함수에 대해 네 번째 예제 테스트를 작성해야 한다면 속성 기반 테스트로 전환하세요.

Requirements:
- 각 속성은 명확한 불변 조건을 정의해야 합니다.
- Assertions은 결정적이고 디버깅 가능해야 합니다.
- 유사한 예제 테스트를 여러 개 작성하는 것보다 속성을 우선적으로 사용하세요.

다음과 같은 경우에는 속성 기반 테스트를 사용하지 마세요.
- 데이터베이스 상호 작용
- 부작용이 있는 사용 사례
- 외부 시스템

Performance:
- 속성 기반 테스트는 빠르고 메모리 내에서만 실행되어야 합니다.

## Flaky Test Rules

- 불안정한 테스트는 절대 커밋하지 마세요. 만약 커밋되었다면 24시간 이내에 격리하세요.
- 격리란, 이슈, 담당자, 마감일을 연결하여 건너뛰는 것을 의미합니다. 담당자가 없으면 삭제합니다.
- 불안정성의 근본 원인을 해결하세요. 재시도 루프, `sleep()`, 또는 타임아웃 시간을 늘리는 방식은 절대 사용하지 마세요.
- 일반적인 원인: 공유되는 전역 상태, 실제 시간, 테스트 순서, 시드가 없는 난수 생성, 네트워크 문제. 증상이 아닌 근본 원인을 해결하세요.

## Migration Rules (existing Mockist codebase)

테스트 코드를 완전히 새로 작성하지 마십시오. 단계적으로 마이그레이션하십시오.

1. 새로운 테스트
- 모든 새로운 테스트는 다음 규칙을 따라야 합니다.
- 내부 모킹 사용 금지
- 데이터 영구 저장을 위한 실제 데이터베이스 사용
- 동작 및 관찰 가능한 상태 검증

2. 수정 파일
- 테스트 수정 시:
- 내부 협업자를 위한 Mocking 제거
- 실제 외부 경계를 위한 Mocking만 유지

3. 문제가 심각한 파일부터 우선 처리
- 다음 조건을 충족하는 3~5개 파일의 우선순위를 정합니다.
- 상호작용 기반 검증이 많은 파일
- 변경 빈도가 높거나 오류가 자주 발생하는 파일
- 핵심 비즈니스 로직을 포함하는 파일

4. 실제 데이터베이스를 점진적으로 도입
- 다음 도메인 중 하나부터 시작합니다.
- 버그 발생률이 가장 높거나 스키마가 가장 간단한 도메인
- 다음 사항을 검증합니다.
- 속도 (<100ms/테스트)
- 결정성
- CI 안정성
- 패턴이 검증된 후에만 확장

5. 스냅샷 제거
- 비결정적 출력을 위한 스냅샷 삭제
- 구조적 또는 불변성 검증으로 대체

6. 완료 기준 (도메인별)
- 상호작용 기반 검증 금지
- 내부 Mocking 사용 금지
- 테스트에서 실제 데이터베이스 사용 (해당되는 경우)
- 테스트는 결정론적이고 안정적입니다.

7. 성능 가이드라인
- 테스트 실행 시간이 2배 이상 증가하는 경우:
- 진행하기 전에 최적화(트랜잭션 롤백, 컨테이너 재사용)

8. 안전한 마이그레이션
- 새 테스트가 안정될 때까지 기존 테스트를 유지합니다.
- 작은 PR(한 번에 하나의 도메인)로 마이그레이션 합니다.

9. 일시적인 불일치는 허용됩니다.
- 전환 기간 동안 혼합된 스타일이 허용됩니다.

## Workflow Rules

- 구현이 아닌 관찰 가능한 동작(사양)을 기반으로 테스트를 작성하세요.
- 동작이 명확할 경우 테스트 우선 방식을 선호하세요.
- 데이터베이스/인프라 작업의 경우, 최소한의 구현을 먼저 작성한 후 테스트를 추가하는 것이 좋습니다.
- 코드를 먼저 생성한 후 그로부터 테스트를 도출하지 마세요.(코드 커버리지 보여주기식 행태를 피하세요)
- 하나의 테스트는 하나의 동작 시나리오에 대응합니다. 동일한 동작을 설명하는 여러 개의 어설션은 허용됩니다.
- 레이어별이 아닌 유스케이스 수준에서 테스트를 작성하세요.
- red → green → refactor 순서를 따르되, 간결하게 유지하세요.
- 테스트 실패는 올바른 이유로 발생해야 합니다.
- 동작 누락이 원인이어야 하며, 설정 오류가 원인일 이유는 없어야 합니다.

## PR Red Flags — Reject or Rework

Reject:
- 테스트가 상호 작용만 검증함(모의 호출, 검증, toHaveBeenCalledWith 등)
- 반환 값 또는 관찰 가능한 상태에 대한 검증 없음
- 실제 데이터베이스 대신 모의 데이터베이스/리포지토리 사용
- 비공개/내부 모듈(공개되지 않은 API)에서 가져오기
- 비결정적 출력(시간, 무작위, 외부, LLM 등)의 스냅샷 사용
- 문제, 담당자, 이유 없이 무시/건너뛴 테스트
- 테스트 이름이 동작이 아닌 함수 이름을 따옴
- 경계에 대한 정당성 없이 새로운 모킹 프레임워크(예: mockall) 추가
- 불안정한 테스트(타이밍, 순서, 공유 상태 등)

Review:
- 상태 검증보다 상호 작용 검증이 더 많음
- 실제 검증에 비해 테스트 설정이 과도함
- 간단한 로직에 비해 테스트 파일 크기가 지나치게 큼

## When NOT to Write a Test

- 비즈니스 규칙이 없는 단순 CRUD → 통합 테스트/E2E 테스트로 처리
- 프레임워크 내부 → 직접 테스트하지 않음(통합 테스트를 통해 간접적으로 처리)
- 정적 설정/상수 → 타입/스키마에 의존(단, 파싱 및 중요한 기본값은 테스트해야 함)
- 일회성 스크립트 → 프로덕션 데이터에 영향을 미치지 않는 한 테스트하지 않음
- 삭제 예정인 코드

Always test:
- 비즈니스 핵심 로직 (인증, 결제, 권한)
- 복잡한 분기 또는 상태 전환
- 이전에 오류가 있었던 부분 (회귀 테스트)

Guideline:
- 동작을 명확하게 설명할 수 없는 경우, 테스트를 세분화하거나 분할하세요. 복잡한 로직 테스트는 건너뛰지 마세요.

## 벤치마크 작성 기준

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
Job 6: cargo bench (선택)          성능 회귀 확인
```

```bash
# 로컬 사전 확인 (PR 올리기 전)
cargo test --all
cargo tarpaulin --out Html
```
