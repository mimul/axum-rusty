---
name: code-review-rust
description: >
  도메인 중심 진화형 코딩 스타일을 기반으로 Rust 코드를 리뷰하기 위한 통합 기준.
  도메인 모델, 타입 설계, 경계 조건, 에러 처리, 테스트, 보안을 Rust의 타입 시스템을 통해 강제하고, 체크리스트(What)와 구현 패턴(How)을 함께 제공한다.
  /code-review-rust 커맨드의 SKILL.md가 이 문서를 참조하여 코드를 검토한다.
reference: SKILL.md
---

# CODE REVIEW RUST — Rust 코드 리뷰 기준 (통합)

## 리뷰 결과 분류 체계

리뷰 결과는 다음 4가지 카테고리로 분류한다:

| 분류 | 의미 | 대응 |
|------|------|------|
| **🚫 Blocking Issues** | 반드시 수정이 필요한 항목 (보안, 버그, 아키텍처 위반) | 머지 전 필수 수정 |
| **⚠️ Recommended Changes** | 권장 개선 사항 (성능, 가독성, 베스트 프랙티스) | 가능하면 이번 PR에 반영 |
| **💡 Suggestions** | 선택적 개선 아이디어 (리팩토링, 최적화 기회) | 향후 고려 |
| **📝 Tech Debt** | 향후 개선이 필요한 기술 부채 | 별도 이슈로 추적 |

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
## 리뷰 기준

### 1. 핵심 리뷰 원칙

- 도메인이 코드에 드러나야 한다
- 타입이 규칙을 강제해야 한다
- 경계 조건은 명시적으로 처리되어야 한다
- 테스트는 도메인을 검증해야 한다
- 보안은 기본적으로 포함되어야 한다

### 2. 도메인 모델 `R-01`

**2.1 체크**

- 도메인 개념이 타입으로 표현되는가?
- primitive(`String`, `i32`)를 직접 사용하지 않았는가?
- 동일 개념에 여러 이름이 존재하지 않는가? (`find_`/`get_`/`retrieve_` 혼용 금지)
- 로직이 타입 내부에 캡슐화되어 있는가?
- 함수·타입명이 RFC 430 네이밍 관례를 따르는가? (`UpperCamelCase`/`snake_case`/`SCREAMING_SNAKE_CASE`)
- 접근 권한이 최소화되어 있는가? (`pub` 대신 `pub(crate)` / `pub(super)` 우선)
- 공개 API(`pub fn`, `pub struct`, `pub trait`)에 `///` 문서 주석이 있는가?

**2.2 패턴**

- Value Object로 의미를 강제
- Smart Constructor로 invariant 보장
- Newtype 패턴으로 도메인 식별자 보호 (`UserId(i64)`, `OrderId(i64)`)
- 접근 권한 표: `(기본)` 비공개 → `pub(super)` → `pub(crate)` → `pub` (실제로 필요할 때만)

### 3. 상태 & 모델링 `R-02`

**3.1 체크**

- 상태가 `enum`으로 표현되는가?
- Invalid State가 타입으로 방지되는가? (enum variant에 데이터 부착)
- 상태 전이가 명확히 정의되어 있는가?
- Tell, Don't Ask 원칙이 적용되는가? (상태를 물어 외부에서 결정하지 않고 객체에 행동 위임)
- `_ => {}` wildcard 패턴으로 새 variant를 조용히 무시하지 않는가?

**3.2 패턴**

- Enum State Machine (variant에 데이터 부착: `Paid { paid_at, transaction_id }`)
- 상태 전이 메서드로 제한 (`order.mark_as_paid(tx_id)`)
- Tell Don't Ask: `if user.role == Admin { ... }` → `user.grant_access()?`

### 4. 경계 조건 & 에지 케이스 `R-03`

**4.1 체크**

- `Option` / `Result`로 상태를 표현했는가?
- 경계값(빈 컬렉션, 0, 최대값, 오버플로우)이 안전하게 처리되는가?
- 경계 입력이 검증되는가?
- `match`가 모든 variant를 명시적으로 처리하는가? (`_ => {}` 남용 금지)
- 에러를 정상 제어 흐름으로 사용하지 않는가? (`Err`로 정상 분기 표현 금지)
- `None`과 빈 컬렉션이 명확히 구분되는가?
- 컬렉션 반환 시 `None` 대신 빈 `Vec`를 반환하는가?

**4.2 패턴**

- Parse → Validate → Construct
- Fail Fast
- Exhaustive Match (wildcard 없이 모든 variant 명시)
- 에러 vs 정상 분기 구분: `Ok(user)` → 정상, `Err(...)` → 실제 실패만

### 5. 에러 처리 `R-04`

**5.1 체크**

- `unwrap()`/`expect()`가 라이브러리 코드에 없는가?
- 에러가 레이어별로 분리된 enum으로 정의되어 있는가? (`DomainError` / `AppError`)
- 문자열 기반 에러(`Box<dyn Error>`, `String`)가 아닌가?
- 에러가 도메인 의미를 가지는가?
- 에러 응답에 내부 구현 정보(스택 트레이스, DB 에러)가 노출되지 않는가?
- `thiserror`로 도메인 에러를 정의하고, `anyhow`는 바이너리 main에서만 사용하는가?

**5.2 패턴**

- `Result<T, DomainError>` (도메인 레이어)
- `Result<T, AppError>` (컨트롤러 레이어, `thiserror` + `IntoResponse`)
- `#[from]`으로 에러 레이어 전환, `#[source]`로 원인 분리·로그만 기록

### 6. 소유권 & 메모리 `R-05`

**6.1 체크**

- 불필요한 `clone()`이 없는가? (컴파일 오류 회피용 `clone` 금지)
- 함수 파라미터가 `String` 대신 `&str`, `Vec<T>` 대신 `&[T]`를 사용하는가?
- mutable 상태가 최소화되어 있는가?
- async 컨텍스트에서 `std::sync::Mutex` 대신 `tokio::sync::Mutex` / `RwLock`을 사용하는가?
- N+1 쿼리 패턴이 없는가? (반복문 내 DB 조회 → 배치 조회로 전환)

**6.2 패턴**

- 파라미터: `&str` / `&[T]` 우선 (호출자 선택권 보장)
- 공유 상태: `Arc<T>` / `tokio::sync::RwLock<T>`
- N+1 해결: `find_by_ids(&ids).await?` 배치 조회

### 7. 제어 흐름 `R-06`

**7.1 체크**

- `match`가 상태를 명확히 표현하는가?
- `_ => {}` 패턴으로 의미를 숨기지 않는가?
- 중첩 깊이가 2 이하인가?
- Tell, Don't Ask 원칙이 적용되는가?
- 복잡한 조건식이 의미 있는 변수로 분해되어 있는가?
- 에러를 제어 흐름으로 사용하지 않는가? (정상 분기를 `Err`로 표현 금지)

**7.2 패턴**

- Exhaustive match (모든 variant 명시)
- Early Return (조건을 함수 상단으로 올림)
- 조건 분해: `let can_access = is_admin && (has_recent_activity || is_superuser);`

### 8. 추상화 & trait `R-07`

**8.1 체크**

- trait가 필요한 경우에만 사용되는가? (Rule of Three: 3회 반복 이후에 도입)
- 제네릭이 과도하지 않은가?
- 테스트 편의만을 위한 추상화가 아닌가?
- 함수/메서드 파라미터에 `impl Trait`를 사용하는가? (구체 타입 대신)
- 반환 타입에도 `impl Iterator` 등 `impl Trait`를 우선 사용하는가?

**8.2 패턴**

- 최소 추상화 (YAGNI — 지금 필요하지 않은 trait 금지)
- `impl Trait` 파라미터: `fn send(sender: &impl NotificationSender, msg: &str)`
- `impl Trait` 반환: `fn active_users(users: &[User]) -> impl Iterator<Item = &User>`

### 9. 테스트 `R-08`

**9.1 체크** — `rust-test-style.md` §1~§15 기반

🚫 **Blocking** — 즉시 반려, §13.1 PR 거절 신호에 해당

| 항목 | 근거 |
|------|------|
| Assertion이 전혀 없는 테스트가 있는가? (`assert!`·`assert_eq!`·`assert_matches!` 없이 컴파일·실행만 확인) | §13.1 ⑧ |
| `assert!(result.is_some())`·`assert!(result.is_ok())` 처럼 타입 존재만 확인하는 의미 없는 Assertion만 있는가? | §13.1 ⑨ |
| `mockall`의 `expect_xxx().times(n)` 등 상호작용 검증만 있고 반환값·DB 저장 상태·HTTP 응답 같은 결과 상태 검증이 없는가? (이메일 발송·이벤트 발행처럼 side-effect가 비즈니스 요구사항인 경우 제외) | §13.1 ①, §5.2 |
| 통합 테스트에서 `MockOrderRepository` 같이 `mockall`로 DB·Repository를 대체하는가? (`in-memory Fake` 또는 `#[sqlx::test]` 실제 DB 사용 필요) | §13.1 ②, §4.3 |
| `SystemTime::now()`·시드 없는 `rand::random()`·LLM 응답처럼 비결정적 출력을 특정 값으로 단정하는가? (`Clock` 인터페이스 주입·`FakeClock` 사용 필요) | §13.1 ④, §10.2 |
| `#[ignore]`에 이슈 링크·담당자·기한·원인 가설 없이 단순 사유만 기재하는가? | §13.1 ⑤, §10.3 |
| `test_find_unique_called_once`·`test_calls_upsert_then_emits_event`처럼 테스트 이름이 함수명이나 내부 구현 구조를 그대로 반영하는가? | §13.1 ⑥, §3.1 |
| `mod tests` 외부에서 `pub(super)` 등을 이용해 비공개 구현에 직접 접근하는가? | §13.1 ③ |
| `wiremock`·`sqlx::test` 등 기존 도구로 충분한데 새 모킹 크레이트를 `Cargo.toml`에 추가하는가? | §13.1 ⑦ |

⚠️ **Recommended** — 머지 전 필수 수정

| 항목 | 근거 |
|------|------|
| 인증·권한·결제·도메인 상태 전환 등 핵심 비즈니스 로직에 테스트가 없는가? 이번 PR이 버그 수정인데 해당 버그를 재현하는 회귀 테스트가 없는가? | §6.3 |
| 메서드 이름 변경·루프→재귀 같은 순수 리팩토링이 테스트를 깨뜨리는가? (구현이 아닌 관찰 가능한 동작을 잠가야 함) | §1.1 |
| DB·캐시·같은 crate 내 협력자를 `mockall`로 Mock하는가? (자체 소유·통제하는 것은 실제 구현 또는 `in-memory Fake` 사용 필요) | §1.2, §4.3 |
| 서비스가 DB 행에 직접 산술 연산·상태 전환을 수행하여 도메인 로직을 DB 없이 단위 테스트할 수 없는가? (도메인 엔티티 추출 신호 — §8.1 판단 기준 하나 이상 해당 시) | §8.1, §8.2 |
| `mockall expect` 호출이 실제 `assert_eq!`·`assert_matches!`보다 압도적으로 많은가? | §13.2 ① |
| Arrange(설정·모킹) 코드가 Assert(검증) 코드보다 10배 이상 긴가? (Builder 패턴·Fixture 도입 신호) | §13.2 ②, §11.1 |

💡 **Suggestions** — 가능하면 이번 PR에 반영

| 항목 | 근거 |
|------|------|
| 테스트 이름이 `<동작>_<예상_결과>_when_<조건>` 형식이고, `returns`·`fails`·`rejects`·`succeeds` 같은 일관된 동사를 사용하는가? | §3.2 |
| AAA(Arrange / Act / Assert) 구조를 따르는가? Act(동작 실행)가 정확히 한 번인가? | §5.1 |
| `Result` 에러 검증 시 `unwrap_err()` 단독 사용에 그치지 않고 `assert!(matches!(err, DomainError::InvalidInput(_)))`처럼 에러 타입까지 검증하는가? | §5.4 |
| 타임스탬프·자동 생성 UUID처럼 비결정적 필드를 제외하고, 검증 가능한 결정적 필드만 비교하는가? | §5.3 |
| `#[tokio::test]`·`#[sqlx::test(fixtures(...))]`·`axum::test + tower::ServiceExt`를 용도에 맞게 사용하는가? | §7.1, §7.2, §7.3 |
| 단위 70% / 통합 20% / E2E 10% 피라미드 비율에 근접하는가? 특정 레이어에 편중되지 않는가? | §6.1 |
| 순수 CRUD·프레임워크 배선(axum 라우팅, DI)·정적 상수·삭제 예정 코드에 테스트를 작성하는가? ("이 테스트가 보호하는 동작을 한 문장으로 설명할 수 없으면 작성하지 말 것") | §12 |

📝 **Tech Debt** — 향후 개선 고려

| 항목 | 근거 |
|------|------|
| 같은 함수에 예시 기반 테스트를 4회 이상 작성하려는 시점에서 `proptest`(멱등성·불변식·교환법칙 검증)로 전환을 검토했는가? | §9.1, §9.2 |
| 여러 테스트 파일에서 반복되는 setup 코드가 `tests/common/mod.rs` 또는 `tests/helpers.rs`로 분리되어 있는가? | §14.2 |
| 반복되는 DB 시드 데이터를 `tests/fixtures/*.sql`로 분리하여 `#[sqlx::test(fixtures("..."))]`로 재사용하는가? | §11.2 |
| 단위 테스트 1ms·통합 테스트 100~300ms 기준을 크게 초과하는 테스트가 있는가? (CI 전체 5분 목표) | §14.3 |

**9.2 패턴** — `rust-test-style.md` 핵심 설계 사상

- **Classicist TDD** (§1.3) — 상태 검증 우선. `mockall`은 이메일 발송·이벤트 발행처럼 side-effect가 비즈니스 요구사항일 때만. AI 리팩토링 시 Mockist 테스트는 깨지지만 Classicist는 사양을 만족하면 통과
- **시스템 경계 모킹** (§1.2 / §4.1) — "내가 통제·소유하는 것은 실제 구현 사용, 외부 경계(결제 API·OAuth·외부 HTTP)만 `wiremock`으로 대체." DB·Repository는 절대 Mock 금지
- **FIDT 원칙** (§1.5) — Fast(단위 1ms)·Isolated(공유 전역 상태 없음)·Deterministic(`Clock`·난수 주입으로 비결정성 제거)·Trustworthy(실패 시 실제 버그 재현). 하나라도 무너지면 스위트 전체 신뢰도 저하
- **동작 기반 네이밍** (§1.1 / §3.1) — `<동작>_<예상_결과>_when_<조건>`. 함수명·호출 순서가 아닌 관찰 가능한 행동을 설명. 순수 리팩토링 후에도 테스트 이름이 유효해야 함
- **in-memory Fake vs `#[sqlx::test]`** (§4.3 / §4.4) — 도메인 순수 로직 → `in-memory Fake`로 DB 없이 단위 테스트. 유스케이스·레포지토리 → `#[sqlx::test]`로 실제 DB + 자동 트랜잭션 롤백
- **AAA + 단일 Act** (§5.1) — Arrange → Act(정확히 1회) → Assert. Act가 두 번 나타나면 테스트 분리 신호. 각 테스트는 관찰 가능한 결과 하나만 검증
- **Builder 패턴** (§11.1) — 설정·모킹 코드가 검증 코드보다 훨씬 많을 때 도입. 테스트 의도가 데이터 구성 코드에 묻히지 않도록

### 10. 보안 `R-09`

**10.1 체크** — `rust-security-style.md` §1~§12 기반. OWASP Top 10 / STRIDE 위협 모델 적용.

🚫 **Blocking** — 즉시 차단, 보안 사고 직결

| 항목 | OWASP / STRIDE |
|------|----------------|
| 소스코드에 JWT_SECRET·API 키·DB URL 등 시크릿 리터럴이 하드코딩되어 있는가? (`Config::from_env()` + 환경 변수 로드 필수) | A02·A03 / Spoofing |
| `format!()` 또는 문자열 연결로 SQL을 동적으로 조합하는가? (`sqlx` 파라미터 바인딩·`query!` 매크로 전용) | A05 인젝션 / Tampering |
| 리소스 접근 시 `find_by_id_and_owner` 패턴 없이 ID만으로 조회하여 타인 리소스 열람이 가능한가? (접근 거부는 403이 아닌 404로 통일하여 리소스 존재 여부를 숨겨야 함) | A01 접근 제어 우회 / Elevation of Privilege |
| 라이브러리·핸들러 코드에서 `unwrap()`/`expect()`로 패닉을 유발하는가? (`expect()`는 컴파일 타임 유효성이 보장된 리터럴에만 허용하며 이유 주석 필수) | A10 예외 처리 취약점 / Denial of Service |
| JWT 검증에서 알고리즘을 명시하지 않거나 (`Validation::new(Algorithm::HS256)` 필수), `validate_exp`·`validate_nbf`가 `true`로 설정되지 않아 만료·미래 토큰을 허용하는가? | A07 인증 실패 / Spoofing |
| `unsafe` 블록에 `// SAFETY:` 주석 없이 포인터 유효성·비중첩 보장 등 안전 불변식을 증명하지 않는가? | (메모리 안전) / Tampering |

⚠️ **Recommended** — 머지 전 필수 수정

| 항목 | OWASP / STRIDE |
|------|----------------|
| HTTP 파라미터·헤더·파일·메시지 큐 등 외부 입력이 `req.validate()`로 Allowlist 기반 검증되는가? ("내부 요청"이라는 이유로 검증을 생략하는가?) | A02 보안 설정 오류 / Tampering |
| 요청 구조체에 `#[serde(deny_unknown_fields)]`가 없거나, `role`·`is_admin`·`permissions` 같은 민감 필드가 포함되어 공격자의 권한 상승에 악용될 수 있는가? | A08 무결성 실패 / Elevation of Privilege |
| 서로 다른 도메인 식별자(`UserId`, `OrderId`)가 동일한 원시 타입(`i64`)으로 선언되어 파라미터 혼동이 컴파일 타임에 차단되지 않는가? (Newtype + `#[sqlx(transparent)]`) | A01 접근 제어 우회 / Elevation of Privilege |
| 패스워드 해싱에 MD5·SHA1·SHA256·저비용 bcrypt 등 취약한 알고리즘을 사용하는가? (`argon2` 크레이트의 `Argon2::default()` — Argon2id 기본값 — 필수) | A04 암호화 결함 / Information Disclosure |
| `IntoResponse` 구현에서 `e.to_string()`, DB 에러 원인, 파일 경로, 스택 트레이스를 클라이언트에 직접 반환하는가? (`thiserror`로 에러 타입 정의, `IntoResponse`에서 제네릭 외부 메시지만 반환) | A10 예외 처리 취약점 / Information Disclosure |
| `tracing` 로그에 패스워드·JWT 토큰·API 키·신용카드번호·주민등록번호가 구조화 필드나 포맷 문자열로 기록되는가? | A09 로깅·경보 실패 / Information Disclosure |
| 사용자 입력 URL로 외부 요청을 전달하는 기능에 `ALLOWED_HOSTS` 목록 검증과 내부 네트워크 주소(169.254.x.x·10.x.x.x·172.16.x.x 등) 차단이 없는가? | A10 예외 처리 취약점 / Elevation of Privilege |

💡 **Suggestions** — 가능하면 이번 PR에 반영

| 항목 | OWASP / STRIDE |
|------|----------------|
| API 키·세션 토큰·HMAC 등 비밀값 비교에 `==` 연산자를 사용하여 실행 시간 차이로 값이 추론 가능한가? (`subtle::ConstantTimeEq` 사용 필수) | A07 인증 실패 / Spoofing |
| 패스워드·API 키 등 민감한 값이 `Zeroizing<T>`로 래핑되지 않아 함수 종료 후에도 메모리에 남아 있는가? | A04 암호화 결함 / Information Disclosure |
| 인증·로그인·비밀번호 재설정 엔드포인트에 Rate Limiting이 없어 브루트 포스 공격이 가능한가? | A07 인증 실패 / Denial of Service |
| 인증 성공·실패, 권한 거부, 민감 리소스 접근 이벤트가 `tracing`으로 Who(user_id)·What(action)·When(타임스탬프)·Result(success/forbidden)를 포함하여 구조화 기록되는가? | A09 로깅·경보 실패 / Repudiation |
| 새 API에 대해 "비정상 호출 순서", "권한 없는 사용자의 ID 추측", "입력값 의도적 변형" 시나리오를 설계 단계에서 검토했는가? (할인 중복 적용·결제 흐름 우회 같은 비즈니스 로직 취약점은 SAST 도구로 탐지 불가) | A06 안전하지 않은 설계 / Tampering·Elevation |

📝 **Tech Debt** — 향후 개선 고려

| 항목 | 근거 |
|------|------|
| `src/lib.rs`에 `#![warn(clippy::unwrap_used)]`·`#![warn(clippy::expect_used)]`·`#![warn(clippy::panic)]`·`#![warn(clippy::integer_arithmetic)]`가 설정되어 있는가? | §8.3 보안 Clippy lint |
| `cargo audit --deny warnings`가 CI에 포함되어 있는가? Cargo.toml 의존성에 `default-features = false`를 적용하여 불필요한 기능을 비활성화하는가? | §8.1 정기 감사, §8.2 CI 게이트 |
| `gitleaks detect --source . --log-opts="HEAD"`로 Git 이력 전체의 시크릿 누출을 스캔하는가? | §7.2 시크릿 감사 |
| C 라이브러리 FFI에서 내부 `unsafe`를 공개 API에 직접 노출하지 않고 안전한 Rust 래퍼로 감싸며 `// SAFETY:` 주석을 포함하는가? | §6.2 FFI 경계 |

**10.2 패턴** — `rust-security-style.md` 핵심 설계 사상

- **악용 명세 우선** (§1.1) — "어떻게 동작해야 하는가"가 아닌 "어떻게 악용될 수 있는가"를 먼저 설계. 비즈니스 로직 취약점은 SAST·DAST 도구로 탐지 불가, 오직 설계 단계에서만 제거 가능
- **Find-by-Owner** (§3.2) — `find_by_id_and_owner()`: 소유권 검증과 DB 조회를 단일 연산으로 결합. 접근 거부는 반드시 404로 통일하여 리소스 존재 여부 자체를 숨김
- **Allowlist Validation** (§2.1) — Blocklist(차단 목록)이 아닌 Allowlist(허용 목록) 기반 입력 검증. "내부 요청"이라는 이유만으로 검증 생략 금지
- **Assume Breach + Defense in Depth** (§1.3) — 침해를 전제로 Blast Radius 최소화. 입력 검증 → 인가 → 에러 처리 → 감사 로그, 각 계층이 독립적으로 방어
- **Zeroize on Drop** (§4.4) — `Zeroizing<T>`: 민감 값이 Drop될 때 메모리를 0으로 덮어써 메모리 덤프·스왑 파일 공격 방어
- **STRIDE 체크포인트** (§1.2) — 기능 추가·변경마다 Data Flow와 Trust Boundary를 Spoofing·Tampering·Repudiation·Information Disclosure·DoS·Elevation 6가지 관점으로 재검토
- **thiserror + IntoResponse 분리** (§5.1) — `thiserror`로 내부 에러 타입 정의, `IntoResponse`에서 제네릭 외부 메시지만 반환. DB 에러 원인·테이블명이 클라이언트에 노출되지 않도록
- **상수 시간 비교** (§4.2) — `subtle::ConstantTimeEq`: 비밀값 비교 시 실행 시간 차이로 값을 추론하는 타이밍 채널 공격 방어

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
| R-04 에러 처리 — unwrap→? 변환 | ✅ 단순 패턴 | 로직 변경 없이 기계적 변환 가능 |
| R-05 소유권 — clone 제거·참조 변환 | ✅ 참조 변환 | &str/&[T]로 서명 변경, 컴파일로 검증 |
| R-06 제어 흐름 — 수동 루프→Iterator | ✅ 기계적 변환 | 결과 동일, 컴파일로 검증 |
| 공통 매직 넘버→const 추출 | ✅ 명명 추출 | 의미 변경 없는 리팩토링 |
| fmt/clippy 위반 | ✅ 도구 자동화 | cargo fmt/clippy --fix |

Claude가 **자동 수정하지 않고 코멘트만** 남기는 이슈:

| 카테고리 | 수동 처리 이유 |
|----------|---------------|
| R-01 도메인 모델 | API 시그니처 변경 → 영향 범위 큼 |
| R-02 상태 & 모델링 | 상태 전이 의미 파악 필요 |
| R-03 경계 조건 | 비즈니스 의도 파악 필요 |
| R-07 추상화 & trait | 설계 의도 판단 필요 |
| R-09 보안 (unsafe 포함) | 안전 불변식 인간 검토 필수 |

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
