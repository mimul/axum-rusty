---
name: code-review-feedback-rust
description: >
  /code-review-feedback-rust 커맨드로 실행되는 Rust 코드 리뷰 피드백 스킬.
  /code-review-rust와 동일한 분석(10개 카테고리)을 수행하지만, 소스 코드를 직접 수정하지 않고 GitHub PR에 리뷰 코멘트를 게시한다.
  PR 모드에서는 gh api로 인라인 코멘트·리뷰 본문을 PR에 직접 등록하고, 비PR 모드에서는 현재 브랜치의 PR을 자동 감지하거나 Markdown 리포트를 출력한다.
  모든 파라미터는 /code-review-rust와 동일하게 지원한다.
---

## 스킬 개요

이 스킬은 **`/code-review-feedback-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`/code-review-rust`와 동일한 10개 카테고리 분석을 수행하되,
**소스 코드를 절대 수정하지 않고** 분석 결과를 GitHub PR 리뷰 코멘트로 게시한다.

**`/code-review-rust`와의 차이점:**

| 항목 | /code-review-rust | /code-review-feedback-rust |
|------|-------------------|---------------------------|
| 목적 | 코드 분석 + 직접 수정 | 코드 분석 + PR 코멘트 게시 |
| 소스 수정 | Before/After 제시 후 적용 | **절대 수정하지 않음** |
| 결과물 | 수정된 코드 + 커밋 | GitHub PR 리뷰 코멘트 |
| 주 사용자 | 코드 작성자 | 리뷰어 역할을 하는 사람 |

리뷰의 핵심 불변 조건:
- **코드 무수정** — 어떤 상황에서도 소스 파일을 변경하지 않는다
- **변경분만 분석** — `git diff`로 실제 변경 파일만 정확히 추출
- **인라인 코멘트 우선** — 가능하면 해당 파일·행에 직접 인라인으로 게시
- **게시 전 확인** — 코멘트 초안을 먼저 보여주고 인간 승인 후에만 게시
- **dry-run 지원** — `--dry-run` 옵션으로 게시 없이 초안만 확인

---

## 커맨드 문법

```
# PR 피드백
/code-review-feedback-rust --pr 42                   PR #42 분석 후 GitHub 리뷰 게시
/code-review-feedback-rust --pr 42 --scope error     에러 처리 카테고리만 분석 후 게시
/code-review-feedback-rust --pr 42 --event request-changes  변경 요청 리뷰로 게시
/code-review-feedback-rust --pr 42 --dry-run         게시 없이 코멘트 초안만 출력

# 로컬 변경사항 분석 (현재 브랜치의 PR 자동 감지)
/code-review-feedback-rust                           staged + unstaged 분석 (기본값)
/code-review-feedback-rust --staged                  staged 변경사항만 분석
/code-review-feedback-rust --commit a1b2c3d          특정 커밋 분석
/code-review-feedback-rust --branch                  현재 브랜치 vs main 분석
/code-review-feedback-rust --branch feature/payment  지정 브랜치 vs main 분석
/code-review-feedback-rust src/order/handler.rs      특정 파일 분석

# 필터 옵션 (모든 모드와 조합 가능)
/code-review-feedback-rust --scope error             C-CR-01 에러 처리만
/code-review-feedback-rust --scope ownership         C-CR-02 소유권·차용만
/code-review-feedback-rust --scope async             C-CR-06 비동기만
/code-review-feedback-rust --scope unsafe            C-CR-07 unsafe만
/code-review-feedback-rust --scope test              C-CR-10 테스트만
/code-review-feedback-rust --scope security          security.md 기준 집중 분석
/code-review-feedback-rust --severity blocking       🚫 Blocking 이슈만
/code-review-feedback-rust --severity recommended    ⚠️ Recommended 이상

# 리뷰 이벤트 타입 (--pr 모드 전용)
/code-review-feedback-rust --pr 42 --event comment          일반 코멘트 (기본값)
/code-review-feedback-rust --pr 42 --event request-changes  변경 요청
/code-review-feedback-rust --pr 42 --event approve          승인 (이슈 없을 때)

# 정보
/code-review-feedback-rust --catalog                카테고리 목록 출력
/code-review-feedback-rust --help                   사용법 출력
```

---

## 실행 모드 판별

```
--pr [번호] 있음            → [PR 피드백 모드]
                               분석 후 gh api로 GitHub PR에 직접 리뷰 게시
                               STEP 0-PR → STEP 1 → STEP 2 → STEP 3 → STEP 4

인수 없음                   → [로컬 변경 모드]  ← 기본값
                               git diff HEAD 자동 감지 → PR 자동 탐색
                               STEP 0-DIFF → STEP 1 → STEP 2 → STEP 3 → STEP 4

--staged 있음               → [staged 모드]
                               git diff --cached 수집 → PR 자동 탐색
                               STEP 0-STAGED → STEP 1 → STEP 2 → STEP 3 → STEP 4

--commit [해시] 있음        → [커밋 모드]
                               git show [해시] 수집 → PR 자동 탐색
                               STEP 0-COMMIT → STEP 1 → STEP 2 → STEP 3 → STEP 4

--branch (브랜치명 없음)    → [브랜치 모드]
                               현재 브랜치 vs origin/main → PR 자동 탐색
                               STEP 0-BRANCH → STEP 1 → STEP 2 → STEP 3 → STEP 4

--branch [브랜치명] 있음    → [지정 브랜치 모드]
                               지정 브랜치 vs origin/main → PR 자동 탐색
                               STEP 0-BRANCH → STEP 1 → STEP 2 → STEP 3 → STEP 4

파일경로 / 모듈명 있음      → [파일 모드]
                               Read로 직접 읽기 → PR 자동 탐색
                               STEP 0-FILE → STEP 1 → STEP 2 → STEP 3 → STEP 4
```

STEP 1~4는 모든 모드 공통이다.

**우선순위**: `--pr` > `--staged` > `--commit` > `--branch` > 파일경로 > 인수 없음(기본값)

**`--dry-run`**: 모든 모드에서 사용 가능. STEP 3에서 코멘트 초안만 출력하고 STEP 4(게시)를 건너뛴다.

---

## [PR 피드백 모드] STEP 0-PR — PR 정보 및 head SHA 수집

Claude가 Bash 도구로 아래 커맨드를 직접 실행한다:

```bash
# 1. PR 기본 정보 + head commit SHA (인라인 코멘트에 필요)
gh api repos/{owner}/{repo}/pulls/{번호} \
  --jq '{title: .title, base: .base.ref, head_branch: .head.ref, head_sha: .head.sha, changed_files: .changed_files}'

# 2. 변경 파일 목록
gh api repos/{owner}/{repo}/pulls/{번호}/files \
  --jq '[.[] | select(.filename | endswith(".rs")) | {filename, additions, deletions, patch}]'

# 3. diff 내용 (.rs 파일만)
git fetch origin [PR 브랜치명]
git diff origin/[base]...origin/[head] -- '*.rs'
```

**출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  PR #[번호] 정보
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
제목:      [PR 제목]
base:      [base 브랜치]  ←  head: [PR 브랜치명]
head SHA:  [SHA 7자리]  (인라인 코멘트 연결용)
변경 파일: [N]개 .rs 파일
리뷰 타입: [--event 값 또는 기본값 "comment"]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## [비PR 모드] STEP 0-DIFF / STEP 0-STAGED / STEP 0-COMMIT / STEP 0-BRANCH / STEP 0-FILE

코드 수집은 `/code-review-rust`의 동일 STEP과 완전히 동일하게 수행한다.

수집 후 추가로 **현재 브랜치의 열린 PR을 자동 탐색**한다:

```bash
gh pr list --head $(git branch --show-current) --state open --json number,title,url \
  --jq '.[0] | {number, title, url}'
```

**탐색 결과별 분기:**

| 결과 | 처리 |
|------|------|
| PR 발견 | PR 번호를 내부적으로 확보 → STEP 3에서 해당 PR에 게시 옵션 제공 |
| PR 없음 | Markdown 리포트 출력 전용 모드로 전환 (게시 불가 안내) |

**PR 발견 시 안내:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔗  연결된 PR 감지: PR #[번호] — [제목]
    분석 완료 후 해당 PR에 리뷰를 게시할 수 있습니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**PR 없을 때 안내:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ️  연결된 PR 없음 → Markdown 리포트 출력 전용
    PR에 게시하려면 /code-review-feedback-rust --pr [번호] 로 실행하세요.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 1 — 리뷰 대상 코드 접수

**모든 모드**: STEP 0에서 수집한 diff / 파일 내용을 그대로 사용한다.
사용자에게 코드 붙여넣기를 요청하지 않는다.

코드 수신 후 내부적으로 파악한다:
- 주요 fn / struct / impl / trait 목록
- async 여부 / unsafe 여부 / 테스트 존재 여부 / pub 노출 범위

---

## STEP 2 — rules 로드 및 10개 카테고리 분석

**`/code-review-rust`의 STEP 2와 동일하게 수행한다.**

`security.md`와 `test.md`를 로드하고 C-CR-01~C-CR-10 기준으로 분석한다.
분석 리포트 형식도 동일하되 헤더만 스킬명을 반영한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /code-review-feedback-rust 분석 리포트
    [PR 피드백 모드:  PR #[번호] — [PR 제목]]
    [로컬 변경 모드:  브랜치 [브랜치명] — staged + unstaged]
    [staged 모드:     브랜치 [브랜치명] — staged only]
    [커밋 모드:       커밋 [해시 7자리] — [커밋 메시지 첫 줄]]
    [브랜치 모드:     [브랜치명] vs origin/main]
    [파일 모드:       파일 [파일 경로]]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 리뷰 범위
   - 파일:  [리뷰한 .rs 파일 목록]
   - 구성:  [주요 fn/struct/trait 목록]
   - async: [있음 / 없음]  |  unsafe: [있음 / 없음]  |  테스트: [있음 / 없음]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚨 발견된 이슈 ([N]건)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking Issues
  🔴 Critical ([N]건)
  • [C-CR-XX] [파일명:행번호] [이슈 제목]
    → [설명] / 근거: [security.md §섹션 또는 test.md §섹션]

  🟠 High ([N]건)
  • ...

⚠️ Recommended Changes
  🟡 Medium ([N]건)
  • ...

💡 Suggestions / 📝 Tech Debt
  🔵 Low ([N]건)
  • ...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 이상 없는 카테고리
  • [C-CR-XX] [카테고리명] — 문제 없음

📝 종합 평가
  [설계 방향, 잠재 리스크, 개선 제안 3~5줄]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 리뷰 코멘트 초안 생성 및 승인

**이 스킬의 핵심 단계. 소스 수정은 절대 하지 않는다.**
**사용자의 명시적 승인("게시") 없이는 gh api를 절대 호출하지 않는다.**

이 단계는 3개의 하위 단계로 구성된다:

```
3-A: 코멘트 요약 목록 출력    ← 전체 내용을 한눈에 파악
3-B: 코멘트 상세 초안 출력    ← 실제 게시될 내용 확인
3-C: 명시적 승인 게이트        ← 여기서 멈추고 응답 대기
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
  4   전체     (리뷰 본문)           💡 Suggestion     문서화 누락

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
🏷️  분류:   🚫 Blocking  [C-CR-01]

🚫 Blocking | src/handler.rs:42
**unwrap() 사용으로 패닉 위험이 있습니다.**

[문제 설명 2~3줄. 왜 문제인지, 어떤 위험이 있는지.]

```rust
// Before
[현재 문제 코드]

// After
[개선 예시 코드]
```

📎 참고: `security.md §에러 응답`
───────────────────────────────────────

─── #2  인라인 코멘트 ─────────────────────
...
───────────────────────────────────────

─── 전체 리뷰 본문 ────────────────────────

## 코드 리뷰 — PR #[번호]

### 🚫 Blocking Issues
- **[C-CR-XX]** `[파일:행]` [이슈 제목]: [설명]

### ⚠️ Recommended Changes
- **[C-CR-XX]** `[파일:행]` [이슈 제목]: [설명]

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
   ✏️  "수정해줘: [번호] [요청]"    → 해당 번호 코멘트 재작성 후 재확인
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
**PR이 없는 모드에서 PR 미감지 시 Markdown 리포트 출력으로 대체한다.**

### 4-A. head SHA 확인

인라인 코멘트 게시에 `commit_id`(PR head SHA)가 필요하다:

```bash
HEAD_SHA=$(gh api repos/{owner}/{repo}/pulls/{번호} --jq '.head.sha')
```

### 4-B. 인라인 코멘트 게시

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

**`--event` 파라미터 → GitHub API `event` 매핑:**

| 파라미터 값 | API 값 | 의미 |
|------------|--------|------|
| `comment` (기본값) | `COMMENT` | 일반 코멘트 (승인·거부 없음) |
| `request-changes` | `REQUEST_CHANGES` | 변경 요청 (머지 블록) |
| `approve` | `APPROVE` | 승인 (이슈 없을 때만 사용) |

**`approve` 사용 조건**: 🚫 Blocking (Critical + High) 이슈가 0건인 경우에만 허용.
🚫 Blocking (Critical / High)이 있으면 자동으로 `COMMENT`로 강등하고 경고를 출력한다.

### 4-C. 게시 결과 확인

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
URL:        [PR URL]#pullrequestreview-[리뷰ID]
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

### 4-D. PR 미감지 시 Markdown 리포트 출력

PR이 없는 경우 아래 형식으로 클립보드 복사 가능한 리포트를 출력한다:

````markdown
## 코드 리뷰 리포트 — [날짜]

**대상**: [파일 / 브랜치 / 커밋]
**분석 기준**: CODE_REVIEW_RUST.md C-CR-01~C-CR-10

### 발견된 이슈 ([N]건)

#### 🚫 Blocking Issues
- `[파일명:행번호]` **[C-CR-XX]** [이슈 제목]
  - [설명]
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

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `../code-review-rust/CODE_REVIEW_RUST.md` | C-CR-01~10 체크리스트 | 스킬 실행 시 항상 |
| `../../rules/security.md` | 보안 규칙 | **STEP 2 분석 시작 전 로드** |
| `../../rules/test.md` | 테스트 규칙 | **STEP 2 분석 시작 전 로드** |

---

## 주의사항

- **소스 파일을 절대 수정하지 않는다.** Edit / Write 도구를 사용하지 않는다.
- `--event approve`는 🚫 Blocking (Critical + High) 이슈가 0건일 때만 허용한다.
- 인라인 코멘트의 `line` 번호는 diff 내 변경된 행이어야 한다 (변경되지 않은 행에 게시 불가).
- `commit_id`는 반드시 PR head의 최신 SHA를 사용한다 (오래된 SHA 사용 시 API 오류).
- `gh auth status`에서 `repo` 스코프가 있어야 리뷰 게시가 가능하다.
