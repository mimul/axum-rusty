---
name: code-review-rust
description: >
  /code-review-rust 커맨드로 실행되는 Rust 코드 리뷰 스킬.
  coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로,
  CODE_REVIEW_RUST.md의 9개 카테고리(R-01~R-09)를 통해 Rust 코드를 분석한다.
  일곱 가지 모드를 지원한다:
    PR 모드          — MCP로 PR 정보(제목·설명·대상 브랜치) 확인 후 전용 worktree를 생성하고, git diff로 변경 파일만 정확히 추출하여 리뷰한다.
    로컬 변경 모드     — 인수 없이 실행 시 기본값. git diff HEAD로 staged + unstaged 변경사항을 자동 감지하여 즉시 리뷰한다.
    staged 모드      — --staged 옵션. git diff --cached로 staged 변경사항만 리뷰한다.
    커밋 모드         — --commit <hash>. 특정 커밋 단독 변경사항을 리뷰한다.
    브랜치 모드       — --branch. 현재 브랜치와 기본 브랜치(main)의 diff를 리뷰한다.
    지정 브랜치 모드   — --branch <name>. 지정 브랜치와 기본 브랜치(main)의 diff를 리뷰한다.
    로컬 파일 모드     — 파일 경로 또는 모듈명 지정 시 해당 파일을 직접 리뷰한다.
  이슈마다 coding-style.md §섹션 근거와 Before/After를 제시하고 인간 확인 후에만 수정을 적용한다.
---

# `/code-review-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/code-review-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`CODE_REVIEW_RUST.md`의 9개 카테고리(R-01~R-09)를 분석 구조로 삼되,
**`coding-style.md`를 1차 판단 기준**으로, `rust-security-style.md`·`rust-test-style.md`를 보완 기준으로 적용한다.
STEP 2 분석 시작 전 세 파일을 반드시 로드하고, 이슈마다 어느 `coding-style.md §섹션`을 근거로 판단했는지 명시한다.

리뷰의 핵심 불변 조건:
- **PR 기반 격리** — PR 리뷰는 전용 worktree에서 수행, main 브랜치 보호
- **변경분만 리뷰** — `git diff`로 실제 변경 파일만 정확히 추출
- **coding-style.md 1차 적용** — R-01~R-09 전체 판단의 근거는 coding-style.md §섹션
- **rust-security-style.md 보완 적용** — R-04·R-05·R-09 판단 시 보안 규칙을 추가 기준으로 적용
- **rust-test-style.md 보완 적용** — R-08 판단 시 테스트 규칙의 커버리지·네이밍 기준 적용
- **보여주고 확인받기** — Before/After 제시 → 인간 승인 후에만 수정 적용
- **분류 우선** — 🚫 Blocking → ⚠️ Recommended → 💡 Suggestions 순서로 처리
- **항상 그린** — 수정 후 `cargo test` + `cargo clippy` 통과 확인

---

## 커맨드 문법

```
# PR 리뷰
/code-review-rust --pr 42                   PR #42를 MCP로 조회 후 worktree 리뷰
/code-review-rust --pr 42 --scope error     PR 리뷰 + 에러 처리 집중

# 로컬 변경사항 리뷰
/code-review-rust                           staged + unstaged 변경사항 전체 리뷰 (기본값)
/code-review-rust --staged                  staged(git add된) 변경사항만 리뷰

# 커밋 리뷰
/code-review-rust --commit a1b2c3d          특정 커밋의 변경사항만 리뷰
/code-review-rust --commit HEAD             직전 커밋 리뷰
/code-review-rust --commit HEAD~2           2커밋 전 리뷰

# 브랜치 diff 리뷰
/code-review-rust --branch                  현재 브랜치 vs main 전체 diff 리뷰
/code-review-rust --branch feature/payment  feature/payment 브랜치 vs main diff 리뷰

# 파일/모듈 리뷰
/code-review-rust src/order/handler.rs      특정 파일만 리뷰

# 필터 옵션 (모든 모드와 조합 가능)
/code-review-rust --scope domain            R-01 도메인 모델만
/code-review-rust --scope state             R-02 상태 & 모델링만
/code-review-rust --scope boundary          R-03 경계 조건만
/code-review-rust --scope error             R-04 에러 처리만
/code-review-rust --scope ownership         R-05 소유권 & 메모리만
/code-review-rust --scope control           R-06 제어 흐름만
/code-review-rust --scope abstraction       R-07 추상화 & trait만
/code-review-rust --scope test              R-08 테스트만
/code-review-rust --scope security          R-09 보안 (rust-security-style.md 전체 기준 집중 리뷰)
/code-review-rust --severity blocking       🚫 Blocking 이슈만 보고
/code-review-rust --severity recommended    ⚠️ Recommended 이상 보고

# 정보
/code-review-rust --catalog                 카테고리 목록 출력
/code-review-rust --help                    사용법 출력
```

---

## `--help` 출력 형식

`/code-review-rust --help` 실행 시 아래 형식으로 출력한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📖  /code-review-rust 사용법
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

모드:
  (인수 없음)              로컬 변경 모드 — staged + unstaged 전체 리뷰
  --pr [번호]              PR 모드 — MCP로 PR 조회 후 worktree 리뷰
  --staged                 staged 변경사항만 리뷰
  --commit [해시]          특정 커밋 단독 리뷰 (HEAD, HEAD~N 가능)
  --branch [브랜치명]      지정 브랜치 vs main diff 리뷰
                           (브랜치명 생략 시 현재 브랜치)
  [파일경로]               지정 파일 직접 리뷰

필터 (모든 모드와 조합 가능):
  --scope [카테고리]       특정 카테고리만 분석
    domain | state | boundary | error | ownership
    control | abstraction | test | security
  --severity [분류]        특정 심각도 이상만 보고
    blocking | recommended

정보:
  --catalog               카테고리 목록 (R-01~R-09)
  --help                  이 도움말

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 실행 모드 판별

```
--pr [번호] 있음            → [PR 모드]
                               STEP 0-PR → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

인수 없음                   → [로컬 변경 모드]  ← 기본값
                               git diff HEAD (staged + unstaged) 자동 감지
                               STEP 0-DIFF → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

--staged 있음               → [staged 모드]
                               git diff --cached (staged만) 자동 수집
                               STEP 0-STAGED → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

--commit [해시] 있음        → [커밋 모드]
                               git show [해시]로 해당 커밋 변경사항 수집
                               STEP 0-COMMIT → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

--branch (브랜치명 없음)    → [브랜치 모드]
                               현재 브랜치 vs origin/main diff 수집
                               STEP 0-BRANCH → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

--branch [브랜치명] 있음    → [지정 브랜치 모드]
                               지정 브랜치 vs origin/main diff 수집
                               STEP 0-BRANCH → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

파일경로 / 모듈명 있음      → [로컬 파일 모드]
                               지정된 파일·모듈을 직접 리뷰
                               STEP 0-LOCAL → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5
```

STEP 1~5는 모든 모드 공통이다.

**우선순위**: `--pr` > `--staged` > `--commit` > `--branch` > 파일경로 > 인수 없음(기본값)
**조합 가능 필터**: `--scope`, `--severity`는 위 모든 모드와 함께 사용 가능하다.

---

## [PR 모드] STEP 0-PR — PR 정보 확인 및 Worktree 준비

### 0-PR-1. MCP로 PR 정보 조회

Claude가 MCP GitHub 도구로 직접 아래 항목을 조회한다:
- PR 제목 / 설명(본문) / 작성자
- base 브랜치 / head 브랜치
- 변경 파일 수 및 목록
- 라벨

조회 후 아래 형식으로 출력한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  PR #[번호] 정보
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
제목:   [PR 제목]
작성자: [작성자]
base:   [base 브랜치]  ←  head: [PR 브랜치명]
라벨:   [라벨 또는 없음]

설명: [본문 요약 3~5줄]

변경 파일 ([N]개):
  [.rs 파일 목록 / 기타는 "(기타 N개)"]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 0-PR-2. 리뷰용 Worktree 생성

Claude가 Bash 도구로 아래 커맨드를 **단일 블록**으로 직접 실행한다:

```bash
set -e
WORKTREE_PATH="./review-[PR번호]"
PR_BRANCH="[PR 브랜치명]"
BASE_BRANCH="[base 브랜치]"

# 1. PR 브랜치 가져오기
git fetch origin "$PR_BRANCH"

# 2. worktree 생성 (이미 존재하면 재사용)
if git worktree list | grep -q "$WORKTREE_PATH"; then
  echo "worktree 재사용: $WORKTREE_PATH"
else
  git worktree add "$WORKTREE_PATH" "origin/$PR_BRANCH"
fi

# 3. 빌드 확인
cargo check --manifest-path "$WORKTREE_PATH/Cargo.toml" 2>&1

# 4. 변경 파일 목록 및 diff 추출
git -C "$WORKTREE_PATH" diff --name-only "origin/$BASE_BRANCH...HEAD"
git -C "$WORKTREE_PATH" diff "origin/$BASE_BRANCH...HEAD" -- '*.rs'
```

실행 완료 후 아래 형식으로 보고하고 바로 STEP 1로 진행한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  PR #[번호] Worktree 준비 완료 (자동 실행됨)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

브랜치: [PR 브랜치명]
경로:   ./review-[PR번호]

✅ git fetch      — 완료
✅ git worktree   — 준비 완료
✅ cargo check    — 통과

변경 .rs 파일: [N]개
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 0-PR-3. git diff 결과 해석

```
분류 기준:
  핵심 리뷰 대상 — src/**/*.rs
  테스트 파일   — tests/**/*.rs, *_test.rs, 내부 tests 모듈
  설정 파일     — Cargo.toml, Cargo.lock
  기타          — README, .github/ 등 (코드 리뷰 범위 제외)
```

### 0-PR-4. PR 수정 방식 선택

리뷰 결과 출력 후 수정이 필요한 경우 방식을 제안한다:

**방식 A — 권장**: Claude가 Before/After 제안 → PR 코멘트 초안 생성 → 작성자가 직접 수정
**방식 B**: `fix/cr-pr[번호]-[module]` 브랜치에 직접 수정 후 push
**방식 C**: worktree에서 로컬 브랜치 생성 후 수정 push

---

## [로컬 변경 모드] STEP 0-DIFF — 로컬 변경사항 자동 감지

**인수 없이 `/code-review-rust`만 실행했을 때 이 단계를 따른다.**

Claude가 직접 아래 커맨드를 실행하여 staged + unstaged 변경사항을 수집한다:

```bash
# 1. staged 변경 파일 목록
git diff --cached --name-only -- '*.rs'

# 2. unstaged 변경 파일 목록
git diff --name-only -- '*.rs'

# 3. 실제 diff 내용 (staged + unstaged 통합)
git diff HEAD -- '*.rs'
```

**감지 결과별 분기:**

| 상태 | 처리 |
|------|------|
| staged + unstaged 변경 있음 | 통합 diff를 리뷰 대상으로 즉시 진행 |
| 변경 없음 (clean working tree) | 경고 출력 후 선택 요청 (A: 브랜치 전체 리뷰 / B: 파일 경로 직접 입력) |

**변경사항 감지 시 출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  로컬 변경 감지 (staged + unstaged)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
브랜치:  [현재 브랜치명]
staged:  [staged .rs 파일 목록 또는 "없음"]
unstaged:[unstaged .rs 파일 목록 또는 "없음"]
총 변경: [N]개 파일

→ 위 변경사항을 리뷰합니다. STEP 1로 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**변경 없을 때 출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  로컬 변경사항 없음 (working tree clean)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
A: 브랜치 전체 리뷰 (origin/main...HEAD diff)
B: 파일 경로 직접 입력
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## [staged 모드] STEP 0-STAGED — staged 변경사항 수집

**`--staged` 옵션으로 실행했을 때 이 단계를 따른다.**

Claude가 직접 아래 커맨드를 실행하여 staged 변경사항만 수집한다:

```bash
# 1. staged 변경 파일 목록
git diff --cached --name-only -- '*.rs'

# 2. staged diff 내용
git diff --cached -- '*.rs'
```

**감지 결과별 분기:**

| 상태 | 처리 |
|------|------|
| staged 변경 있음 | staged diff를 리뷰 대상으로 즉시 진행 |
| staged 없음 | 경고 출력 후 종료 (git add 후 재실행 안내) |

**출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🟢  staged 변경 감지
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
브랜치:  [현재 브랜치명]
staged:  [staged .rs 파일 목록]
총 변경: [N]개 파일

→ staged 변경사항을 리뷰합니다. STEP 1로 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**staged 없을 때:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  staged 변경사항 없음
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
git add <파일> 후 다시 실행해 주세요.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## [커밋 모드] STEP 0-COMMIT — 특정 커밋 변경사항 수집

**`--commit <해시>` 옵션으로 실행했을 때 이 단계를 따른다.**

Claude가 직접 아래 커맨드를 실행하여 해당 커밋의 변경사항을 수집한다:

```bash
# 1. 커밋 정보 확인
git show --stat [해시]

# 2. .rs 파일 diff만 추출
git show [해시] -- '*.rs'
```

**특수 해시 처리:**

| 입력 | 해석 |
|------|------|
| `HEAD` | 직전 커밋 |
| `HEAD~N` | N커밋 전 |
| `a1b2c3d` (7자 이상) | 해당 커밋 |

**출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔖  커밋 리뷰
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
커밋:    [해시 7자리] [커밋 메시지 첫 줄]
작성자:  [author]
날짜:    [date]
변경:    [N]개 .rs 파일

→ 해당 커밋 변경사항을 리뷰합니다. STEP 1로 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**해시 오류 시:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  커밋 [해시]를 찾을 수 없습니다
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
git log --oneline -10 으로 최근 커밋을 확인하세요.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## [브랜치 모드 / 지정 브랜치 모드] STEP 0-BRANCH — 브랜치 diff 수집

**`--branch` 또는 `--branch <브랜치명>` 옵션으로 실행했을 때 이 단계를 따른다.**

Claude가 직접 아래 커맨드를 실행하여 브랜치 diff를 수집한다:

```bash
# --branch (브랜치명 없음) → 현재 브랜치 사용
TARGET=$(git branch --show-current)

# --branch <브랜치명> → 지정 브랜치 사용
TARGET=[브랜치명]

# 1. 변경 파일 목록
git diff --name-only origin/main...[TARGET] -- '*.rs'

# 2. diff 내용
git diff origin/main...[TARGET] -- '*.rs'
```

**주의사항:**
- 기본 브랜치가 `main`이 아닌 경우 (`master`, `develop` 등) 자동 감지하여 사용
- 원격 추적 브랜치(`origin/main`)가 없으면 로컬 `main`으로 폴백

**출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  브랜치 diff 리뷰
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
대상 브랜치: [TARGET 브랜치명]
기준 브랜치: origin/main
변경 파일:   [N]개 .rs 파일
커밋 수:     [N]개 커밋 포함

→ 브랜치 diff를 리뷰합니다. STEP 1로 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**diff 없을 때:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  [TARGET] 브랜치와 origin/main 간 .rs 변경사항 없음
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## [로컬 파일 모드] STEP 0-LOCAL — 브랜치 상태 확인

**파일 경로 또는 모듈명을 인수로 지정했을 때 이 단계를 따른다.**

Claude가 Bash 도구로 아래 커맨드를 직접 실행한다:

```bash
git branch --show-current
```

**브랜치별 분기:**

- `feature/*` / `fix/*` → 정상 진행. 바로 STEP 1로 진행한다.
- `main` / `master` → 경고 출력 후 선택 요청 (A: fix/cr 브랜치 생성 / B: 리포트만)
- `fix/cr-*` → 이전 리뷰 수정 작업 중으로 인식하고 이어서 진행한다.

**로컬 모드 수정 커밋 네이밍:**
```
feature/xxx 작업 중  → 같은 브랜치에 커밋
main에서 수정 필요   → fix/cr-{module-name} 생성

커밋 메시지: fix([scope]): [R-XX] [50자 이내 요약]
```

---

## STEP 1 — 리뷰 대상 코드 접수

**PR 모드**: worktree 완료 확인 후, diff 결과 기반으로 리뷰 대상 파일 안내
**로컬 변경 모드**: STEP 0-DIFF에서 수집한 `git diff HEAD -- '*.rs'` 결과를 자동 사용. 사용자에게 코드를 요청하지 않고 바로 분석으로 진행한다.
**staged 모드**: STEP 0-STAGED에서 수집한 `git diff --cached -- '*.rs'` 결과를 자동 사용한다.
**커밋 모드**: STEP 0-COMMIT에서 수집한 `git show [해시] -- '*.rs'` 결과를 자동 사용한다.
**브랜치 모드 / 지정 브랜치 모드**: STEP 0-BRANCH에서 수집한 `git diff origin/main...[TARGET] -- '*.rs'` 결과를 자동 사용한다.
**로컬 파일 모드**: Claude가 Read 도구로 지정된 파일을 직접 읽는다. 사용자에게 코드 붙여넣기를 요청하지 않는다.

```bash
# 인수로 받은 파일 경로를 직접 읽음
Read(file_path: "[지정된 파일 경로]")
```

코드 수신 후 내부적으로 파악한다:
- 주요 fn / struct / impl / trait 목록
- async 여부 / unsafe 여부 / 테스트 존재 여부 / pub 노출 범위

---

## STEP 2 — rules 로드 및 9개 카테고리 분석

**분석 시작 전 아래 세 파일을 반드시 이 순서대로 로드한다:**

1. `.claude/rules/coding-style.md` — **1차 판단 기준** (모든 카테고리의 근거)
2. `.claude/rules/rust-security-style.md` — R-04·R-05·R-09 보완 기준
3. `.claude/rules/rust-test-style.md` — R-08 보완 기준

---

### coding-style.md → R-01~R-09 전체 판단 기준 (1차 기준)

모든 카테고리 판단의 **근본 기준**은 coding-style.md다. 이슈를 보고할 때 반드시 어느 §섹션을 근거로 판단했는지 명시한다.

| 카테고리 | coding-style.md 근거 | 핵심 판단 원칙 |
|----------|----------------------|----------------|
| **R-01** 도메인 모델 | §1.3 도메인 중심 설계, §2.2 도메인 모델, §4 네이밍 규칙 | 도메인 개념이 타입으로 표현되는가? primitive 집착인가? 이름이 도메인 용어인가? 로직이 엔티티 내부에 캡슐화되어 있는가? |
| **R-02** 상태 & 모델링 | §2.2 도메인 모델, §2.4 제어 흐름 | 상태가 enum으로 표현되는가? invalid state가 타입으로 방지되는가? Tell, Don't Ask 원칙이 적용되는가? |
| **R-03** 경계 조건 & 에지 케이스 | §5 에지 케이스 & 경계 조건 전체 | 에지 케이스를 도메인의 일부로 처리하는가? 경계 조건이 코드에 명시적으로 드러나는가? 숨겨진 가정이 없는가? |
| **R-04** 에러 처리 | §5.3 처리 방식, §5.4 금지, §1.2 의도를 드러내는 코드 | 에러가 의도를 드러내는가? 침묵하는 실패가 없는가? 에러 타입이 도메인 의미를 가지는가? |
| **R-05** 소유권 & 메모리 | §1.1 변화 용이성 우선, §2.1 구조 | 변경하기 쉬운 소유권 구조인가? 불필요한 clone()이 없는가? mutable 상태가 최소화되어 있는가? |
| **R-06** 제어 흐름 | §2.4 제어 흐름, §2.1 구조 | 명확한 흐름이 선택되었는가? 중첩 깊이가 2 이하인가? Tell, Don't Ask 원칙을 따르는가? |
| **R-07** 추상화 & trait | §2.3 추상화, §1.1 변화 용이성 우선 | 추상화가 3번 반복(Rule of Three) 이후에 도입되었는가? 테스트 편의만을 위한 추상화가 아닌가? |
| **R-08** 테스트 | §6 테스트 철학 전체 | 테스트가 도메인 행동을 검증하는가? 구현이 아닌 관찰 가능한 동작을 테스트하는가? 경계 조건이 검증되는가? |
| **R-09** 보안 | §7 보안 기본 원칙 전체 | 신뢰하지 않은 입력이 검증되는가? 최소 권한 원칙을 따르는가? 민감 정보가 보호되는가? |

coding-style.md §9 안티 패턴도 횡단적으로 체크한다:
- 성급한 추상화 → R-07 이슈
- 빈약한 도메인 모델 → R-01, R-02 이슈
- 깊은 중첩 구조 → R-06 이슈
- 암묵적 경계 처리 → R-03, R-04 이슈
- 테스트 없는 핵심 로직 → R-08 이슈
- 보안을 고려하지 않은 설계 → R-09 이슈

---

### rust-security-style.md → R-04 · R-05 · R-09 보완 기준

**R-04 에러 처리** 판단 시 추가 적용:

| 조건 | 분류 | 근거 |
|------|------|------|
| `unwrap()`/`expect()` in 라이브러리·핸들러 코드 | 🚫 Blocking (Critical) | §5.3 unwrap 금지 |
| 에러 응답에 DB 쿼리·파일 경로·스택 트레이스 포함 | 🚫 Blocking (High) | §5.1 정보 노출 방지 |
| 로그에 패스워드·토큰·카드 번호 기록 | 🚫 Blocking (High) | §5.2 로그 민감 데이터 |
| 외부 노출 에러 타입에 내부 구현 상세 포함 | ⚠️ Recommended (Medium) | §5.1 에러 처리와 정보 노출 |

**R-05 소유권 & 메모리** 판단 시 추가 적용:

| 조건 | 분류 | 근거 |
|------|------|------|
| `static mut` 사용 | 🚫 Blocking (Critical) | §6 unsafe 코드 (데이터 레이스) |
| async 컨텍스트에서 `std::sync::Mutex` | 🚫 Blocking (High) | §6 (교착 위험) |
| 민감 값이 `Zeroizing<T>` 없이 `Drop` 처리 | ⚠️ Recommended (Medium) | §4.4 zeroize |

**R-09 보안** 판단 시 추가 적용:

🔴 Critical — 즉시 차단:

| 조건 | 근거 |
|------|------|
| 하드코딩된 시크릿(JWT 시크릿·API 키·비밀번호) | §7 시크릿 관리 |
| `unsafe` 블록에 `// SAFETY:` 주석 없음 | §6.1 |
| SQL 쿼리 문자열 포맷 조합 | §3.3 SQL 인젝션 방지 |
| `unwrap()`/`expect()` in 핸들러·라이브러리 | §5.3 (DoS 패닉) |
| JWT `none` 알고리즘 허용 | §4.1 |

🟠 High — 머지 전 필수:

| 조건 | 근거 |
|------|------|
| 외부 입력 미검증 (타입 수준 Newtype + `validator` 없음) | §2 신뢰 경계, §3.1 |
| BOLA: 소유권·권한 검증 없이 리소스 접근 | §3.2 |
| `#[serde(deny_unknown_fields)]` 미사용 + 민감 필드 포함 | §3.4 역직렬화 보안 |
| 에러 응답에 내부 구현 정보 노출 | §5.1 |
| 패스워드 해싱에 MD5/SHA1 사용 | §4.3 |
| SSRF 방지 미적용 (외부 URL 허용 호스트 미검증) | §2.3 |
| FFI `unsafe` 블록에 ABI 미검증 | §6.2 |

🟡 Medium — 가능하면 이번 PR에:

| 조건 | 근거 |
|------|------|
| 비밀값 비교에 일반 `==` 사용 (타이밍 공격) | §4.2 상수 시간 비교 |
| 인증 엔드포인트에 Rate Limiting 없음 | §1.2 DoS 방어 |
| 보안 이벤트 감사 로그 미흡 (Who/What/When/Result 누락) | §9 감사 로그 |

---

### rust-test-style.md → R-08 보완 기준

**R-08 테스트** 판단 시 rust-test-style.md §1~§13 우선순위 기반 추가 적용:

🔴 **Critical** — 즉시 Blocking (§13.1 즉시 반려 기준)

| 확인 항목 | 심각도 매핑 |
|-----------|------------|
| 통합 테스트에서 Mock DB / Mock Repository 사용 | 🚫 Blocking |
| 상호작용 검증만 있고 결과 상태 검증 없음 | 🚫 Blocking |
| `SystemTime::now()` / 시드 없는 난수 등 비결정적 출력 고정 사용 | 🚫 Blocking |
| 이슈 링크·담당자·기한 없이 단순 `#[ignore]` | 🚫 Blocking |
| Assertion 없는 테스트 / 의미 없는 Assertion 단독 사용 | 🚫 Blocking |
| 기존 도구로 충분한데 새 Mock 크레이트 추가 | 🚫 Blocking |

🟠 **High** — 머지 전 필수 수정

| 확인 항목 | 심각도 매핑 |
|-----------|------------|
| 핵심 비즈니스 로직(인증·권한·결제·상태 전환) 테스트 없음 | ⚠️ Recommended |
| `mockall expect` 호출이 실제 assert보다 압도적으로 많음 | ⚠️ Recommended |
| Arrange 코드가 Assert 코드보다 10배 이상 긴 경우 (→ Builder/Fixture 필요) | ⚠️ Recommended |
| `Result` 반환 함수에 에러 케이스 테스트 없음 | ⚠️ Recommended |

🟡 **Medium** — 가능하면 이번 PR에 반영

| 확인 항목 | 심각도 매핑 |
|-----------|------------|
| 테스트 이름이 `<동작>_<예상_결과>_when_<조건>` 템플릿을 따르지 않음 | 💡 Suggestions |
| `#[should_panic]` 사용 (→ `Result` 반환 + `assert!(result.is_err())` 방식 권고) | 💡 Suggestions |
| 단위 70% / 통합 20% / E2E 10% 피라미드 비율 미준수 | 💡 Suggestions |
| 비동기 테스트에 `#[tokio::test]` / `#[sqlx::test]` 미사용 | 💡 Suggestions |

🟢 **Low** — 향후 개선 권고

| 확인 항목 | 심각도 매핑 |
|-----------|------------|
| 동일 함수 예시 테스트 4개 이상 시 proptest 전환 미검토 | 📝 Tech Debt |
| Builder / Fixture 패턴 미도입 (설정 코드 과잉) | 📝 Tech Debt |

---

### 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /code-review-rust 분석 리포트
    [PR 모드:          PR #[번호] — [PR 제목]]
    [로컬 변경 모드:   브랜치 [브랜치명] — staged + unstaged]
    [staged 모드:      브랜치 [브랜치명] — staged only]
    [커밋 모드:        커밋 [해시 7자리] — [커밋 메시지 첫 줄]]
    [브랜치 모드:      [브랜치명] vs origin/main]
    [로컬 파일 모드:   파일 [파일 경로]]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 리뷰 범위
   - 파일:  [리뷰한 .rs 파일 목록]
   - 구성:  [주요 fn/struct/trait 목록]
   - async: [있음 / 없음]  |  unsafe: [있음 / 없음]  |  테스트: [있음 / 없음]
   - 도메인 가시성: [높음 / 중간 / 낮음 — 도메인 개념이 타입·이름에 드러나는 정도]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚨 발견된 이슈 ([N]건)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking Issues ([N]건)
  • [R-XX] [파일명:행번호] [이슈 제목]
    → [설명]
    📐 coding-style.md: §[섹션] [섹션명]
    / 보완 근거: [rust-security-style.md §섹션 또는 rust-test-style.md §섹션] (해당 시)

⚠️ Recommended Changes ([N]건)
  • ...

💡 Suggestions ([N]건)
  • ...

📝 Tech Debt ([N]건)
  • ...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 이상 없는 카테고리
  • [R-XX] [카테고리명] — 문제 없음

📝 종합 평가
  [설계 방향, 잠재 리스크, coding-style.md 철학 관점 개선 제안 3~5줄]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 수정 계획 수립 및 확인

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  수정 계획 — 총 [N]건
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking — 즉시 수정 필수 (Critical + High: [N]건)
  1. [R-XX] [이슈 제목] — [파일명:행번호]

⚠️ Recommended — 권장 수정 (Medium: [N]건)
  2. [R-XX] [이슈 제목] — [파일명:행번호]

💡 Suggestions / 📝 Tech Debt — 선택 사항 (Low: [N]건)
  3. [R-XX] [이슈 제목] — [파일명:행번호]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

| 응답 | Claude 행동 |
|------|-------------|
| `"전체 수정"` | 모든 이슈 순서대로 개별 확인 (🚫 Blocking은 항상 개별 확인) |
| `"Blocking만"` | 🚫 Blocking 이슈만 처리 후 STEP 5 |
| `"Recommended 이상"` | 🚫 + ⚠️ 이슈만 처리 후 STEP 5 |
| `"[번호]번만"` | 지정 이슈만 Before/After 제시 |
| `"리포트만"` | 수정 없이 STEP 5 완료 요약으로 이동 |

---

## STEP 4 — Before/After 비교 제시 → 인간 확인 → 수정 적용

이 단계는 수정할 이슈 수만큼 반복(루프)된다.
Claude는 **절대 먼저 코드를 변경하지 않는다.**

### 4-A. Before/After 비교 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[분류 이모지] [R-XX] [이슈 제목]  —  Before/After 비교
    ([진행 현황: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 위치:   [파일명 : 행번호]
📖 문제:   [위험한 이유 1~2줄]
🏷️  분류:   [🚫 Blocking / ⚠️ Recommended / 💡 Suggestion / 📝 Tech Debt]
📐 근거:   coding-style.md §[섹션번호] [섹션명]
📏 보완:   [해당 시 — rust-security-style.md §섹션 또는 rust-test-style.md §섹션]

─── BEFORE ──────────────────────────────
[문제 코드]

─── AFTER ───────────────────────────────
[수정 코드]

─── 수정 근거 ───────────────────────────
  • [변경 포인트 1]
  • [변경 포인트 2]
  • (보안 관련 시) 🔒 rust-security-style.md §[섹션]: [규칙 설명]
  • (테스트 관련 시) 🧪 rust-test-style.md §[섹션]: [확인 사항]
  • (해당 시) ⚠️  영향 범위: [다른 코드에 미치는 영향]

─── 검증 커맨드 ─────────────────────────
  cargo fmt
  cargo clippy -- -D warnings
  cargo test [관련_테스트_경로]

─── 커밋 메시지 제안 ────────────────────
  fix([scope]): [R-XX] [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 수정을 적용할까요?

   ✅ "적용" / "ok" / "yes"   → 수정 적용 후 다음 이슈
   ❌ "건너뜀" / "skip"        → 건너뛰고 다음으로
   ✏️  "수정해줘: [요청]"       → After 코드 재제안
   ⏸️  "여기서 멈춰"            → 완료 요약으로 이동
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-B. 사용자 응답별 처리

| 응답 | Claude 행동 |
|------|-------------|
| `"적용"` / `"ok"` / `"ㅇ"` | 수정 코드 + 커밋 메시지 → 다음 이슈 |
| `"건너뜀"` / `"skip"` / `"ㄴ"` | 건너뜀 기록 → 다음 이슈 |
| `"수정해줘: [내용]"` | After 재제안 → 재출력 |
| `"왜?"` / `"설명해줘"` | 근거 상세 설명 → 같은 비교 유지 |
| `"여기서 멈춰"` / `"stop"` | 루프 종료 → STEP 5 |
| `"전체 적용"` | 일괄 적용 (**🚫 Blocking + 보안 이슈는 개별 확인 유지**) |

> **주의**: `"전체 적용"` 명령에도 **🚫 Blocking 이슈와 보안 이슈는 반드시 개별 확인**을 받는다.

### 4-C. 수정 적용 시 출력

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  [R-XX] 수정 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

수정된 파일: [파일 경로]
[수정된 최종 코드]

실행 순서:
  1. cargo fmt
  2. cargo clippy -- -D warnings
  3. cargo test [관련_테스트_경로]
  4. git add [파일]
  5. git commit -m "fix([scope]): [R-XX] [요약]"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 5-0 — 커버리지 게이트 (PR 생성 요청 시 필수 통과)

**적용 범위**: PR 모드 또는 사용자가 PR 생성을 요청하는 경우에만 실행한다.
로컬 변경·커밋·브랜치·파일 모드에서 리뷰만 완료하는 경우에는 건너뛴다.

PR 코멘트(5-C) 또는 PR 설명 초안(5-D)을 생성하기 전에 반드시 `cargo tarpaulin`을 실행하고
커버리지가 **80% 이상**인지 확인한다.
**이 단계를 통과하지 못하면 PR을 절대 생성하지 않는다.**

#### 실행 커맨드

```bash
# PR 모드 — worktree 경로 사용
cargo tarpaulin --manifest-path "./review-[PR번호]/Cargo.toml" --out Stdout 2>&1 | tail -5

# 로컬 모드 (staged·커밋·브랜치·파일) — 현재 경로 사용
cargo tarpaulin --out Stdout 2>&1 | tail -5
```

#### 판정 기준

| 결과 | 조건 | 다음 단계 |
|------|------|-----------|
| ✅ 통과 | 커버리지 ≥ 80% | STEP 5 완료 요약 후 5-C 또는 5-D PR 초안 생성 진행 |
| 🚫 차단 | 커버리지 < 80% | PR 생성 금지, 커버리지 갭 리포트 출력 |

#### 통과 시 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  커버리지 게이트 통과
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
→ PR 생성을 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 차단 시 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚫  PR 차단 — 커버리지 기준 미달
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
부족분:        +Y.YY%p 필요

커버리지 낮은 파일:
  • [파일명] — X.X%  (기준 미달)
  • [파일명] — X.X%  (기준 미달)

대응 방법:
  1. /test-rust 스킬로 부족한 파일에 테스트 추가
  2. cargo tarpaulin --out Stdout 재측정
  3. 80% 달성 후 PR 진행

🔒 정책: 커버리지 80% 미만이면 PR을 생성하지 않습니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 5 — 완료 요약 출력

### 5-A. 작업 완료 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉  코드 리뷰 완료 요약
    [PR #[번호] — [PR 제목] / 브랜치 [브랜치명]]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

수정된 이슈 ([N]건):
  ✅ [R-XX] [제목] — [파일명:행번호]

건너뛴 이슈 ([M]건):
  ⏭️  [R-XX] [제목] — [사유]

💡 Suggestions / 📝 Tech Debt:
  🔵 [R-XX] [제목] — [내용]

최종 검증 커맨드:
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test --all
  cargo tarpaulin --out Stdout  # 커버리지 (필수 — STEP 5-0 게이트)

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  □ cargo fmt --check 포맷 위반 없음
  □ 🚫 Blocking 이슈 전부 해결 (Critical + High)
  □ Critical 보안 이슈 전부 해결 (하드코딩 시크릿·SAFETY 주석·SQL 포맷·unwrap·JWT none)
  □ High 보안 이슈 해결 (입력 검증·BOLA·역직렬화·에러 정보 노출·Argon2id·SSRF)
  □ 에러 케이스 테스트 존재 (rust-test-style.md §6. 테스트 피라미드)
  ■ 커버리지 ≥ 80% 확인 완료 (STEP 5-0 통과 필수 — 미달 시 PR 차단)
  □ 공개 API rustdoc 주석 완비
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-B. 커밋 요약 (수정사항 있는 경우)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  커밋 히스토리 요약
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

브랜치: fix/cr-[module-name]
베이스: [원본 브랜치]

[최신]
  xxxxxxx fix([scope]): [R-XX] [요약]
  xxxxxxx fix([scope]): [R-XX] [요약]
[기준] origin/[베이스]

확인 커맨드:
  git log --oneline [베이스]..HEAD
  git diff --stat [베이스]..HEAD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-C. PR 코멘트 초안 (PR 모드)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💬  PR 코멘트 초안 (GitHub에 붙여넣기)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 🤖 Claude 코드 리뷰 결과 (PR #[번호])

> 리뷰 기준: CODE_REVIEW_RUST.md R-01~R-09
> 1차 기준: .claude/rules/coding-style.md (도메인 중심 진화형 코딩 원칙)
> 보완 기준: .claude/rules/rust-security-style.md · .claude/rules/rust-test-style.md
> 리뷰 대상: [파일 목록]

### 📊 이슈 요약

| 분류 | 건수 | 처리 |
|------|------|------|
| 🚫 Blocking (Critical+High) | [N]건 | [처리 내용] |
| ⚠️ Recommended (Medium)     | [N]건 | [처리 내용] |
| 💡 Suggestions (Low)        | [N]건 | 참고 사항 |

### 🚨 이슈 상세

#### 🚫 Blocking Issues (Critical + High)

**[R-XX] [제목]** — `[파일명:행번호]`
[설명] / 근거: [rust-security-style.md §섹션 또는 rust-test-style.md §섹션]
<details><summary>수정 제안</summary>

**Before:**
```rust
[수정 전 코드]
```
**After:**
```rust
[수정 후 코드]
```
</details>

#### ⚠️ Recommended Changes (Medium) / 💡 Suggestions (Low)
[동일 형식]

### ✅ 이상 없는 카테고리
[목록]

### 📝 종합 평가
[2~3줄]

---
> 👤 **리뷰어 체크포인트**: `CODE_REVIEW_RUST.md §인간 리뷰어가 반드시 확인해야 할 체크포인트` 참조.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

─── PR 모드 마무리 ──────────────────────
  cd ..
  git worktree remove ./review-[PR번호]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-D. PR 설명 초안 (로컬 모드 — fix/cr 브랜치)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝  PR 초안 (fix/cr 브랜치)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

■ PR 제목
  fix([모듈명]): 코드 리뷰 지적사항 수정 ([N]건)

■ PR 본문
────────────────────────────────────────
## 개요
코드 리뷰에서 발견된 [N]건의 이슈를 수정합니다.
원본 PR: #[번호] (해당 시)

## 수정 항목

| 항목 | 심각도 | 파일 | 변경 내용 | 적용 규칙 |
|------|--------|------|-----------|-----------|
| [R-XX] | 🔴 | [파일:행] | [내용] | [security/rust-test-style.md 해당 시] |

## 수정하지 않은 항목

| 항목 | 사유 |
|------|------|
| [R-XX] | [사유] |

## 테스트
- [ ] cargo test --all 전체 통과
- [ ] cargo clippy -D warnings 경고 0건
────────────────────────────────────────

■ gh CLI
  git push origin fix/cr-[module-name]
  gh pr create \
    --title "fix([모듈명]): 코드 리뷰 지적사항 수정 ([N]건)" \
    --body "위 본문" \
    --base [원본 브랜치 또는 main]

─── 로컬 모드 마무리 ────────────────────
  git push origin [현재 브랜치명]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 카테고리 빠른 참조 (`/code-review-rust --catalog`)

| 코드 | 카테고리 | coding-style.md | 핵심 탐지 신호 | 분류 | 보완 규칙 |
|------|----------|-----------------|----------------|------|-----------|
| **R-01** | 도메인 모델 | §1.3, §2.2, §4 | primitive 직접 사용, 유비쿼터스 언어 미반영, 이름이 구현 방식을 드러냄 | ⚠️💡 | — |
| **R-02** | 상태 & 모델링 | §2.2, §2.4 | bool 남용, invalid state 존재, 상태 전이 불명확, Tell-Don't-Ask 위반 | 🚫⚠️ | — |
| **R-03** | 경계 조건 | §5 전체 | `v[0]`, 0 나누기, 오버플로우, `unwrap_or_default()`, 침묵하는 실패 | 🚫 | — |
| **R-04** | 에러 처리 | §5.3, §5.4, §1.2 | `unwrap()`, `panic!` in lib, 문자열 에러, 내부 정보 노출 | 🚫⚠️ | rust-security-style.md §5 에러 처리와 정보 노출 |
| **R-05** | 소유권 & 메모리 | §1.1, §2.1 | `.clone()` 남발, `static mut`, `std::Mutex` in async, mutable 과다 | 🚫⚠️ | — |
| **R-06** | 제어 흐름 | §2.4, §2.1 | 중첩 깊이 >2, 수동 루프, `_` 패턴 남용, 복잡한 표현 | ⚠️💡 | — |
| **R-07** | 추상화 & trait | §2.3, §1.1 | 과도한 제네릭, 테스트 편의 추상화, 3회 미만 추상화 도입 | 💡 | — |
| **R-08** | 테스트 | §6 전체 | 에러 케이스 없음, 도메인 행동 미검증, 구현 세부 검증 | ⚠️💡 | **rust-test-style.md 전체** |
| **R-09** | 보안 | §7 전체 | 하드코딩 시크릿, SAFETY 주석 없음, SQL 포맷 조합, unwrap in lib, JWT none 허용, BOLA, 역직렬화 미검증, 내부 정보 노출 | 🚫 | **rust-security-style.md §1~§12** |

---

## 금지 사항

```
🚫 main 브랜치에 직접 수정 커밋
🚫 승인 없이 코드 자동 변경
🚫 PR worktree에서 직접 커밋
🚫 여러 이슈를 단일 커밋으로 묶기
🚫 기능 변경 (리뷰는 품질 개선만)
🚫 Cargo.toml 크레이트 추가 (명시적 요청 없이)
🚫 테스트 삭제 또는 비활성화 (rust-test-style.md §13. PR 거절 신호 (Red Flags) 참조)
🚫 Suggestions 이슈를 Blocking으로 과장 보고
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `CODE_REVIEW_RUST.md` | R-01~R-09 체크리스트 | 스킬 실행 시 항상 |
| `../../rules/coding-style.md` | 도메인 중심 코딩 원칙 — R-01~R-09 **1차 판단 기준** | **STEP 2 분석 시작 전 로드 (1순위)** |
| `../../rules/rust-security-style.md` | 보안 규칙 — R-04·R-05·R-09 보완 기준 | **STEP 2 분석 시작 전 로드 (2순위)** |
| `../../rules/rust-test-style.md` | 테스트 규칙 — R-08 보완 기준 | **STEP 2 분석 시작 전 로드 (3순위)** |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
