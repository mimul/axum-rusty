---
name: code-review-feedback
description:
  /code-review-feedback 커맨드로 실행되는 Rust 코드 리뷰 피드백 스킬.
  rust-coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로 소스 코드를 직접 수정하지 않고 GitHub PR에 리뷰 코멘트를 게시한다.
  PR 모드에서는 gh api로 인라인 코멘트·리뷰 본문을 PR에 직접 등록하고, 비PR 모드에서는 현재 브랜치의 PR을 자동 감지하거나 Markdown 리포트를 출력한다.
---

# `/code-review-feedback` 커맨드 스킬

coding-style.md를 판단 기준으로 소스 코드를 절대 수정하지 않고 코드 리뷰에 대한 분석 결과를 GitHub PR 리뷰 코멘트로 게시한다.

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

`--with-security` 전제 조건: [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다.

---

# STEP 0 사전 조건 체크

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

## PR 모드

```bash
# MCP로 PR 메타정보 조회
gh pr view {PR번호} --json title,body,baseRefName,headRefName

# PR 브랜치 체크아웃
git fetch origin
git checkout {headBranch}

# 변경 파일 추출
git diff origin/{baseBranch}...HEAD --name-only   # 변경 파일 목록
git diff origin/{baseBranch}...HEAD               # 전체 diff
```

## 로컬 변경 모드 (기본값)

```bash
git diff HEAD --name-only   # 변경 파일 목록 (staged + unstaged)
git diff HEAD               # 전체 diff
```

## staged 모드

```bash
git diff --cached --name-only   # staged 변경 파일 목록
git diff --cached               # staged 전체 diff
```

수집 후 확인:
- [ ] 변경 파일 목록 확보
- [ ] 각 파일 전체 내용 Read로 로드 완료

# STEP 2 coding-style.md 기준 분석

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
    → 근거   : coding-style.md §[섹션번호] [섹션명]
    → 설명   : 무엇이 문제인가, 어떤 위험(보안·버그·아키텍처 위반)이 있는가
    → 영향   : API 영향, transaction 영향, concurrency 영향 등 구체적으로 기술

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
  [설계 방향, 잠재 리스크, coding-style.md 분석후 관점 개선 제안 3~5줄]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## STEP 3 — 리뷰 코멘트 초안 생성

```
3-A: 코멘트 요약 목록 출력    ← 전체 내용을 한눈에 파악
3-B: 코멘트 상세 초안 출력    ← 실제 게시될 내용 확인
3-C: 명시적 승인 게이트       ← 여기서 멈추고 응답 대기
```

### 3-A. 코멘트 요약 목록 출력

분석 결과를 코멘트로 변환하기 전에 먼저 게시 예정 목록을 표로 보여준다.

**코멘트 형식 선택 기준:**

| 조건 | 코멘트 형식 |
|------|------------|
| 특정 파일·행을 특정할 수 있음 | 인라인 코멘트 |
| 전반적인 설계 문제 / 복수 파일 | 전체 리뷰 본문 |
| 파일은 특정되나 행 특정 불가 | 파일 수준 코멘트 |

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

### 3-B. 코멘트 상세 초안 출력

요약 목록 직후, 실제 GitHub에 게시될 내용을 번호 순서대로 출력한다.

**인라인 코멘트 초안 형식 (번호별):**

```
─── #1  인라인 코멘트 ─────────────────────
📍 위치:   src/handler.rs : 42행
🏷️ 분류:   🚫 Blocking  [R-04]
📐 근거:   coding-style.md 경계조건(unwrap 금지) 처리 방식

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

> 리뷰 기준: coding-style.md (도메인 중심 진화형 코딩 원칙)

### 🚫 Blocking Issues
- `[파일:행]` [이슈 제목]: [설명] / 📐 rust-coding-style.md §[섹션]

### ⚠️ Recommended Changes
- `[파일:행]` [이슈 제목]: [설명] / 📐 rust-coding-style.md §[섹션]

### 💡 Suggestions
- ...

### ✅ 잘 된 부분
- [긍정적인 점 1~3가지]

───────────────────────────────────────
```

### 3-C. 명시적 승인 게이트

**상세 초안 출력 후 반드시 이 프롬프트를 출력하고 응답을 기다린다.**
**사용자가 응답하기 전까지 gh api 호출을 포함한 어떤 액션도 실행하지 않는다.**

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
| `"인라인만"` | STEP 4 실행 (인라인만, 전체 본문 제외) |
| `"전체만"` | STEP 4 실행 (전체 본문만, 인라인 제외) |
| `"수정해줘: 2 [요청]"` | #2 코멘트 재작성 → 3-B 해당 항목만 재출력 → 3-C 재표시 |
| `"취소"` / `"stop"` / `"ㄴ"` | 게시 없이 종료. Markdown 리포트 출력 여부 확인 |

**`--dry-run` 플래그**: 3-C에서 승인을 받더라도 STEP 4를 건너뛰고 종료한다.

---

## STEP 4 — GitHub PR에 리뷰 코멘트 게시

**STEP 3-C에서 사용자가 "게시" / "ok" / "ㅇ" / "인라인만" / "전체만" 중 하나로 승인한 경우에만 이 단계를 실행한다.**
**`--dry-run`이면 승인을 받더라도 이 단계를 건너뛰고 초안 출력으로 종료한다.**

### 4-A. PR 존재 여부 확인 및 분기

**PR 없는 모드**에서 PR이 감지되지 않은 경우 즉시 4-D로 이동한다.
**PR 있는 경우** 4-B로 진행한다.

### 4-B. head SHA 확인 (PR 있는 경우)

인라인 코멘트 게시에 `commit_id`(PR head SHA)가 필요하다:

```bash
HEAD_SHA=$(gh api repos/{owner}/{repo}/pulls/{번호} --jq '.head.sha')
```

### 4-C. 인라인 코멘트 게시

각 인라인 이슈에 대해 `/reviews` API의 `comments` 배열로 한 번에 게시한다
(API 호출 횟수 최소화):

```bash
gh api repos/{owner}/{repo}/pulls/{번호}/reviews \
  --method POST \
  -f body="[전체 리뷰 본문]" \
  -f event="[COMMENT|REQUEST_CHANGES|APPROVE]" \
  -f "comments[][path]"="[파일경로]" \
  -F "comments[][line]"=[행번호] \
  -f "comments[][side]"="RIGHT" \
  -f "comments[][body]"="[인라인 코멘트 본문]"
  # 인라인 이슈 수만큼 comments[] 반복
```

| 파라미터 값 | API 값 | 의미 |
|------------|--------|------|
| `comment` (기본값) | `COMMENT` | 일반 코멘트 (승인·거부 없음) |
| `request-changes` | `REQUEST_CHANGES` | 변경 요청 (머지 블록) |
| `approve` | `APPROVE` | 승인 (이슈 없을 때만 사용) |

**`approve` 사용 조건**: 🚫 Blocking (Critical + High) 이슈가 0건인 경우에만 허용.
🚫 Blocking (Critical / High)이 있으면 자동으로 `COMMENT`로 강등하고 경고를 출력한다.

### 4-D. 게시 결과 확인

```bash
# 게시된 리뷰 확인
gh api repos/{owner}/{repo}/pulls/{번호}/reviews \
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

### 4-E. PR 미감지 시 Markdown 리포트 출력 (4-A에서 분기)

PR이 없는 경우 아래 형식으로 클립보드 복사 가능한 리포트를 출력한다:

````markdown
## 코드 리뷰 리포트 — [날짜]

**대상**: [파일 / 브랜치 / 커밋]
**기준**: coding-style.md (도메인 중심 진화형 코딩 원칙)

### 발견된 이슈 ([N]건)

#### 🚫 Blocking Issues
- `[파일명:행번호]` **[R-XX]** [이슈 제목]
  - [설명] / 📐 rust-coding-style.md §[섹션]
  - ```rust
    // Before
    [현재 코드]
    // After
    [개선 코드]
    ```

#### ⚠️ Recommended Changes
...

#### 💡 Suggestions
...

### 종합 평가
[3~5줄]
````

---

## 주의사항

- **소스 파일을 절대 수정하지 않는다.** Edit / Write 도구를 사용하지 않는다.
- 인라인 코멘트의 `line` 번호는 diff 내 변경된 행이어야 한다 (변경되지 않은 행에 게시 불가).
- `commit_id`는 반드시 PR head의 최신 SHA를 사용한다 (오래된 SHA 사용 시 API 오류).
- `gh auth status`에서 `repo` 스코프가 있어야 리뷰 게시가 가능하다.
