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
