---
name: code-review-feedback
description:
  /code-review-feedback 커맨드로 실행되는 Rust 코드 리뷰 피드백 스킬.
  coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로, security-style.md(소스 레벨 보안 체크리스트)를 2차 판단 기준으로 소스 코드를 직접 수정하지 않고 GitHub PR에 리뷰 코멘트를 게시한다.
  PR 모드에서는 gh api로 인라인 코멘트·리뷰 본문을 PR에 직접 등록하고, 비PR 모드에서는 현재 브랜치의 PR을 자동 감지하거나 Markdown 리포트를 출력한다.
---

# `/code-review-feedback` 커맨드 스킬

coding-style.md와 security-style.md를 판단 기준으로 소스 코드를 절대 수정하지 않고 코드 리뷰에 대한 분석 결과를 GitHub PR 리뷰 코멘트로 게시한다.

**`/code-review`와의 차이점:**

| 항목 | /code-review | /code-review-feedback |
|------|-------------------|---------------------------|
| 목적 | 코드 분석 + 직접 수정 | 코드 분석 + PR 코멘트 게시 |
| 소스 수정 | Before/After 제시 후 적용 | **절대 수정하지 않음** |
| 결과물 | 수정된 코드 + 커밋 | GitHub PR 리뷰 코멘트 |
| 주 사용자 | 코드 작성자 | 리뷰어 역할을 하는 사람 |

## 커맨드 문법

```
# PR 리뷰 피드백
/code-review-feedback --pr 42                   PR #42 분석 후 GitHub 리뷰 게시
/code-review-feedback --pr 42 --dry-run         GitHub 게시 없이 코멘트 초안만 출력

# 로컬 변경사항 리뷰 피드백
/code-review-feedback                           staged + unstaged 변경사항 전체 리뷰 (기본값) 후 GitHub 리뷰 게시
/code-review-feedback --dry-run                 GitHub 게시 없이 staged + unstaged 변경사항 전체 리뷰 코멘트 초안만 출력
/code-review-feedback --staged                  staged(git add된) 변경사항만 리뷰 후 GitHub 리뷰 게시

```

필요 시에만 아래 옵션을 추가한다.

```
--with-tests      리뷰 후 /test-align 명령으로 테스트 갭 분석 및 보완 수행
--with-security   리뷰 후 /security-full-scan + /security-scan 보안 스캔 수행
```

- `--with-tests ` 명령은 `! ls ~/.claude/skills/`` 또는 `! ls .claude/skills/`를 입력해 `/test-align` 명령이 있는지 확인해 없으면 사용자에게 확인한다.
- `--with-security` 전제 조건: [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다. `! ls ~/.claude/skills/`` 또는 `! ls .claude/skills/`를 입력해 `/security-full-scan`, `/security-scan`이 있는지 확인해 없으면 사용자에게 확인한다.

---

## STEP 0 사전 조건 체크

리뷰 시작 전 아래를 순서대로 확인한다. 하나라도 실패하면 사용자에게 보고하고 중단한다.

```bash
git fetch origin && git status && git log --oneline -5 && git log --oneline origin/main -5   # 최신 브랜치 확인
cargo build                 # 빌드 통과 여부 확인
cargo test --all            # 리뷰 전 테스트 baseline 확보
```

확인 항목:
- [ ] 최신 브랜치 확인
- [ ] `cargo build` 통과
- [ ] `cargo test --all` 통과

---

## STEP 1 — 리뷰 대상 코드 접수

모드별로 아래 git 커맨드로 변경 코드를 수집한다.

### PR 모드

```bash
# PR 메타정보 조회
gh pr view {PR번호} --json title,body,baseRefName,headRefName

# PR 브랜치 체크아웃
git fetch origin
git checkout {headBranch}

# 변경 파일 추출
git diff origin/{baseBranch}...HEAD --name-only   # 변경 파일 목록
git diff origin/{baseBranch}...HEAD               # 전체 diff
```

### 로컬 변경 모드 (기본값)

```bash
git diff HEAD --name-only   # 변경 파일 목록 (staged + unstaged)
git diff HEAD               # 전체 diff

# 현재 브랜치에 연결된 PR 자동 감지
gh pr view --json number,url,baseRefName,headRefName 2>/dev/null
```

### staged 모드

```bash
git diff --cached --name-only   # staged 변경 파일 목록
git diff --cached               # staged 전체 diff

# 현재 브랜치에 연결된 PR 자동 감지
gh pr view --json number,url,baseRefName,headRefName 2>/dev/null
```

수집 후 확인:
- [ ] 변경 파일 목록 확보
- [ ] 각 파일 전체 내용 Read로 로드 완료
- [ ] PR 존재 여부 확인 완료 (로컬·staged 모드)

---

## STEP 2 — coding-style.md · security-style.md 기준 분석

> **역할**: 내부 분석용 리포트. 이 결과를 바탕으로 STEP 3에서 GitHub 게시용 코멘트 초안을 생성한다.

## 2-1 .claude/rules/coding-style.md 기준 분석

`.claude/rules/coding-style.md` 19개 섹션의 요약 체크리스트를 변경 코드에 순서대로 적용한다. 섹션별로 위반 항목을 수집하고 분류(🚫/⚠️/💡/📝)를 결정한다.

| 섹션 | 분석 관점 |
|------|-----------|
| §1 Domain First | DB concern이 domain으로 새지 않는가? Newtype/Enum 사용 여부. explicit conversion 존재 여부 |
| §2 Architecture First | 레이어 간 의존 방향이 올바른가? domain이 framework를 참조하지 않는가? workspace boundary 유지 여부 |
| §3 Explicit & Intentional | 데이터 흐름이 명확한가? hidden mutation이 없는가? `?` 연산자로 에러를 명시적으로 전파하는가? |
| §4 Readability | nesting depth, 함수 크기(<50줄), 명명 규칙(`<동사>_<대상>`), handle/process/run 금지 접두사 사용 여부 |
| §5 Complexity Control | 불필요한 abstraction, premature generics, 모듈 응집도 여부 |
| §6 Changeability | stable boundary 유지 여부, framework coupling 여부, concern separation 여부 |
| §7 Consistency | 네이밍·모듈 구조·에러 전략의 일관성, validation·auth·logging 패턴 일관성 |
| §8 Usecase Oriented | controller가 얇은가? usecase가 workflow와 transaction boundary를 소유하는가? usecase 단일 의도 여부 |
| §9 Dependency Injection | constructor injection 사용 여부, trait boundary 의존 여부, composition root 존재 여부 |
| §10 Error Handling | unwrap/expect 남용 여부, 에러 경계(layer별 타입 변환) 존재 여부, `.ok()` 남용 여부 |
| §11 Type-Driven Design | invalid state 허용 여부, 식별자→Newtype/상태값→Enum 적용 여부, generic complexity 통제 여부 |
| §12 Async & Concurrency | Arc 사용 여부, `std::sync::Mutex` vs `tokio::sync::Mutex` 구분, mutable state 최소화 여부 |
| §13 Database & Repository | SQL이 infra에만 있는가? repository interface가 domain에 정의되어 있는가? transaction boundary 위치 |
| §14 API Design | DTO 분리 여부, boundary validation 수행 여부, domain model 직접 노출 여부 |
| §15 Documentation | `pub fn`/`pub struct`/`pub trait`에 `///` 주석 여부, `unsafe` 블록에 `// SAFETY:` 주석 여부 |
| §16 Authentication | 인증 middleware 분리 여부, auth duplication 없는가? 명시적 보안 실패 처리 여부 |
| §17 Observability | tracing 필드 기반 로깅(`key = value` 형식) 여부, 에러 삼킴 여부, context 포함 여부 |
| §18 Testing | business behavior 검증 여부, implementation coupling 최소화 여부, 테스트 커버리지 80% 이상 여부 |
| §19 AI Alignment | 아키텍처 일관성, predictable 구조 유지 여부, naming·folder 구조 일관성 |

## 2-2 .claude/rules/security-style.md 기반 Security 점검

`.claude/rules/security-style.md` 15개 섹션 중 소스 코드 변경에 해당하는 항목을 점검한다. 발견된 보안 이슈는 심각도와 무관하게 🚫 Blocking으로 분류한다.

| 섹션 | 주요 체크 |
|------|-----------|
| §1 Authentication | JWT signature·expiration 검증 존재, credential 하드코딩 없음, 로그에 token 미출력, logout 후 세션 무효화 |
| §2 Authorization | 인가 검증 누락 API 없음, IDOR 없음, 수평·수직 권한 상승 불가, multi-tenant 격리 |
| §3 Input Validation | Prepared Statement 사용, OS command injection 없음, HTML escaping 적용, 신뢰되지 않은 역직렬화 없음 |
| §4 File Handling | 파일 확장자·MIME 검증, path traversal 차단, canonical path 검증, 실행 가능 파일 업로드 차단 |
| §5 API Security | 인증 없는 엔드포인트 없음, excessive data exposure 없음, rate limiting 존재 |
| §6 Cryptography & Secrets | API key·password 하드코딩 없음, 약한 난수·ECB mode 미사용, TLS 검증 비활성화 없음 |
| §7 Logging | 비밀번호·token·개인정보 로그 미출력, log forging 불가, structured logging 적용 |
| §8 Error Handling | stack trace·내부 경로·SQL 오류 외부 미노출, deny-default, 예외 시 보안 우회 불가 |
| §11 Concurrency | 대용량 payload 제한, ReDoS 패턴 없음, connection·fd leak 없음 |
| §12 Business Logic | 상태 전이 검증 유지, race condition 방지, replay attack 대응 |
| §14 Secure Coding | dynamic code execution 없음, unsafe native call 없음, trust boundary 명확 |
| §15 Rust 특화 | unsafe 블록 SAFETY 주석, panic 기반 DoS 없음, serde 입력 검증 존재 |

> **보안 체크**: security-style.md 이슈는 심각도 등급(🚫 Critical / ⚠️ High / ⚡ Medium / ℹ️ Low)에 상관없이 리뷰에서 항상 **🚫 Blocking**으로 분류한다.

## 2-3 .claude/rules/test-style.md 기반 테스트 품질 점검

`.claude/rules/test-style.md` 기준으로 변경된 코드의 테스트 품질을 점검한다. 이 섹션은 코드 수정 없이 PR 코멘트로 피드백을 출력하는 **읽기 전용** 단계다.

| 섹션 | 점검 내용 | 판정 기준 |
|------|-----------|-----------|
| §2 Behavior First | implementation coupling 없이 observable behavior 검증 여부 | 상호작용만 검증 시 🚫 Blocking |
| §3 모킹 경계 | Repository Mock 사용 여부, 프로세스 경계 외부 API 처리 방식 | MockRepository 사용 시 ⚠️ Recommended |
| §4 Classicist TDD | state verification 사용 여부, 실제 domain object 사용 여부 | interaction-only 검증 시 ⚠️ Recommended |
| §7 Readability | AAA 구조 준수, `<행동>_<기대결과>_when_<조건>` 네이밍 패턴 | generic 네이밍 시 💡 Suggestion |
| §8 Flaky & Deterministic | 이슈 링크·담당자·기한 없는 `#[ignore]`, 타이밍 의존 패턴 | 이유 없는 ignore 시 🚫 Blocking |
| §15 파일 구조 | 단위 테스트 위치(`src/` 내), 통합·DB 테스트 위치(`tests/`) | 피라미드 역전 시 ⚠️ Recommended |
| §16 불필요 테스트 | 로직 없는 CRUD·프레임워크 배선·getter 단독 테스트 존재 여부 | 삭제 권장 시 💡 Suggestion |
| §17.1 PR 거절 신호 | 5개 Blocking 패턴(Assertion 없음, Mock chain, Mock DB 등) 탐지 | 해당 시 🚫 Blocking |

---

### §2 Behavior First — 탐지 방법

**Implementation Coupling 탐지**

```rust
// 🚫 Blocking — 상호작용만 검증, 결과 상태 없음
mock_repo.expect_save().times(1).returning(|_| Ok(()));
// assert가 전혀 없음

// ✅ 정상 — observable behavior 검증
let result = usecase.create(command).await.unwrap();
assert_eq!(result.status, TodoStatus::Todo);
```

PR 코멘트 예시:
```
🚫 **[Blocking] Behavior First 위반 — 상태 검증 누락**

현재 테스트는 `save()` 호출 횟수만 검증하고, 결과 상태를 확인하지 않습니다.
test-style.md §2.1은 mock 상호작용이 아닌 observable behavior 검증을 요구합니다.

**현재 코드**
```rust
mock_repo.expect_save().times(1).returning(|_| Ok(()));
// assert 없음
```

**개선 방향**
```rust
let created = usecase.create(command).await.unwrap();
assert_eq!(created.title, "write guide");
assert_eq!(created.status, TodoStatus::Todo);
```
```

---

### §3 모킹 경계 — 경계별 판정

| 경계 유형 | 탐지 패턴 | 판정 | PR 코멘트 안내 |
|-----------|-----------|------|----------------|
| Repository Mock | `MockTodoRepository`, `mock_repo.expect_find` | ⚠️ Recommended | in-memory Fake 또는 `sqlx::test` 전환 권장 |
| 외부 HTTP API Mock | `reqwest::Client` 직접 Mock | ⚠️ Recommended | `wiremock-rs`로 HTTP 레벨 Fake 전환 권장 |
| Domain Object Mock | `MockTodo`, `MockOrder` 등 | 🚫 Blocking | 자체 소유 도메인 객체는 절대 Mock 금지 |
| 순수 함수 Mock | 유틸·변환 함수 Mock | 🚫 Blocking | 순수 함수는 Mock 대상이 아님 |
| 외부 결제·OAuth | HTTP 직접 의존 | ⚠️ Recommended | `wiremock-rs` 사용 권장 |

PR 코멘트 예시 (Repository Mock):
```
⚠️ **[Recommended] MockRepository 사용 — Fake 또는 실제 DB 전환 권장**

test-style.md §3.2는 Repository에 Mock 대신 in-memory Fake 또는
`sqlx::test`를 사용할 것을 요구합니다. Mock은 실제 쿼리 동작을 검증하지 못합니다.

**전환 방법**
```rust
// in-memory Fake
let repo = InMemoryTodoRepository::new();

// 또는 실제 DB (권장)
#[sqlx::test]
async fn create_todo_persists(pool: PgPool) {
    let repo = PostgresTodoRepository::new(pool);
    // ...
}
```
```

---

### §4 Classicist TDD — 체크리스트

탐지 기준:

- `expect_xxx().times(n)` 호출이 `assert_eq!` / `assert!` 보다 많으면 → ⚠️ Recommended
- `unwrap()` 이후 assertion 없으면 → ⚠️ Recommended
- 도메인 객체를 `Mock*` 으로 대체하면 → 🚫 Blocking

PR 코멘트 예시:
```
⚠️ **[Recommended] Classicist TDD — state verification 누락**

`expect_save().times(1)` 외에 결과 상태를 검증하는 assert가 없습니다.
test-style.md §4.1은 상호작용 검증보다 상태 검증을 우선합니다.

**현재 코드**
```rust
mock_repo.expect_save().times(1).returning(|_| Ok(()));
usecase.create(cmd).await.unwrap();
// assert 없음
```

**개선 방향**: `create()` 반환값 또는 저장된 엔티티의 필드를 `assert_eq!`로 검증하세요.
```

---

### §7 Readability — AAA 구조 및 네이밍 패턴

**AAA 구조 탐지**

```rust
// 💡 Suggestion — AAA 구분 없음
#[test]
fn test_complete() {
    let mut t = Todo::new(TodoTitle::new("x".to_string()).unwrap());
    t.start().unwrap(); t.complete().unwrap();
    assert!(t.complete().is_err());
}

// ✅ 정상 — AAA 명확
#[test]
fn complete_todo_returns_error_when_already_done() {
    // Arrange
    let mut todo = Todo::new(TodoTitle::new("x".to_string()).unwrap());
    todo.start().unwrap();
    todo.complete().unwrap();

    // Act
    let result = todo.complete();

    // Assert
    assert!(result.is_err());
}
```

**네이밍 패턴 탐지**

| 나쁜 패턴 (탐지) | 판정 | 권장 패턴 |
|-----------------|------|-----------|
| `test_complete()` | 💡 Suggestion | `complete_todo_returns_error_when_already_done()` |
| `test_save()` | 💡 Suggestion | `save_todo_persists_title_and_status()` |
| `handle_auth()` | 💡 Suggestion | `returns_401_when_auth_header_is_missing()` |
| `process_request()` | 💡 Suggestion | `create_todo_returns_201_when_valid_input()` |

PR 코멘트 예시:
```
💡 **[Suggestion] 테스트 네이밍 — behavior 기반 패턴 권장**

`test_complete`는 구현 구조를 그대로 반영한 이름입니다.
test-style.md §7.3은 `<행동>_<기대결과>_when_<조건>` 패턴을 권장합니다.

**예시**
- `test_complete` → `complete_todo_returns_error_when_already_done`
- `test_save` → `save_todo_persists_when_title_is_valid`
```

---

### §8 Flaky & Deterministic — 탐지 패턴

| 탐지 패턴 | 판정 | PR 코멘트 안내 |
|-----------|------|----------------|
| `#[ignore]` 이유·이슈 링크 없음 | 🚫 Blocking | 이슈 링크·담당자·기한 추가 필수 |
| `SystemTime::now()` 테스트 내 직접 사용 | ⚠️ Recommended | `Clock` 인터페이스 주입으로 교체 |
| `tokio::time::sleep` 타이밍 조정 | ⚠️ Recommended | 채널 동기화 또는 상태 기반 대기로 교체 |
| `rand::random()` 시드 없이 사용 | ⚠️ Recommended | 결정적 시드 사용 또는 인터페이스 주입 |
| 공유 전역 상태(`static`) 테스트 간 공유 | ⚠️ Recommended | 테스트마다 독립 인스턴스 생성 |

Quarantine 기준 — PR 코멘트 예시:
```
🚫 **[Blocking] Flaky 테스트 격리 규칙 위반**

`#[ignore]` 에 이슈 링크·담당자·기한이 없습니다.
test-style.md §8.4는 아래 형식을 요구합니다.

**현재 코드**
```rust
#[ignore]
#[tokio::test]
async fn some_flaky_test() { }
```

**필수 형식**
```rust
#[ignore = "Flaky: race condition in async setup. Issue: #123, Owner: @mimul, Due: 2024-03-01"]
#[tokio::test]
async fn some_flaky_test() { }
```
```

---

### §15 파일 구조 — 테스트 위치 점검

| 테스트 종류 | 올바른 위치 | 잘못된 위치 | 판정 |
|-------------|------------|------------|------|
| 단위 테스트 | `src/` 내 `#[cfg(test)]` | `tests/` 최상위 | ⚠️ Recommended |
| DB/HTTP 통합 테스트 | `{crate}/tests/` | `src/` 내 `#[cfg(test)]` | ⚠️ Recommended |
| 통합 테스트가 전체의 70%+ | — | — | ⚠️ Recommended (단위 테스트 보강 권장) |

PR 코멘트 예시:
```
⚠️ **[Recommended] 테스트 파일 구조 — 위치 불일치**

DB 쿼리를 포함한 통합 테스트가 `src/` 내 `#[cfg(test)]`에 위치합니다.
test-style.md §15.2는 DB/HTTP 테스트를 `{crate}/tests/` 하위에 배치할 것을 권장합니다.

**권장 구조**
```
infra/
  src/repository/user.rs          # 구현체
  tests/user_repository_test.rs  # DB 통합 테스트
```
```

---

### §16 불필요 테스트 — 삭제 권장 기준

삭제를 권장하는 경우:

- 로직 없는 순수 CRUD (E2E 1개로 충분)
- axum 라우팅·shaku DI 배선 검증
- 타입 시스템이 보장하는 정적 상수
- 단순 getter/setter

PR 코멘트 예시:
```
💡 **[Suggestion] 불필요 테스트 — 삭제 검토 권장**

이 테스트는 axum 라우팅 등록만 검증하며, 비즈니스 동작이 없습니다.
test-style.md §16은 프레임워크 배선 테스트를 불필요 테스트로 분류합니다.

삭제하거나, HTTP contract를 검증하는 통합 테스트로 대체하는 것을 권장합니다.
```

---

### §17.1 PR 거절 신호 (Blocking Issues)

아래 5개 패턴을 탐지하면 **모두 🚫 Blocking**으로 분류한다.

| 패턴 | 탐지 조건 | PR 코멘트 안내 |
|------|-----------|----------------|
| Assertion 없는 테스트 | `#[test]` 함수 내 `assert` 계열 없음 | assert 추가 필수 |
| 의미 없는 Assertion | `assert!(result.is_some())` 단독 | 구체적 필드 값 검증으로 교체 |
| 통합 테스트에서 Mock DB | `MockRepository` in `tests/` | in-memory Fake 또는 `sqlx::test` 전환 |
| 내부 구현 직접 접근 | `pub(super)` 로 테스트 외부에서 내부 접근 | public API를 통한 black-box 테스트로 전환 |
| `#[ignore]` 이유 없음 | annotation만, 이슈 링크 없음 | 이슈 링크·담당자·기한 추가 |

PR 코멘트 예시 (Assertion 없음):
```
🚫 **[Blocking] Assertion 없는 테스트**

테스트 함수에 `assert` 계열 구문이 없습니다.
test-style.md §17.1은 이를 즉시 반려 사유로 명시합니다.

검증할 상태나 결과가 없다면 해당 테스트 자체를 삭제하거나,
실제 business behavior를 검증하는 assert를 추가하세요.
```

분석 리포트는 아래와 같다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /code-review-feedback 분석 리포트
    [PR 피드백 모드: PR #[번호] — [PR 제목]]
    [로컬 변경 모드:  브랜치 [브랜치명] — staged + unstaged]
    [staged 모드:   브랜치 [브랜치명] — staged only]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 리뷰 범위
   - 파일:  [리뷰한 .rs 파일 목록]
   - 구성:  [주요 fn/struct/trait 목록]
   - async: [있음 / 없음]  |  unsafe: [있음 / 없음]  |  테스트: [있음 / 없음]
   - 도메인 가시성: [높음 / 중간 / 낮음 — 도메인 개념이 타입·이름에 드러나는 정도]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 발견된 이슈 ([N]건)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking Issues ([N]건)
  • [파일명:행번호] 이슈 제목
    → 근거   : coding-style.md §[섹션번호] [섹션명]  또는  security-style.md §[섹션번호] [섹션명]
    → 설명   : 무엇이 문제인가, 어떤 위험(보안·버그·아키텍처 위반)이 있는가
    → 영향   : API 영향, transaction 영향, concurrency 영향, 보안 영향 등 구체적으로 기술

⚠️ Recommended Changes ([N]건)
  • ...

💡 Suggestions ([N]건)
  • ...

📝 Tech Debt ([N]건)
  • ...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 이상 없는 카테고리
  • [섹션명] — 문제 없음

📝 종합 평가
  [설계 방향, 잠재 리스크, coding-style.md·security-style.md 분석 후 관점 개선 제안 3~5줄]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 리뷰 코멘트 초안 생성

> **역할**: STEP 2 분석 결과를 GitHub 게시용 코멘트로 변환한다. 실제 게시 전 사용자 확인을 받는 단계다.

```
3-1: 코멘트 요약 목록 출력    ← 전체 내용을 한눈에 파악
3-2: 코멘트 상세 초안 출력    ← 실제 게시될 내용 확인
3-3: 명시적 승인 게이트       ← 여기서 멈추고 응답 대기
```

### 3-1. 코멘트 요약 목록 출력

분석 결과를 코멘트로 변환하기 전에 먼저 게시 예정 목록을 표로 보여준다.

**코멘트 형식 선택 기준:**

| 조건 | 코멘트 형식 |
|------|------------|
| 특정 파일·행을 특정할 수 있음 | 인라인 코멘트 |
| 전반적인 설계 문제 / 복수 파일 | 전체 리뷰 본문 |
| 파일은 특정되나 행 특정 불가 | 파일 수준 코멘트 |

**`event` 타입 자동 결정 기준:**

| 조건 | event 값 |
|------|----------|
| 🚫 Blocking 1건 이상 | `REQUEST_CHANGES` |
| 🔒 Security Issues 1건 이상 (보안 이슈는 항상 Blocking) | `REQUEST_CHANGES` |
| 🚫/🔒 없고 ⚠️ / 💡 / 📝 만 있음 | `COMMENT` |
| 이슈 0건 | `APPROVE` |

**요약 목록 출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  게시 예정 코멘트 목록 — 총 [N]건
    대상: PR #[번호] / 리뷰 타입: [COMMENT|REQUEST_CHANGES|APPROVE]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  #   종류      위치                  분류              제목
  ─── ──────── ──────────────────── ──────────────── ────────────────────────
  1   인라인   src/handler.rs:42     🚫 Blocking       unwrap() 사용
  2   인라인   src/handler.rs:87     🚫 Blocking       std::Mutex in async
  3   인라인   src/repo.rs:15        ⚠️ Recommended    clone() 불필요
  4   전체    (리뷰 본문)              💡 Suggestion     문서화 누락

  인라인 [N]건 + 전체 리뷰 본문 [있음/없음]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
아래에 각 코멘트의 상세 초안을 보여드립니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 3-2. 코멘트 상세 초안 출력

요약 목록 직후, 실제 GitHub에 게시될 내용을 번호 순서대로 출력한다.

**인라인 코멘트 초안 형식 (번호별):**

```
─── #1  인라인 코멘트 ─────────────────────
📍 위치:   src/handler.rs : 42행
🏷️ 분류:   🚫 Blocking  [R-04]
📐 근거:   coding-style.md §10 Error Handling (unwrap 금지)
          또는 security-style.md §6 Cryptography & Secrets (하드코딩 API key)

🚫 Blocking | src/handler.rs:42
**unwrap() 사용으로 패닉 위험이 있습니다.**

[문제 설명 2~3줄. 왜 문제인지, 어떤 위험/영향이 있는지.]

```rust
// Before
[현재 문제 코드]

// After
[개선 예시 코드]
```

```
───────────────────────────────────────

─── #2  인라인 코멘트 ─────────────────────
...
───────────────────────────────────────

─── 전체 리뷰 본문 ────────────────────────

## 코드 리뷰 — PR #[번호]

> 리뷰 기준: coding-style.md (도메인 중심 진화형 코딩 원칙) · security-style.md (소스 레벨 보안 체크리스트)

### 🚫 Blocking Issues
- `[파일:행]` [이슈 제목]: [설명] / 📐 coding-style.md §[섹션]  또는  security-style.md §[섹션]

### 🔒 Security Issues
- `[파일:행]` [보안 이슈 제목]: [설명] / 📐 security-style.md §[섹션]

### ⚠️ Recommended Changes
- `[파일:행]` [이슈 제목]: [설명] / 📐 coding-style.md §[섹션]

### 💡 Suggestions
- ...

### ✅ 잘 된 부분
- [긍정적인 점 1~3가지]

───────────────────────────────────────
```

> **보안 체크**: 보안 이슈는 반드시 🔒 Security Issues 섹션에 별도로 나열한다. security-style.md §1/§2/§3/§6 (인증·인가·인젝션·시크릿)은 우선 검토한다.

### 3-3. 명시적 승인 게이트

**상세 초안 출력 후 반드시 이 프롬프트를 출력하고 응답을 기다린다.**
**사용자가 응답하기 전까지 gh api 호출을 포함한 어떤 액션도 실행하지 않는다.**
**`--dry-run` 플래그가 활성화된 경우, 초안 확인 완료 메시지를 출력하고 이 게이트를 건너뛴다 (게시하지 않음).**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚦  PR #[번호]에 위 [N]건의 리뷰를 게시합니다.
    게시 후에는 GitHub에서 리뷰어가 확인할 수 있습니다.

    응답해 주세요:

   ✅ "게시" / "ok" / "ㅇ"        → 전체 게시 (STEP 4 실행)
   🔇 "인라인만"                   → 인라인 [N]건만 게시
   🔇 "전체만"                     → 전체 리뷰 본문만 게시
   ✏️  "수정해줘: [번호] [요청]"      → 해당 번호 코멘트 재작성 후 재확인
   ❌ "취소" / "stop"              → 게시하지 않고 종료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**응답별 처리:**

| 응답 | Claude 행동 |
|------|-------------|
| `"게시"` / `"ok"` / `"ㅇ"` | STEP 4 실행 (전체 게시) |
| `"인라인만"` | STEP 4 실행 (인라인만, 전체 본문 body를 빈 문자열로 설정) |
| `"전체만"` | STEP 4 실행 (전체 본문만, comments 배열을 비워서 게시) |
| `"수정해줘: 2 [요청]"` | #2 코멘트 재작성 → 3-2 해당 항목만 재출력 → 3-3 재표시 |
| `"취소"` / `"stop"` / `"ㄴ"` | 게시 없이 종료. Markdown 리포트 출력 여부 확인 |

---

## STEP 4 — GitHub PR에 리뷰 코멘트 게시

**STEP 3-3에서 사용자가 "게시" / "ok" / "ㅇ" / "인라인만" / "전체만" 중 하나로 승인한 경우에만 이 단계를 실행한다.**
**`--dry-run`이면 STEP 3-3을 건너뛰었으므로 이 단계는 실행되지 않는다.**

### 4-1. PR 존재 여부 확인 및 분기

STEP 1에서 감지된 PR 정보를 참조한다.

**PR이 없는 경우** 즉시 4-5로 이동한다.
**PR이 있는 경우** 4-2로 진행한다.

### 4-2. owner/repo 및 head SHA 확인

인라인 코멘트 게시에 `commit_id`(PR head SHA)와 `{owner}/{repo}` 정보가 필요하다:

```bash
# 리포지토리 정보 확인
REPO=$(gh repo view --json owner,name --jq '"\(.owner.login)/\(.name)"')

# PR head SHA 확인
HEAD_SHA=$(gh api repos/${REPO}/pulls/{번호} --jq '.head.sha')
```

### 4-3. 리뷰 게시 (인라인 + 전체 본문)

인라인 코멘트와 전체 리뷰 본문을 `/reviews` API로 한 번에 게시한다 (API 호출 횟수 최소화).

**"인라인만" 승인 시**: `body`를 빈 문자열(`""`)로 설정한다.
**"전체만" 승인 시**: `comments` 배열을 비운다(`[]`).

복수의 인라인 코멘트를 정확히 전달하기 위해 `--input`으로 JSON을 직접 전달한다:

```bash
gh api repos/${REPO}/pulls/{번호}/reviews \
  --method POST \
  --input - <<'EOF'
{
  "body": "[전체 리뷰 본문. '인라인만' 승인 시 빈 문자열]",
  "event": "[COMMENT|REQUEST_CHANGES|APPROVE]",
  "comments": [
    {
      "path": "src/handler.rs",
      "line": 42,
      "side": "RIGHT",
      "body": "[인라인 코멘트 본문 #1]"
    },
    {
      "path": "src/handler.rs",
      "line": 87,
      "side": "RIGHT",
      "body": "[인라인 코멘트 본문 #2]"
    }
  ]
}
EOF
```

`event` 값은 3-1에서 결정된 값을 그대로 사용한다:

| 조건 | event 값 |
|------|----------|
| 🚫 Blocking 1건 이상 | `REQUEST_CHANGES` |
| 🔒 Security Issues 1건 이상 (보안 이슈는 항상 Blocking) | `REQUEST_CHANGES` |
| 🚫/🔒 없고 ⚠️ / 💡 / 📝 만 있음 | `COMMENT` |
| 이슈 0건 | `APPROVE` |

> **보안 체크**: 보안 이슈가 포함된 경우 인라인 코멘트에 `security-style.md §섹션` 근거를 명시하고, 전체 리뷰 본문의 `🔒 Security Issues` 섹션에도 중복 기재한다. `cargo audit` 결과도 함께 첨부할 것.

### 4-4. 게시 결과 확인

```bash
# 게시된 리뷰 확인
gh api repos/${REPO}/pulls/{번호}/reviews \
  --jq '.[-1] | {id, state, submitted_at, body: .body[:100]}'
```

**게시 성공 시:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  GitHub PR #[번호]에 리뷰 게시 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
리뷰 타입:  [COMMENT / REQUEST_CHANGES / APPROVE]
인라인:     [N]건 게시
전체 본문:  [있음 / 없음]
URL:      [PR URL]#pullrequestreview-[리뷰ID]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**게시 실패 시:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  게시 실패: [오류 메시지]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
확인 사항:
  gh auth status            인증 상태 확인
  gh auth scope             repo 스코프 필요
  행번호가 diff 범위 내인지 확인
  --dry-run 으로 초안 재확인
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-5. PR 미감지 시 Markdown 리포트 출력 (4-1에서 분기)

PR이 없는 경우 아래 형식으로 클립보드 복사 가능한 리포트를 출력한다:

````markdown
## 코드 리뷰 리포트 — [날짜]

**대상**: [파일 / 브랜치 / 커밋]
**기준**: coding-style.md (도메인 중심 진화형 코딩 원칙) · security-style.md (소스 레벨 보안 체크리스트)

### 발견된 이슈 ([N]건)

#### 🚫 Blocking Issues
- `[파일명:행번호]` **[R-XX]** [이슈 제목]
  - [설명] / 📐 coding-style.md §[섹션]  또는  security-style.md §[섹션]
  - ```rust
    // Before
    [현재 코드]
    // After
    [개선 코드]
    ```

#### 🔒 Security Issues
- `[파일명:행번호]` [보안 이슈 제목]
  - [설명] / 📐 security-style.md §[섹션]

#### ⚠️ Recommended Changes
...

#### 💡 Suggestions
...

### 종합 평가
[3~5줄]
````

---

## STEP 5 — 선택적 추가 검사 (옵션 지정 시에만 실행)

STEP 4 완료 후, 커맨드 옵션에 따라 아래를 실행한다.

### `--with-tests` 지정 시

```bash
/test-align   # 테스트 갭 분석 및 보완 수행
```

### `--with-security` 지정 시

```bash
/security-full-scan {리뷰 대상 경로}   # 전체 소스 + 의존성 CVE + 시크릿 전수 감사
/security-scan staging                 # 스테이징 환경 런타임 보안 감사
```

`--with-security` 실행 전 [claude-security-scan](https://github.com/mimul/claude-security-scan) 설치 여부를 확인하고, 미설치 시 사용자에게 안내 후 건너뛴다.

---

## 주의사항

- **소스 파일을 절대 수정하지 않는다.** Edit / Write 도구를 사용하지 않는다.
- 인라인 코멘트의 `line` 번호는 diff 내 변경된 행이어야 한다 (변경되지 않은 행에 게시 불가).
- `commit_id`는 반드시 PR head의 최신 SHA를 사용한다 (오래된 SHA 사용 시 API 오류).
- `gh auth status`에서 `repo` 스코프가 있어야 리뷰 게시가 가능하다.
- 복수의 인라인 코멘트는 반드시 `--input` + JSON heredoc 방식으로 게시한다 (`-f` 반복은 배열 요소를 올바르게 전달하지 못한다).
