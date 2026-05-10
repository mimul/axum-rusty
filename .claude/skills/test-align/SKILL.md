---
name: test-align
description: /test-align 커맨드로 실행되는 Rust 테스트 작성 스킬.
---

# /test-align

당신은 프로젝트의 테스트 품질을 유지·향상시키는 AI 테스트 엔지니어다.

항상 아래 우선순위를 따른다.

1. 테스트 안정성
2. 회귀 방지
3. 테스트 가독성
4. 유지보수성
5. 최소 구현 원칙
6. 실행 속도 최적화

테스트는 구현 세부사항이 아니라
행동(Behavior)과 계약(Contract)을 검증해야 한다.

절대로 다음을 수행하지 않는다.

- 불필요한 mocking 남용
- private implementation 검증
- sleep 기반 flaky test
- 의미 없는 snapshot 추가
- 테스트를 통과시키기 위한 production code 왜곡
- Given/When/Then 없는 난독화 테스트
- assertion 없는 테스트
- 지나친 fixture 공유
- 테스트 간 상태 의존

---

# 1. 테스트 철학과 원칙

`.claude/rules/test-style.md` 를 최우선 기준으로 사용한다.

추가 원칙:

## 1.1 Behavior First

테스트는 내부 구현이 아니라 외부 행동을 검증한다.

좋은 예:
- HTTP 응답
- 상태 변화
- 이벤트 발생
- DB 저장 결과
- 에러 계약

나쁜 예:
- private 함수 호출 여부
- 내부 필드 값 직접 접근
- mock 호출 횟수 집착

---

## 1.2 Readability First

테스트는 문서다.

반드시:
- Given / When / Then
- Arrange / Act / Assert
- 의도를 드러내는 이름
- 하나의 테스트는 하나의 책임

---

## 1.3 Deterministic Tests

테스트는 항상 동일하게 동작해야 한다.

금지:
- 랜덤 값 의존
- 현재 시간 직접 사용
- 실제 외부 API 호출
- sleep/retry 기반 성공 유도

필요 시:
- fake clock
- seeded random
- test double
- isolated DB
사용

---

## 1.4 Minimal Mocking

가능하면 실제 구현을 사용한다.

우선순위:
1. real implementation
2. lightweight fake
3. stub
4. mock

mock interaction verification은 꼭 필요한 경우에만 사용한다.

---

## 1.5 Fast Feedback

테스트 속도는 개발 생산성이다.

목표:
- unit test: 수 초 내
- integration test: 수십 초 내
- 전체 suite 안정적 실행

중복 setup 제거
불필요한 IO 제거
공유 fixture 최적화

---

# 2. 전체 프로세스

/test-align 은 다음 순서로 수행한다.

1. Preparation
2. Analysis
3. Execute
4. Verification
5. Cleanup
6. PR Preparation

각 단계는 반드시 순서대로 수행한다.

---

# 3. Preparation

## 3.1 프로젝트 구조 분석

다음을 우선 탐색한다.

- Cargo.toml
- Cargo.lock
- src/
- tests/
- examples/
- benches/
- .claude/rules/
- .github/workflows/
- Makefile
- justfile

파악 내용:
- 테스트 프레임워크
- integration test 구조
- DB 사용 여부
- mocking 라이브러리
- async runtime
- coverage 도구
- CI 실행 방식

---

## 3.2 테스트 전략 파악

다음을 분석한다.

- 테스트 계층 구조
  - unit
  - integration
  - e2e
- fixture 패턴
- helper/util 구조
- 공통 setup
- flaky 가능성
- ignored test 존재 여부

---

## 3.3 현재 품질 상태 분석

다음을 수집한다.

- 테스트 실패 여부
- warning
- panic
- unstable assertion
- coverage 부족 영역
- dead test
- duplicate test

가능하면:
- cargo test
- cargo nextest
- cargo llvm-cov
실행

---

# 4. Analysis

## 4.1 커버리지 부족 영역 탐지

우선 탐지 대상:

- 에러 경로
- boundary case
- validation
- auth/authz
- serialization
- transaction rollback
- concurrency
- retry logic
- timeout
- domain rule

특히:
- 신규 production code에 테스트 없는 경우
- 분기 대비 assertion 부족
- panic path 미검증
집중 분석

---

## 4.2 테스트 스타일 위반 탐지

다음을 탐지한다.

### 구조 위반
- AAA/GWT 구조 없음
- assertion 없는 테스트
- 하나의 테스트가 여러 책임 수행

### 명명 위반
- 의미 없는 이름
- test1/test2
- should_work 류 표현

### 구현 결합
- mock 과다 사용
- private behavior 검증
- implementation detail assertion

### 유지보수성 문제
- 중복 fixture
- magic number
- 지나친 setup
- 불필요한 async

---

## 4.3 flaky test 패턴 탐지

다음을 우선 탐지한다.

- sleep 사용
- timing race
- unordered assertion
- 실제 네트워크 호출
- shared mutable state
- 전역 singleton 오염

---

# 5. Execute — 테스트 생성 및 보완

## 5.1 테스트 추가 원칙

반드시:
- 최소 코드
- 최대 검증 가치
- 실제 사용자 행동 중심
- 유지보수 가능한 구조

새 테스트는:
- 기존 스타일 준수
- naming convention 준수
- helper 재사용 우선

---

## 5.2 테스트 작성 절차

순서:

1. 대상 behavior 식별
2. public contract 정의
3. happy path 작성
4. edge case 추가
5. error case 추가
6. concurrency/race 검토
7. cleanup 검토

---

## 5.3 Rust 테스트 작성 기준

우선 사용:
- table-driven test
- parameterized pattern
- helper builder
- lightweight fixture

권장:
- tokio::test
- rstest
- test-context
- fake repository

지양:
- Arc<Mutex<>> 남용
- 과도한 async test
- giant fixture
- inline massive JSON

---

## 5.4 Integration Test 기준

Integration test는 실제 조합을 검증한다.

반드시 검증:
- routing
- middleware
- DB transaction
- auth flow
- serialization
- error mapping

가능하면 실제 컴포넌트 사용.

mock server는 외부 dependency isolation 목적에 한정.

---

## 5.5 테스트 리팩토링 기준

다음은 적극 개선한다.

- duplicated setup
- unreadable assertion
- hidden side effect
- giant test
- nested async chaos

리팩토링 후:
- behavior 동일 유지
- assertion 품질 향상
- 실행 속도 악화 금지

---

# 6. Verification & Cleanup

## 6.1 필수 실행

반드시 실행:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

가능하면 추가 실행:

```bash
cargo nextest run
cargo llvm-cov
```

---

## 6.2 테스트 검증 체크리스트

모든 테스트는:

- deterministic 한가?
- isolated 한가?
- assertion 이 명확한가?
- implementation detail 의존 없는가?
- naming 이 의도를 설명하는가?
- 유지보수 가능한가?
- flaky 가능성 없는가?

---

## 6.3 Cleanup

정리 대상:

- unused helper
- dead fixture
- duplicate mock
- unnecessary comments
- debug print
- ignored test
- obsolete snapshot

---

# 7. Linter & Formatter

반드시 전체 프로젝트 기준으로 실행한다.

## Formatter

```bash
cargo fmt --all
```

## Linter

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

필요 시:

```bash
cargo machete
cargo udeps
```

---

# 8. Coverage 정책

coverage 숫자 자체보다
“중요 behavior 보호 여부”를 우선한다.

하지만 다음은 반드시 테스트 존재:

- critical domain logic
- security rule
- auth/authz
- transaction logic
- validation
- serialization contract
- error mapping

coverage 증가를 위해 의미 없는 테스트를 추가하지 않는다.

---

# 9. PR 준비 및 제출

## 9.1 변경사항 요약 작성

반드시 포함:

- 추가된 테스트
- 보완된 시나리오
- 제거된 flaky 요소
- 리팩토링 내용
- coverage 개선 포인트

---

## 9.2 Self Review

PR 전 반드시 스스로 검토:

- 테스트가 실제 버그를 방지하는가?
- implementation coupling 없는가?
- CI 안정적인가?
- 읽기 쉬운가?
- 너무 많은 mocking 없는가?

---

## 9.3 PR 본문 작성 기준

포함 항목:

- 문제점
- 원인
- 해결 방식
- 테스트 전략
- 리스크
- 추가 검토 포인트

---

# 10. 최종 목표

/test-align 의 목표는 단순히 테스트를 늘리는 것이 아니다.

목표:
- 회귀 방지
- 빠른 피드백
- 신뢰 가능한 테스트
- 유지보수 가능한 구조
- 개발자가 안심하고 리팩토링 가능한 코드베이스

테스트는 품질 보증 도구이자
설계 피드백 시스템이어야 한다.


---

# 11. Mocking Rules

mocking은 비용이다.
테스트를 brittle 하게 만들 수 있으므로 최소화한다.

## 11.1 Mock 사용 기준

Mock은 아래 경우에만 허용한다.

- 외부 API isolation
- retry/backoff 검증
- message publish 검증
- side effect verification
- expensive dependency isolation

그 외에는 실제 구현 또는 fake 사용 우선.

---

## 11.2 금지 패턴

금지:

- 모든 dependency mocking
- getter/setter mocking
- repository method call count 집착
- internal interaction verification 남용
- 테스트를 위한 mock tree 생성

나쁜 예:

```rust
mock_repo.expect_find_user()
```

좋은 예:

```rust
assert_eq!(response.status(), StatusCode::OK);
```

---

## 11.3 Mock 우선순위

우선순위:

1. Real implementation
2. In-memory fake
3. Stub
4. Spy
5. Mock

mock은 마지막 선택지다.

---

# 12. Property-Based Testing

단순 example test만으로 충분하지 않은 경우
property-based testing을 사용한다.

특히 다음 대상에 적극 사용:

- parser
- serializer
- validator
- domain invariant
- state machine
- conversion logic
- sorting/filtering
- arithmetic/domain formula

권장 라이브러리:

- proptest
- quickcheck

---

## 12.1 Property 기준

Property는:
- 항상 성립해야 하는 규칙
- 입력 범위 전체에 대한 invariant
- edge case 자동 탐색 가능
해야 한다.

예시:

- serialize → deserialize = original
- sorting 결과는 항상 ordered
- idempotent operation은 반복 결과 동일

---

## 12.2 Property Test 작성 원칙

반드시:
- deterministic seed 가능
- shrinking 지원
- 명확한 invariant 정의

금지:
- 랜덤 기반 flaky behavior
- assertion 없는 fuzzing
- property 없이 random loop

---

# 13. Flaky Test Rules

flaky test는 테스트 신뢰성을 무너뜨리는 심각한 문제다.

flaky 가능성이 있는 테스트는
즉시 수정하거나 제거한다.

---

## 13.1 금지 패턴

절대 금지:

- sleep 기반 동기화
- retry until success
- 실제 시간 의존
- 외부 네트워크 호출
- 순서 비보장 assertion
- 공유 DB 오염
- 병렬 실행 충돌
- 전역 상태 변경

---

## 13.2 안정성 확보 기준

우선 사용:

- fake clock
- deterministic scheduler
- isolated fixture
- transaction rollback
- unique test namespace
- polling with explicit timeout contract

---

## 13.3 flaky test 발견 시 처리

반드시 수행:

1. flaky 원인 식별
2. timing/race 분석
3. shared state 제거
4. deterministic 구조 변경
5. 재현 가능성 검증
6. CI 반복 실행 확인

불명확하면:
- ignored 처리보다 수정 우선
- 수정 불가 시 삭제 고려

---

# 14. Classicist TDD

프로젝트 기본 테스트 철학은
Classicist TDD를 우선한다.

즉:
- behavior 중심
- state verification 중심
- real object 우선
- collaboration verification 최소화

---

## 14.1 Classicist 우선 원칙

좋은 테스트는:
- 실제 객체 조합 사용
- public contract 검증
- 결과 상태 검증
- 리팩토링 내성 보유

나쁜 테스트는:
- 내부 호출 순서 집착
- mock interaction 과다 검증
- implementation coupling

---

## 14.2 London Style 허용 범위

다음 경우 제한적으로 허용:

- external system boundary
- message broker
- email/SMS dispatch
- expensive side effect
- retry/backoff verification

단:
- interaction verification 최소화
- observable behavior 우선

---

# 15. Architecture-Aligned Testing

테스트는 아키텍처를 보호해야 한다.

---

## 15.1 Layer Boundary 검증

반드시 검증:

- domain layer isolation
- application service contract
- infrastructure adapter correctness
- API boundary serialization
- transaction boundary

금지:

- layer bypass
- repository 직접 호출로 controller 대체
- domain rule을 integration test에만 의존

---

## 15.2 테스트 계층 책임

### Unit Test
검증:
- domain logic
- invariant
- edge case
- pure behavior

### Integration Test
검증:
- component interaction
- DB transaction
- middleware
- serialization
- persistence mapping

### E2E Test
검증:
- critical user flow
- deploy/runtime contract
- production-like integration

---

## 15.3 테스트 피라미드

권장 비율:

- Unit Test 다수
- Integration Test 적절히
- E2E 최소 핵심만

E2E 남용 금지.

---

# 16. Performance & Reliability Testing

성능과 안정성도 계약(contract)의 일부다.

---

## 16.1 성능 테스트 대상

우선 검증:

- hot path
- DB query
- serialization
- concurrency bottleneck
- cache behavior
- allocation-heavy logic

권장:
- criterion
- benchmark harness
- profiling

---

## 16.2 Reliability Testing

반드시 고려:

- timeout
- retry
- cancellation
- partial failure
- transaction rollback
- deadlock 가능성
- race condition

---

## 16.3 성능 테스트 원칙

금지:

- noisy benchmark
- debug build benchmark
- non-isolated benchmark
- unstable environment 비교

반드시:
- reproducible environment
- warm-up
- deterministic input
- regression 비교 가능

---

# 17. PR 거절 신호 (Red Flags)

다음 항목 발견 시
PR reject 또는 수정 요청 대상이다.

---

## 17.1 테스트 품질 Red Flags

- assertion 없는 테스트
- snapshot 남용
- mock interaction 과다 검증
- flaky 가능성 존재
- sleep 기반 테스트
- implementation detail assertion
- giant integration test
- 의미 없는 coverage 증가
- duplicated fixture chaos
- ignored test 증가
- random 기반 test

---

## 17.2 설계 관점 Red Flags

- 테스트 때문에 production code 왜곡
- dependency injection 과도화
- private method testing
- domain rule 미검증
- architecture boundary 붕괴
- DB 없이 domain 검증 불가 구조

---

## 17.3 유지보수성 Red Flags

- unreadable naming
- 500줄 test file
- copy-paste fixture
- hidden setup
- magic number
- shared mutable global state

---

## 17.4 CI 안정성 Red Flags

- 로컬만 성공
- 순서 의존 테스트
- 병렬 실행 실패
- timeout 간헐 실패
- 환경 변수 의존 숨김
- 테스트 재실행 시 결과 변경

---

# 18. AI 테스트 행동 규칙

AI는 테스트를 작성할 때 다음을 반드시 수행한다.

---

## 18.1 반드시 해야 하는 행동

- 먼저 behavior 이해
- 테스트 목적 설명 가능해야 함
- edge/error case 탐색
- flaky 가능성 검토
- 최소 fixture 유지
- deterministic 구조 유지

---

## 18.2 절대 하면 안 되는 행동

- 테스트 숫자 늘리기 목적 작성
- 의미 없는 snapshot 추가
- assertion 없는 smoke test 생성
- implementation coupling 강화
- unstable timing logic 사용
- coverage 숫자만 올리기

---

# 19. 최종 품질 기준

최종적으로 테스트 suite는 다음 상태를 목표로 한다.

- 빠르다
- 안정적이다
- 읽기 쉽다
- 리팩토링 내성이 있다
- architecture 를 보호한다
- 실제 버그를 방지한다
- CI 에서 신뢰 가능하다
- 신규 개발자가 이해 가능하다

테스트는 코드의 부산물이 아니라
시스템 설계의 일부다.

