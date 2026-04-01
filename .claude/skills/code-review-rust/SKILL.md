---
name: code-review-rust
description: >
  /code-review-rust 커맨드로 실행되는 Rust 코드 리뷰 스킬.
  두 가지 모드를 지원한다:
    PR 모드  — MCP로 PR 정보(제목·설명·대상 브랜치) 확인 후 전용 worktree를
               생성하고, git diff로 변경 파일만 정확히 추출하여 리뷰한다.
    로컬 모드 — 현재 작업 브랜치 또는 붙여넣은 코드를 직접 리뷰한다.
  두 모드 모두 CODE_REVIEW_RUST.md의 10개 카테고리(C-CR-01~C-CR-10) 기준으로
  분석하고, 이슈마다 Before/After를 제시하고 인간 확인 후에만 수정을 적용한다.
triggers:
  - /code-review-rust
  - /code-review-rust --pr [PR번호]
  - /code-review-rust [파일경로 또는 모듈명]
  - /code-review-rust --scope [카테고리 키워드]
  - /code-review-rust --severity [critical|high|medium|low]
  - /code-review-rust --catalog
  - /code-review-rust --help
reference: CODE_REVIEW_RUST.md
rules:
  - ../../rules/security.md
  - ../../rules/test.md
---

# `/code-review-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/code-review-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`CODE_REVIEW_RUST.md`의 10개 카테고리를 기준으로 Rust 코드를 분석하고,
**`security.md`와 `test.md` 규칙을 STEP 2 분석 시작 전에 로드하여**
해당 규칙을 각 카테고리 판단 기준으로 직접 적용한다.

리뷰의 핵심 불변 조건:
- **PR 기반 격리** — PR 리뷰는 전용 worktree에서 수행, main 브랜치 보호
- **변경분만 리뷰** — `git diff`로 실제 변경 파일만 정확히 추출
- **security.md 적용** — C-CR-01·05·07 판단 시 보안 규칙을 판단 기준으로 사용
- **test.md 적용** — C-CR-10 판단 시 테스트 규칙의 커버리지·네이밍 기준 적용
- **보여주고 확인받기** — Before/After 제시 → 인간 승인 후에만 수정 적용
- **심각도 우선** — Critical → High → Medium → Low 순서로 처리
- **항상 그린** — 수정 후 `cargo test` + `cargo clippy` 통과 확인

---

## 커맨드 문법

```
/code-review-rust --pr 42              PR #42를 MCP로 조회 후 worktree 리뷰
/code-review-rust --pr 42 --scope error  PR 리뷰 + 에러 처리 집중
/code-review-rust                      현재 브랜치 또는 붙여넣은 코드 리뷰
/code-review-rust src/order/handler.rs 특정 파일만 리뷰
/code-review-rust --scope error        C-CR-01 에러 처리만
/code-review-rust --scope ownership    C-CR-02 소유권·차용만
/code-review-rust --scope async        C-CR-06 비동기만
/code-review-rust --scope unsafe       C-CR-07 unsafe만
/code-review-rust --scope test         C-CR-10 테스트만
/code-review-rust --scope security     security.md 전체 기준 집중 리뷰
/code-review-rust --severity critical  Critical 이슈만 보고
/code-review-rust --severity high      High 이상 이슈만 보고
/code-review-rust --catalog            카테고리 목록 출력
/code-review-rust --help               사용법 출력
```

---

## 실행 모드 판별

```
--pr [번호] 있음 → [PR 모드]
                   STEP 0-PR → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5

--pr 없음        → [로컬 모드]
                   STEP 0-LOCAL → STEP 1 → STEP 2 → STEP 3 → STEP 4 → STEP 5
```

STEP 1~5는 두 모드 공통이다.

---

## [PR 모드] STEP 0-PR — PR 정보 확인 및 Worktree 준비

**이 단계에서 인간의 완료 확인을 받기 전까지 STEP 1로 진행하지 않는다.**

### 0-PR-1. MCP로 PR 정보 조회

MCP GitHub 도구로 아래 항목을 조회한다:
- PR 제목 / 설명(본문) / 작성자
- base 브랜치 / head 브랜치
- 변경 파일 수 및 목록
- 라벨

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

### 0-PR-2. 리뷰용 Worktree 생성 커맨드 출력

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  리뷰 Worktree 준비
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  git fetch origin [PR 브랜치명]

  git worktree add ./review-[PR번호] origin/[PR 브랜치명]

  cd ./review-[PR번호]

  git diff --name-only origin/[base 브랜치]...HEAD

  git diff origin/[base 브랜치]...HEAD -- '*.rs'

  cargo check

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✋ 준비 완료 후 "준비됐어"라고 알려주세요.
   git diff --name-only 결과를 함께 붙여넣으면 리뷰 범위를 자동 파악합니다.
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

**방식 A — 권장**: Claude가 Before/After 제안 → PR 코멘트 초안 생성 → 작성자가 직접 수정
**방식 B**: `fix/cr-pr[번호]-[module]` 브랜치에 직접 수정 후 push
**방식 C**: worktree에서 로컬 브랜치 생성 후 수정 push

---

## [로컬 모드] STEP 0-LOCAL — 브랜치 상태 확인

```
아래 커맨드로 현재 브랜치를 확인해 주세요:
  git branch --show-current
```

**브랜치별 분기:**

- `feature/*` / `fix/*` → 정상 진행. `git diff --name-only origin/main...HEAD` 확인 요청
- `main` / `master` → 경고 후 선택 요청 (A: fix/cr 브랜치 생성 / B: 리포트만)
- `fix/cr-*` → 이전 리뷰 수정 작업 중으로 인식하고 이어서 진행

**로컬 모드 수정 커밋 네이밍:**
```
feature/xxx 작업 중  → 같은 브랜치에 커밋
main에서 수정 필요   → fix/cr-{module-name} 생성

커밋 메시지: fix([scope]): [C-CR-XX] [50자 이내 요약]
```

---

## STEP 1 — 리뷰 대상 코드 접수

**PR 모드**: worktree 완료 확인 후, diff 결과 기반으로 리뷰 대상 파일 안내

**로컬 모드**: 코드를 요청한다
```
리뷰할 Rust 코드를 붙여넣거나 파일 경로를 알려주세요.

추가 정보 (선택):
  - 이 코드의 역할 (서비스 레이어, DB 어댑터, HTTP 핸들러 등)
  - async runtime 사용 여부
  - 라이브러리 / 바이너리 크레이트 구분
  - 특히 집중해서 봐줬으면 하는 부분
```

코드 수신 후 내부적으로 파악한다:
- 주요 fn / struct / impl / trait 목록
- async 여부 / unsafe 여부 / 테스트 존재 여부 / pub 노출 범위

---

## STEP 2 — rules 로드 및 10개 카테고리 분석

**분석 시작 전 `security.md`와 `test.md`를 로드하고,
각 파일의 규칙을 아래와 같이 해당 카테고리 판단에 직접 적용한다.**

### security.md → C-CR-01 · C-CR-05 · C-CR-07 판단 기준

**C-CR-01 에러 처리** 판단 시 security.md §에러 응답 적용:
- `unwrap()`/`expect()`가 라이브러리 코드에 있으면 Critical
- 에러 메시지에 DB 쿼리·파일 경로 등 내부 정보가 포함되면 High로 격상
- `thiserror` 미사용 시 외부 노출 에러 타입의 `Display` 구현 여부 확인

**C-CR-05 동시성** 판단 시 security.md §인증·권한 적용:
- `static mut` 사용 시 Critical (데이터 레이스 = 보안 위험)
- async 컨텍스트에서 `std::sync::Mutex` 사용 시 High

**C-CR-07 unsafe** 판단 시 security.md §unsafe 적용:
- `unsafe` 블록에 `// SAFETY:` 주석 없으면 **Critical**로 판정
- 원시 포인터 null 체크 없으면 **Critical**
- FFI 경계 ABI 미검증이면 **High**

### test.md → C-CR-10 판단 기준

**C-CR-10 테스트** 판단 시 test.md 전체 적용:
- test.md §커버리지 기준: `domain/` 90%, 전체 80% 목표 기준으로 테스트 부족 여부 판단
- test.md §에러 케이스 필수 목록: `Result` 반환 함수에 에러 케이스 테스트 없으면 High
- test.md §네이밍: `test1()`, `test_order()` 등 의미 없는 이름이면 Low
- test.md §금지 패턴: `#[ignore]` 무단 추가, 공유 상태 테스트 있으면 Medium
- test.md §Result 반환 테스트: `#[should_panic]` 사용 시 Medium (Result 반환 방식 권고)

### 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /code-review-rust 분석 리포트
    [PR 모드: PR #[번호] — [PR 제목]]
    [로컬 모드: 브랜치 [브랜치명]]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 리뷰 범위
   - 파일:  [리뷰한 .rs 파일 목록]
   - 구성:  [주요 fn/struct/trait 목록]
   - async: [있음 / 없음]  |  unsafe: [있음 / 없음]  |  테스트: [있음 / 없음]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚨 발견된 이슈 ([N]건)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 Critical ([N]건)
  • [C-CR-XX] [파일명:행번호] [이슈 제목]
    → [설명] / 근거: [security.md §섹션 또는 test.md §섹션]

🟠 High ([N]건)
  • ...

🟡 Medium ([N]건)
  • ...

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

## STEP 3 — 수정 계획 수립 및 확인

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  수정 계획 — 총 [N]건
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 즉시 수정 권장 (Critical + High: [N]건)
  1. [C-CR-XX] [이슈 제목] — [파일명:행번호]

🟡 선택적 수정 (Medium: [N]건)
  2. [C-CR-XX] [이슈 제목] — [파일명:행번호]

🔵 참고 사항 (Low: [N]건)
  3. [C-CR-XX] [이슈 제목] — [파일명:행번호]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  "전체 수정" / "Critical만" / "High 이상만" / "[번호]번만" / "리포트만"
```

---

## STEP 4 — Before/After 비교 제시 → 인간 확인 → 수정 적용

이 단계는 수정할 이슈 수만큼 반복(루프)된다.
Claude는 **절대 먼저 코드를 변경하지 않는다.**

### 4-A. Before/After 비교 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[심각도 이모지] [C-CR-XX] [이슈 제목]  —  Before/After 비교
    ([진행 현황: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 위치:   [파일명 : 행번호]
📖 문제:   [위험한 이유 1~2줄]
⚠️  심각도: [🔴 Critical / 🟠 High / 🟡 Medium / 🔵 Low]
📏 규칙:   [해당 시 — security.md §섹션 또는 test.md §섹션]

─── BEFORE ──────────────────────────────
[문제 코드]

─── AFTER ───────────────────────────────
[수정 코드]

─── 수정 근거 ───────────────────────────
  • [변경 포인트 1]
  • [변경 포인트 2]
  • (보안 관련 시) 🔒 security.md §[섹션]: [규칙 설명]
  • (테스트 관련 시) 🧪 test.md §[섹션]: [확인 사항]
  • (해당 시) ⚠️  영향 범위: [다른 코드에 미치는 영향]

─── 검증 커맨드 ─────────────────────────
  cargo fmt
  cargo clippy -- -D warnings
  cargo test [관련_테스트_경로]

─── 커밋 메시지 제안 ────────────────────
  fix([scope]): [C-CR-XX] [50자 이내 요약]

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
| `"전체 적용"` | 일괄 적용 (**Critical + 보안 이슈는 개별 확인 유지**) |

### 4-C. 수정 적용 시 출력

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  [C-CR-XX] 수정 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

수정된 파일: [파일 경로]
[수정된 최종 코드]

실행 순서:
  1. cargo fmt
  2. cargo clippy -- -D warnings
  3. cargo test [관련_테스트_경로]
  4. git add [파일]
  5. git commit -m "fix([scope]): [C-CR-XX] [요약]"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-D. Critical + 보안 이슈 특별 처리

`🔴 Critical` 이슈와 `security.md`에서 비롯된 보안 이슈는
`"전체 적용"` 명령에도 **반드시 개별 확인**을 받는다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔒 보안 이슈 — 개별 확인 필요
   (security.md §[섹션] 위반)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[동일한 Before/After 비교 형식]

🔒 security.md 규칙:
   [해당 규칙 1~3줄 인용]
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
  ✅ [C-CR-XX] [제목] — [파일명:행번호]

건너뛴 이슈 ([M]건):
  ⏭️  [C-CR-XX] [제목] — [사유]

참고 사항 (Low):
  🔵 [C-CR-XX] [제목] — [내용]

최종 검증 커맨드:
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test --all

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  □ cargo fmt --check 포맷 위반 없음
  □ Critical / High 이슈 전부 해결
  □ unsafe SAFETY 주석 완비 (security.md §unsafe)
  □ 비밀 정보 하드코딩 없음 (security.md §비밀 정보)
  □ 에러 케이스 테스트 존재 (test.md §필수 목록)
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
  xxxxxxx fix([scope]): [C-CR-XX] [요약]
  xxxxxxx fix([scope]): [C-CR-XX] [요약]
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

> 리뷰 기준: CODE_REVIEW_RUST.md C-CR-01~10
> 보안 기준: .claude/rules/security.md
> 테스트 기준: .claude/rules/test.md
> 리뷰 대상: [파일 목록]

### 📊 이슈 요약

| 심각도 | 건수 | 처리 |
|--------|------|------|
| 🔴 Critical | [N]건 | [처리 내용] |
| 🟠 High     | [N]건 | [처리 내용] |
| 🟡 Medium   | [N]건 | [처리 내용] |
| 🔵 Low      | [N]건 | 참고 사항 |

### 🚨 이슈 상세

#### 🔴 Critical

**[C-CR-XX] [제목]** — `[파일명:행번호]`
[설명] / 근거: [security.md §섹션 또는 test.md §섹션]
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

#### 🟠 High / 🟡 Medium
[동일 형식]

### ✅ 이상 없는 카테고리
[목록]

### 📝 종합 평가
[2~3줄]

---
> 👤 **리뷰어 체크포인트**: 비즈니스 로직 정확성 · 요구사항 충족 · 버그 가능성 · 과도한 변경을 추가로 확인해 주세요.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

─── PR 모드 마무리 ──────────────────────
  cd ..
  git worktree remove ./review-[PR번호]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-C. PR 설명 초안 (로컬 모드 — fix/cr 브랜치)

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
| [C-CR-XX] | 🔴 | [파일:행] | [내용] | [security/test.md 해당 시] |

## 수정하지 않은 항목

| 항목 | 사유 |
|------|------|
| [C-CR-XX] | [사유] |

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

| 코드 | 카테고리 | 핵심 탐지 신호 | 심각도 | 적용 규칙 |
|------|----------|----------------|--------|-----------|
| **C-CR-01** | 에러 처리 | `unwrap()`, `panic!` in lib | 🔴🟠 | security.md §에러 응답 |
| **C-CR-02** | 소유권·차용 | `.clone()` 남발, `&mut T` 과용 | 🟡 | — |
| **C-CR-03** | 에지 케이스 | `v[0]`, 0 나누기, 오버플로우 | 🔴🟠 | — |
| **C-CR-04** | 타입 설계 | `bool` 파라미터, `u64` 혼동 | 🟡 | — |
| **C-CR-05** | 동시성·스레드 | `static mut`, `std::Mutex` in async | 🔴 | security.md §인증·권한 |
| **C-CR-06** | 비동기 | `std::fs` in async, `.await` 누락 | 🟠 | — |
| **C-CR-07** | unsafe | SAFETY 주석 없음, null 체크 없음 | 🔴🟠 | **security.md §unsafe** |
| **C-CR-08** | 코드 품질 | 매직 넘버, `pub` 과노출, rustdoc 없음 | 🔵 | — |
| **C-CR-09** | Rust 관용 표현 | 수동 루프, 장황한 match | 🟡 | — |
| **C-CR-10** | 테스트 | 에러 케이스 없음, 의미없는 이름 | 🟡🔵 | **test.md 전체** |

---

## 금지 사항

```
🚫 main 브랜치에 직접 수정 커밋
🚫 승인 없이 코드 자동 변경
🚫 PR worktree에서 직접 커밋
🚫 여러 이슈를 단일 커밋으로 묶기
🚫 기능 변경 (리뷰는 품질 개선만)
🚫 Cargo.toml 크레이트 추가 (명시적 요청 없이)
🚫 테스트 삭제 또는 비활성화 (test.md §금지 패턴 참조)
🚫 Low 이슈를 Critical로 과장 보고
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `CODE_REVIEW_RUST.md` | C-CR-01~10 체크리스트 | 스킬 실행 시 항상 |
| `../../rules/security.md` | 보안 규칙 | **STEP 2 분석 시작 전 로드** |
| `../../rules/test.md` | 테스트 규칙 | **STEP 2 분석 시작 전 로드** |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
