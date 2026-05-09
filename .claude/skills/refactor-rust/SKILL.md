---
name: refactor
description: /refactor 커맨드로 실행되는 Rust 코드 리팩토링 스킬.
---

# Claude용 리팩토링 가이드

## 1. 리팩토링 원칙 (Refactoring Principles)

### 1.1 Behavior Preserving
- 리팩토링은 **동작 변경 없이 구조를 개선**하는 활동이다.
- 기능 추가와 리팩토링을 같은 커밋에 섞지 않는다.
- 작은 단위로 변경하고 매 단계마다 검증한다.

### 1.2 Small Safe Steps
- 한 번에 큰 구조 변경을 하지 않는다.
- 항상 작은 변경, 즉시 테스트, 즉시 검증, 즉시 커밋 순서로 진행한다.
- 실패 시 쉽게 rollback 가능해야 한다.

### 1.3 Readability First
- 코드는 작성보다 읽기가 더 많다.
- 코드 길이보다 의도 전달, 응집도, 변경 용이성을 우선한다.

### 1.4 Domain First
- 기술 구조보다 도메인 개념을 우선한다.
- 도메인 모델이 비즈니스 개념을 자연스럽게 표현해야 한다.
- Primitive Obsession을 제거한다.

### 1.5 Simplicity First
- 미래를 위한 추상화보다 현재 문제를 명확히 해결하는 구조를 선호한다.
- Speculative Generality를 피한다.

### 1.6 Explicit Intent
- 이름만 보고 역할을 이해할 수 있어야 한다.
- 숨겨진 side effect를 제거한다.
- 상태 변경은 명시적으로 표현한다.

### 1.7 Cohesion & Loose Coupling
- 함께 변경되는 코드는 함께 위치시킨다.
- 불필요한 의존성을 제거한다.
- Message Chain / Middle Man / Shotgun Surgery를 줄인다.

### 1.8 Testability
- 테스트하기 어려운 코드는 설계 문제가 존재할 가능성이 높다.
- 테스트 용이성을 설계 품질의 핵심 지표로 본다.

### 1.9 Refactor Continuously
- 리팩토링은 별도 이벤트가 아니라 지속적 활동이다.
- Boy Scout Rule: "코드를 발견했을 때보다 더 깨끗하게 남긴다.""

---

# 2. Refactoring Process (단계별 실행 절차)

## 2.1 Preparation

### 2.1.1 목표 정의
리팩토링 목적을 명확히 정의한다.

예:
- 가독성 개선
- 복잡도 감소
- 중복 제거
- 테스트 용이성 향상
- 도메인 모델 개선
- 성능 병목 제거
- 결합도 감소
- 보안 취약 구조 개선

---

### 2.1.2 변경 범위 식별
아래를 분석한다.

- 영향받는 모듈
- 의존 관계
- API 계약
- 데이터 흐름
- 상태 변경 지점
- 트랜잭션 경계
- 동시성 영향
- 외부 시스템 영향

---

### 2.1.3 Existing Tests 확인
반드시 확인:
- Unit Test 존재 여부
- Integration Test 존재 여부
- E2E Test 존재 여부
- Critical Path 보호 여부

부족하면 먼저 테스트를 작성한다.

특히:
- 현재 동작 캡처(Characterization Test)
- 회귀 방지 테스트
- Edge Case 테스트

---

### 2.1.4 Code Smell 분석

## Bloaters
- Long Method
- Large Class
- Long Parameter List
- Primitive Obsession
- Data Clumps

## OO Abusers
- Switch Statement 남용
- Temporary Field
- Refused Bequest
- Alternative Classes with Different Interfaces

## Change Preventers
- Divergent Change
- Shotgun Surgery
- Parallel Inheritance Hierarchies

## Dispensables
- Duplicate Code
- Dead Code
- Lazy Class
- Speculative Generality
- 과도한 Comment

## Couplers
- Feature Envy
- Inappropriate Intimacy
- Message Chain
- Middle Man

---

### 2.1.5 Refactoring 전략 선택

문제 유형에 따라 전략 선택:

| 문제 | 대표 리팩토링 |
|---|---|
| Long Method | Extract Method |
| Primitive Obsession | Replace Primitive with Object |
| Duplicate Code | Extract Function / Template Method |
| Large Class | Extract Class |
| Switch 남용 | Strategy / Polymorphism |
| Shotgun Surgery | Move Method / Move Field |
| Message Chain | Hide Delegate |
| Long Parameter | Parameter Object |

---

### 2.1.6 리스크 분석
다음을 반드시 점검한다.

- Public API 변경 여부
- Backward Compatibility
- Migration 필요 여부
- 데이터 손상 가능성
- 성능 영향
- Lock/Concurrency 영향
- Security 영향

---

## 2.2 Execute Refactoring

### 2.2.1 작은 단위로 진행

- 절대 금지 : 대규모 변경을 한 번에 수행하지 않고, 기능 추가와 리팩토링을 같이 작업하는 것을 금지한다.
- 권장 사항 : 작업 단위를 작은 단위로 쪼개고, 작업 단위로 브랜치는 "feature/refactor-작업단위" 함축적 의미로 만들고 아래 사항들을 리팩토링을 진행하고 테스트하고, 커밋을 반복 수행한다.

---

### 2.2.2 Naming 개선
다음을 개선한다.

- 의미 없는 변수명 제거
- 타입 기반 이름 제거
- 축약어 최소화
- 도메인 용어 사용
- Boolean Flag 제거

예:
- `data` → `orderItems`
- `flag` → `isExpired`

---

### 2.2.3 함수 리팩토링

목표:
- 단일 책임
- 의도 중심
- Side Effect 최소화

체크:
- 함수가 여러 역할 수행하는가?
- 조건문이 과도한가?
- depth가 깊은가?
- mutable state가 많은가?

---

### 2.2.4 클래스 리팩토링

체크:
- 클래스 책임이 여러 개인가?
- 데이터와 행동이 분리되어 있는가?
- 도메인 규칙이 서비스에 흩어져 있는가?

개선:
- 응집도 증가
- 캡슐화 강화
- 도메인 로직 이동

---

### 2.2.5 조건문 리팩토링

다음을 우선 제거한다.

- 거대한 if/else
- switch 분기
- 상태 기반 분기

대체:
- Polymorphism
- Strategy Pattern
- State Pattern
- Lookup Table

---

### 2.2.6 데이터 구조 개선

다음을 제거한다.

- Primitive Obsession
- Magic Number
- Stringly Typed 구조

대체:
- Value Object
- Enum
- Domain Type

---

### 2.2.7 Dependency 정리

체크:
- 순환 참조
- 숨겨진 의존성
- 테스트 어려운 구조
- 전역 상태 사용

개선:
- Dependency Injection
- Interface 분리
- Layer 명확화

---

### 2.2.8 Dead Code 제거

제거 대상:
- 사용되지 않는 함수
- 사용되지 않는 클래스
- obsolete feature flag
- obsolete comment
- commented-out code

---

### 2.2.9 Comments 제거

설명용 comment보다 이름 개선, 함수 분리, 구조 개선을 우선한다. “what” comment는 code smell 가능성이 높다.

---

## 2.3 Verification & Cleanup

### 2.3.1 전체 테스트 실행

반드시 수행:
- Unit Test
- Integration Test
- E2E Test
- Regression Test

실패 시:
- 원인 분석
- behavior change 여부 확인

---

### 2.3.2 Static Analysis 수행

확인:
- unused code
- nullability
- race condition
- unreachable code
- complexity 증가 여부

---

### 2.3.3 Architecture Review

검증:
- 계층 위반 여부
- 의존 방향
- 도메인 경계
- Aggregate 경계
- Transaction boundary

---

### 2.3.4 Complexity Review

측정:
- Cyclomatic Complexity
- Cognitive Complexity
- Method Length
- Class Size
- Dependency Depth

리팩토링 후 감소했는지 확인한다.

---

### 2.3.5 Diff Review

확인:
- 불필요한 formatting noise 제거
- 기능 변경 섞이지 않았는가
- rename-only commit 분리 가능한가

---

# 3. Test Review / 피드백 반영

`/test-review` 명령을 실행히고 결과 피드백을 자동 수정한다.

---

# 4. Security Scan / 피드백 반영

1. `/security-full-scan` 명령을 실행해 정적분석을 진행하고 결과 피드백을 분석해 자동 수정한다.
2. `/security-scan` 명령을 수행해 동적 분석을 진행하고 결과 피드백을 분석해 자동 수정한다. 정적 분석의 경우 서버가 구동되지 않았으면 `cargo run` 명령어를 실행해 서버를 실행한 다음 `/security-scan` 명령을 다시 실행해 이후 작업을 수행한다.

위 명령을 수행하기 위해서는 [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다. 설치가 안되었을 경우 인간에게 설치를 안내한다.

---

# 5. Linter & Formatter 실행
아래사항을 실행하고 결과 피드백을 개선한다.

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

# 6. 피드백 작성 가이드라인

## 6.1 리백토링 피드백 분류

리팩토링 결과는 아래 4가지 카테고리로 분류한다.

| 분류 | 의미 | 대응 |
|---|---|---|
| 🚫 Blocking Refactoring Issues | 반드시 수정이 필요한 구조적 문제 | 다음 단계 진행 전 필수 수정 |
| ⚠️ Recommended Refactoring Changes | 유지보수성과 안정성을 위한 권장 개선 | 가능하면 현재 작업에서 반영 |
| 💡 Refactoring Suggestions | 선택적 개선 아이디어 및 리팩토링 기회 | 향후 개선 후보로 고려 |
| 📝 Refactoring Tech Debt | 현재 수정 범위를 넘어서는 구조적 부채 | 별도 이슈로 추적 |


## 6.2 리팩터링 피드백 분류 기준

**1. 🚫 Blocking Refactoring Issues**

반드시 수정해야 하는 항목. 다음에 해당하면 Blocking으로 분류한다.

- behavior change 발생 가능성
- public API compatibility 문제
- transaction boundary 손상
- validation/auth logic 누락
- concurrency 문제 가능성
- rollback 어려움
- architecture rule 위반
- cyclic dependency 생성
- 테스트 실패
- security regression
- data corruption 가능성
- excessive complexity 증가
- unsafe refactoring

**2. ⚠️ Recommended Refactoring Changes**

강하게 권장되는 개선 사항. 다음에 해당하면 Recommended로 분류한다.

- long method 개선 가능
- large class 분리 필요
- duplicate code 존재
- dependency direction 개선 가능
- testability 부족
- naming 품질 개선 필요
- unnecessary abstraction 존재
- excessive nesting
- readability 저하
- weak encapsulation
- magic number/string 존재
- excessive mutable state

**3. 💡 Refactoring Suggestions**

선택적 개선 아이디어이다. 다음에 해당하면 Suggestion으로 분류한다.

- future extensibility 개선 가능
- reusable abstraction 가능
- performance optimization 여지
- domain model refinement 가능
- modern language idiom 적용 가능
- helper/util extraction 가능
- builder/factory 적용 가능
- async optimization 가능


**4. 📝 Refactoring Tech Debt**

현재 범위를 넘는 기술 부채이다. 즉시 수정하지 않지만 추적 필요하다. 다음에 해당하면 Tech Debt로 분류한다.

- legacy architecture limitation
- monolith boundary 문제
- outdated framework dependency
- missing integration test infra
- migration 필요한 구조
- global state architecture
- insufficient observability
- low test coverage
- inconsistent domain modeling
- duplicated business logic across modules


**6.3 리팩토링 결과 피드백 가이드라인**

리팩토링 결과 피드백은 코드 냄새 유형 + Before/After 비교 형식으로 작성한다:

코드 위치: 파일명과 라인 번호를 명시 (예: src/domain/order/service.rs:42)
냄새 유형: Code Smell 분석, Refactoring 전략 선택의 하위 단락의 유형을 표시
문제 설명(Problem) : 왜 문제가 되는가, 어떤 위험이 있는가, 어떤 영향(API 영향, transaction 영향, oncurrency 영향, rollback risk, migration 필요 여부)이 있는지 구체적으로 기술
개선 방향(Recommendation) : 가능하면 refactoring strategy, pattern, extraction 방향, dependency 개선 방향을 포함
Before/After: 리팩토링 전후 코드 예시 제공
우선순위: 각 항목의 우선순위(🚫/⚠️/💡/📝) 명시

---

# 7. PR 준비 및 제출

## 7.1 PR 제목 규칙

refactor([모듈명]): [핵심 변경 내용 한 줄 요약]

## 7.2 PR 본문

주요 변경 사항 단락에 리팩토링 결과 피드백의 내용 전체를 간략한 버전으로 정리해서 기술한다.

검증 내용에 테스트 결과 security scan 결과를 기술한다.

## 7.3 최종 완료 조건

다음을 만족해야 완료로 간주한다.

- 모든 테스트 통과
- lint 0 warning
- security scan 통과
- dead code 제거 완료
- backward compatibility 확인
- PR 설명 최신 상태 유지
- 작은 단위 commit 유지
- rollback 가능 상태 확보

---

# 8. Claude Skill Command Style

권장 명령은 아래 수준으로 단순하게 유지한다.

```bash
/refactoring [scope]
```

필요 시에만 아래 옵션을 추가한다.

```bash
--goal
--level
--with-tests
--dry-run
```

예:

```bash
/refactoring
/refactoring payments
/refactoring ./src/domain/order
/refactoring payments --goal maintainability
/refactoring legacy --dry-run
```

## Scope 해석 규칙

scope가 없으면 프로젝트 전체를 대상으로 한다. scope는 아래 중 하나로 해석한다.

- module
- directory
- file
- glob pattern

Claude는 scope를 기반으로 영향 범위와 의존성을 분석한다.

## Goal 해석 규칙

### readability
- naming 개선 우선
- extract method 우선
- 불필요한 comment 제거
- 함수 depth 감소

### maintainability
- dependency reduction 우선
- duplication 제거
- large class 분리
- 응집도 향상

### testability
- 테스트 어려운 구조 개선
- side effect 감소
- dependency injection 개선
- characterization test 추가

### domain-model
- primitive obsession 제거
- domain object 강화
- business rule 응집
- domain terminology 우선

### complexity
- conditional logic 단순화
- nested structure 감소
- cognitive complexity 감소
- dead code 제거

## Level 해석 규칙

### safe
safe level에서는:
- behavior preserving을 최우선으로 한다
- public API 변경을 피한다
- architecture rewrite를 지양한다
- rename / extract method 중심으로 진행한다
- 작은 단계로 나누어 수행한다
- 각 단계마다 테스트를 수행한다

### moderate
moderate level에서는:
- 일반적인 구조 개선을 허용한다
- 클래스 분리와 dependency cleanup을 허용한다
- 내부 API 개선을 허용한다
- maintainability 향상을 우선한다

### aggressive
aggressive level에서는:
- architecture 개선을 허용한다
- domain restructuring을 허용한다
- large-scale class split을 허용한다
- legacy abstraction 제거를 허용한다
- 다만 behavior verification은 반드시 유지한다

## Test 정책

`--with-tests` 옵션이 있으면 Claude는:

- characterization test
- regression test
- edge case test
- flaky test 개선

을 함께 수행한다.

테스트가 부족한 경우:
- 기존 동작을 먼저 캡처한다
- behavior preserving verification을 우선한다

## Dry Run 정책

`--dry-run` 옵션이 있으면 실제 수정 대신:

- code smell 분석
- architecture issue 분석
- dependency risk 분석
- 영향 범위 분석
- incremental 실행 계획
- 예상 위험도

를 우선 출력한다.

## 기본 실행 정책

옵션이 없더라도 Claude는 본 가이드의:

- Refactoring Principles
- Preparation 절차
- Execute Refactoring 절차
- Verification & Cleanup 절차
- Linter / Formatter 정책
- Test Review 정책
- Security Scan 정책
- PR 준비 정책

을 자동으로 적용한다.

즉:

```bash
/refactoring payments
```

만 수행해도 Claude는:

- code smell 분석
- dependency 분석
- 안전한 단계 분리
- 테스트 검증
- lint / formatter 검증
- dead code 제거
- security regression 검토
- PR 전략 생성

을 본 가이드 기준으로 수행한다.

## 중요 정책

리팩토링은 항상:

- 작은 단계로 수행한다
- 기능 추가와 섞지 않는다
- rollback 가능한 상태를 유지한다
- 테스트 없이 대규모 변경하지 않는다
- transaction boundary 변경 시 동시성 영향을 검토한다
- validation / auth logic 누락 여부를 반드시 검증한다

리팩토링의 목표는 단순 코드 이동이 아니라:

- 가독성 향상
- 유지보수성 향상
- 도메인 표현력 향상
- 변경 용이성 향상
- 안정성 향상

이다.
