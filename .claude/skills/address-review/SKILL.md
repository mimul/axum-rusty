---
name: address-review-rust
description: /address-review 커맨드로 실행되는 리뷰 대응 스킬. 코드 리뷰 결과를 받은 사람이 각 지적 사항의 타당성을 판단하고 수정을 수행하는 데 Claude가 보조하는 스킬이다.
  STEP 2 타당성 평가 시 coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로, security-style.md(소스 레벨 보안 체크리스트)를 2차 판단 기준으로, test-style.md를 보완 기준으로 적용한다.
  각 지적의 기술적 타당성과 프로젝트 정책 적합성을 독자적으로 평가하고, 타당한 지적만 Before/After로 수정하며 최종 요약을 출력한다.
---

# `/address-review` 커맨드 스킬

coding-style.md(1차), security-style.md(2차), test-style.md(보완) 로드 후 지적사항에 대해 타당성을 독립적으로 평가해 해당 지적사항을 수정한다.

## 커맨드 문법

```
# PR 리뷰 피드백
/address-review --pr 42                   PR #42 지적사항 독립적 평가 후 수정
/address-review --pr 42 --dry-run         PR #42 지적사항 독립적 평가 후 수정사항에 대해 Before/After 포함한 초안만 출력

```

필요 시에만 아래 옵션을 추가한다.

```
--with-tests      지적사항 수정 후 /test-align 명령으로 테스트 갭 분석 및 보완 수행
--with-security   지적사항 수정 후 /security-full-scan + /security-scan 보안 스캔 수행해 갭 분석 및 보완 수행
```

- `--with-tests` 명령은 `! ls ~/.claude/skills/` 또는 `! ls .claude/skills/`를 입력해 `/test-align` 명령이 있는지 확인해 없으면 사용자에게 확인한다.
- `--with-security` 전제 조건: [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다. `! ls ~/.claude/skills/` 또는 `! ls .claude/skills/`를 입력해 `/security-full-scan`, `/security-scan`이 있는지 확인해 없으면 사용자에게 확인한다.

---

## STEP 0

Bash 도구로 아래 커맨드를 직접 실행하여 코멘트를 수집한다:

```bash
# 1. PR 기본 정보 확인
gh api repos/{owner}/{repo}/pulls/{번호}

# 2. 리뷰 본문 수집 (전체 리뷰 코멘트)
gh api repos/{owner}/{repo}/pulls/{번호}/reviews

# 3. 인라인 코드 코멘트 수집
gh api repos/{owner}/{repo}/pulls/{번호}/comments
```

**수집 후 출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📥  PR #[번호] 리뷰 수집 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
제목:      [PR 제목]
리뷰어:    [리뷰어 목록]
리뷰 수:   [N]건 (승인 [N] / 변경 요청 [N] / 코멘트 [N])
인라인:    [N]개 파일, [N]개 코멘트
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**수집 실패 시:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  PR #[번호] 리뷰를 가져올 수 없습니다
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
gh auth status 로 인증 상태를 확인해 주세요.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 1 — 지적 목록 정리 및 코드 로드

STEP 0에서 수집한 리뷰 내용을 구조화하고, 언급된 파일을 Read 도구로 직접 읽는다.

**지적 목록 정리 기준:**

| 구분 | 내용 |
|------|------|
| 지적 ID | A-RV-01, A-RV-02 … (본 스킬 내 관리용) |
| 출처 | 리뷰어명 / 리뷰 타입 (전체 코멘트 / 인라인) |
| 위치 | 파일명:행번호 (인라인인 경우) |
| 내용 | 지적 원문 요약 |
| 분류 | 🚫 Blocking / ⚠️ Recommended / 💡 Suggestions / 📝 Tech Debt (리뷰어 명시 또는 문맥 추정) |

**출력 형식:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  지적 목록 — 총 [N]건
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[A-RV-01] [리뷰어] | [파일명:행번호 또는 전체]  🚫 Blocking
  지적: [원문 요약]

[A-RV-02] [리뷰어] | [파일명:행번호 또는 전체]  ⚠️ Recommended
  지적: [원문 요약]
...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 2 — 각 지적의 타당성 평가

**평가 시작 전 아래 세 파일을 반드시 이 순서대로 로드한다:**

1. `.claude/rules/coding-style.md` — **1차 판단 기준** (모든 평가의 근본 기준)
2. `.claude/rules/security-style.md` — **2차 판단 기준** (보안 점검 기준)
3. `.claude/rules/test-style.md` — 보완 기준 (테스트 관련 지적 시)

각 지적에 대해 아래 5가지 기준으로 독립적으로 평가한다.
**리뷰어 의견에 동조하지 않고 coding-style.md 원칙과 코드·프로젝트 정책을 직접 확인하여 판단한다.**

---

### ① coding-style.md 1차 판단 (가장 먼저 적용)

지적이 coding-style.md의 어느 원칙에 해당하는지 §섹션을 특정한다. 특정된 §섹션이 있으면 해당 원칙을 판단의 근본 근거로 삼는다.

| 지적 내용 | coding-style.md 근거 | 판단 원칙 |
|-----------|----------------------|-----------|
| 도메인 개념이 타입에 드러나지 않음 | §1 Domain First, §11 Type-Driven Design | Primitive Obsession, 이름이 도메인 용어가 아님 |
| invalid state 허용, bool 남용 | §11 Type-Driven Design, §4 Readability First | 불가능 상태 허용, 상태가 Enum으로 표현되지 않음 |
| 에지 케이스 암묵적 처리, 경계 미처리 | §10 Error Handling, §3 Explicit & Intentional Code | 침묵하는 실패, 의미 없는 기본값 |
| 에러 의도 불명확, unwrap 남용 | §10 Error Handling | 도메인 의미 없는 에러, Silent Failure |
| clone() 남발, 불필요한 mut | §12 Async & Concurrency | 변경하기 어려운 소유권 구조, Shared Mutable State |
| 중첩 깊이 >2, 복잡한 흐름 | §4 Readability First | 명확한 흐름 미선택, Early Return 미적용 |
| 3회 미만 반복에 추상화 도입 | §5 Complexity Control & Simplicity | Rule of Three 미달, Speculative Abstraction |
| 레이어 경계 위반, 의존 방향 오류 | §2 Architecture First | 의존 방향 역전, framework concern 혼입 |
| 테스트가 도메인 행동 미검증 | §18 Testing Philosophy, test-style.md §1~§13 | 구현 세부사항 검증, 의미 없는 assertion |
| 입력 미검증, 시크릿 하드코딩 | security-style.md §1~§15 | 신뢰하지 않은 입력 사용, 보안 설계 누락 |

**coding-style.md §19 Anti-Patterns와 대조하여 체크한다:**

코드 레벨 Anti-Patterns:
- Anemic Domain Model (행동 없는 도메인 객체) → §1 Domain First 위반
- Primitive Obsession (기본 타입으로 도메인 개념 표현) → §1 Domain First, §11 Type-Driven Design 위반
- Technical Naming Dominance (기술 이름이 도메인 압도) → §7 Consistency & Predictability 위반
- God Class / God Object (과도한 책임 집중) → §5 Complexity Control & Simplicity 위반
- Feature Envy (다른 객체의 데이터 과도 의존) → §8 Usecase Oriented Design 위반
- Silent Failure (예외 무시·의미 없는 기본값) → §10 Error Handling 위반
- Long-lived Duplicated Logic & Deep Nesting → §4 Readability First, §5 위반
- Shared Mutable State (공유 가변 상태) → §12 Async & Concurrency 위반
- Magic Numbers & Magic Strings → §3 Explicit & Intentional Code 위반
- Giant Files / Oversized Modules → §5 Complexity Control & Simplicity 위반

설계 / 아키텍처 레벨 Anti-Patterns:
- Speculative Abstraction (성급한 추상화) → §5 Complexity Control & Simplicity 위반
- Framework-driven Design → §2 Architecture First, §6 Changeability & Refactoring 위반
- Hidden Side Effects → §3 Explicit & Intentional Code 위반
- Cyclic Dependencies → §2 Architecture First 위반
- DB-First / UI-First Domain Modeling → §1 Domain First 위반

보안·테스트 Anti-Patterns:
- 테스트 없는 핵심 로직 → §18 Testing Philosophy, test-style.md §1~§13 위반
- 보안을 고려하지 않은 설계 → security-style.md §1~§15 위반
- 입력 신뢰 (신뢰 경계 미구분) → security-style.md §3 Input Validation 위반
- 하드코딩 시크릿 → security-style.md §6.1 Secrets Management 위반

---

### ② security-style.md 2차 보안 판단

보안 관련 지적이 포함된 경우, security-style.md의 아래 섹션을 기준으로 독립적으로 평가한다.

> security-style.md 이슈는 심각도 등급에 상관없이 판정에서 항상 🚫 Blocking으로 분류한다.

| 섹션 | 주요 체크 |
|------|-----------|
| §1 Authentication | JWT signature·expiration 검증 존재, credential 하드코딩 없음, 로그에 token 미출력, logout 후 세션 무효화 |
| §2 Authorization | 인가 검증 누락 API 없음, IDOR 없음, 수평·수직 권한 상승 불가, multi-tenant 격리 |
| §3 Input Validation | Prepared Statement 사용, SQL/Command Injection 없음, HTML escaping 적용, 역직렬화 검증 존재 |
| §4 File Handling | 파일 확장자·MIME 검증, path traversal 차단, canonical path 검증, 실행 가능 파일 업로드 차단 |
| §5 API Security | 인증 없는 엔드포인트 없음, excessive data exposure 없음, rate limiting 존재 |
| §6 Cryptography & Secrets | API key·password 하드코딩 없음, 약한 난수·ECB mode 미사용, TLS 검증 비활성화 없음 |
| §7 Logging | 비밀번호·token·개인정보 로그 미출력, log forging 불가, structured logging 적용 |
| §8 Error Handling | stack trace·내부 경로·SQL 오류 외부 미노출, deny-default, 예외 시 보안 우회 불가 |
| §11 Concurrency | 대용량 payload 제한, ReDoS 패턴 없음, connection·fd leak 없음 |
| §12 Business Logic | 상태 전이 검증 유지, race condition 방지, replay attack 대응 |
| §14 Secure Coding | dynamic code execution 없음, unsafe native call 없음, trust boundary 명확 |
| §15 Rust 특화 | unsafe 블록 SAFETY 주석 존재, panic 기반 DoS 없음, serde 입력 검증 존재 |

---

### ③ 기술적 타당성

- 지적된 코드가 실제로 버그·성능·보안 문제를 일으키는가?
- Rust 언어 규칙 및 관용 표현에 비추어 맞는 지적인가?

---

### ④ 프로젝트 정책 적합성 (보완 기준)

해당 카테고리에만 적용하는 보완 기준:

| 지적 카테고리 | 적용 보완 기준 | 우선순위별 주요 확인 사항 |
|---------------|----------------|--------------------------|
| R-04 에러 처리 | security-style.md §8 Error Handling | 🚫 라이브러리·핸들러에 `unwrap()`/`expect()` 사용 (§8.2) — ⚠️ 에러 응답에 내부 정보(스택 트레이스·DB 에러) 포함 (§8.1) — ⚠️ 로그에 패스워드·토큰 등 민감 데이터 기록 (§7.1) |
| R-05 소유권 | security-style.md §15.4 Rust | 🚫 `unsafe` 블록에 `// SAFETY:` 주석 없음 (§15.4) — ⚠️ async 컨텍스트에서 `std::sync::Mutex` 사용 (coding-style.md §12) |
| R-08 테스트 | test-style.md §1~§13 전체 (우선순위 기반) | 🚫 통합 테스트 Mock DB 사용(§4.3) · Assertion 없는 테스트(§13.1) · 비결정적 출력 고정 사용(§10.2) · 이유 없는 `#[ignore]`(§10.3) — ⚠️ 핵심 로직 테스트 없음(§6.3) · `mockall expect` 과다(§13.2) · `Result` 에러 케이스 없음(§6) — 💡 네이밍 템플릿 미준수(§3.2) · AAA 패턴 미준수(§5.1) |
| R-09 보안 | security-style.md §1~§15 전체 (우선순위 기반) | 🚫 하드코딩 시크릿(§6.1) · SAFETY 주석 없음(§15.4) · SQL Injection(§3.1) · unwrap in lib(§8.2) · JWT none algorithm(§1.3) · IDOR/BOLA(§2.2) · 역직렬화 미검증(§3.5) · 내부 정보 노출(§8.1) · 약한 비밀번호 해시(§1.2) — ⚠️ 약한 암호화·ECB mode(§6.2) · Rate Limiting 부재(§5.1) · 감사 로그 부재(§7.2) · panic 기반 DoS(§15.4) |

- `CLAUDE.md`의 코딩 컨벤션(에러 처리, 소유권, 타입 설계 등)에 맞는가?
- 현재 프로젝트 맥락(레이어 역할, 의존성 구조)을 고려할 때 적절한가?

---

### ⑤ 구현 트레이드오프 타당성

- 수정 시 다른 코드에 미치는 영향 범위는 적절한가?
- 성능·가독성·유지보수성 간 균형이 맞는가?
- 수정 비용 대비 실질적 개선 효과가 있는가?

---

### 평가 결과 분류

| 판정 | 의미 | 처리 |
|------|------|------|
| ✅ 대응 | coding-style.md 원칙 또는 기술적으로 타당하고 수정이 필요하다 | STEP 3에서 수정 |
| ⚠️ 부분 대응 | 방향은 맞지만 제안 방식과 다르게 수정한다 | STEP 3에서 대안 수정 |
| ❌ 대응 불필요 | coding-style.md 원칙에 위배되지 않거나 현재 프로젝트에 적합하지 않다 | 이유 명시 후 스킵 |

---

### 평가 결과 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧠  타당성 평가 결과
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 대응    [N]건: [A-RV-XX], [A-RV-XX] …
⚠️ 부분    [N]건: [A-RV-XX] …
❌ 불필요   [N]건: [A-RV-XX] …

[A-RV-XX] ✅ 대응
  근거: coding-style.md §[섹션] [섹션명] — [판단 이유]
  보완: [security-style.md §섹션 / test-style.md §섹션] (해당 시)

[A-RV-XX] ✅ 대응  🔒 보안
  근거: security-style.md §[섹션] [섹션명] — [판단 이유]
  (보안 이슈는 심각도 무관 Blocking)

[A-RV-XX] ⚠️ 부분 대응
  근거: coding-style.md §[섹션] [섹션명] — [방향은 맞지만 제안 방식과 다른 이유]
  대안: [리뷰어 제안과 달리 적용할 방식]

[A-RV-XX] ❌ 대응 불필요
  근거: [coding-style.md §섹션 위반이 아닌 이유 / 프로젝트 정책과 불일치 / 트레이드오프 판단]
  회신: [리뷰어에게 전달할 설명 초안]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 수정 적용 (Before/After → 인간 확인 → 커밋)

**대응 / 부분 대응** 판정을 받은 지적에 대해서만 이 단계를 수행한다.
Claude는 **절대 먼저 코드를 변경하지 않는다.** Before/After를 제시하고 승인 후에만 적용한다.

### 수정 제안 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[판정 이모지] [A-RV-XX] [지적 제목]  —  수정 제안
    ([진행 현황: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 위치:   [파일명 : 행번호]
🏷️  분류:   [🚫 Blocking / ⚠️ Recommended / 💡 Suggestions]  [R-XX]
📖 지적:   [리뷰어 원문 요약]
🧠 판정:   [✅ 대응 / ⚠️ 부분 대응]
📐 근거:   coding-style.md §[섹션번호] [섹션명] — [판정 이유 1줄]
📏 보완:   [security-style.md §섹션 / test-style.md §섹션] (해당 시)

```rust
// Before
[현재 코드]

// After
[수정 코드]
```

─── 수정 근거 ───────────────────────────
  • [변경 포인트 1]
  • [변경 포인트 2]
  • 📐 coding-style.md §[섹션]: [해당 원칙 인용]
  • (보안 관련 시) 🔒 security-style.md §[섹션]: [규칙 설명]
  • (테스트 관련 시) 🧪 test-style.md §[섹션]: [확인 사항]
  • (⚠️ 부분 대응 시) 리뷰어 제안과 다른 이유: [설명]

─── 검증 커맨드 ─────────────────────────
  cargo fmt
  cargo clippy -- -D warnings
  cargo test [관련_테스트_경로]

─── 커밋 메시지 제안 ────────────────────
  fix([scope]): [A-RV-XX] [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 수정을 적용할까요?

   ✅ "적용" / "ok"          → 수정 적용 후 다음 지적
   ❌ "건너뜀" / "skip"      → 이 지적은 대응 불필요로 전환
   ✏️  "수정해줘: [요청]"    → After 코드 재제안
   ⏸️  "여기서 멈춰"          → 완료 요약으로 이동
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 사용자 응답별 처리

| 응답 | Claude 행동 |
|------|-------------|
| `"적용"` / `"ok"` / `"ㅇ"` | Edit 도구로 코드 수정 + cargo test 실행 + 다음 지적 |
| `"건너뜀"` / `"skip"` / `"ㄴ"` | 대응 불필요 목록에 추가 → 다음 지적 |
| `"수정해줘: [내용]"` | After 코드 재제안 → 재출력 |
| `"왜?"` / `"설명해줘"` | 판정 근거 상세 설명 → 같은 제안 유지 |
| `"전체 적용"` | 대응 판정 전체 일괄 적용 |
| `"여기서 멈춰"` / `"stop"` | 루프 종료 → STEP 4 |

### 수정 적용 후 검증

각 수정 적용 직후 Claude가 Bash 도구로 직접 실행한다:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test 2>&1 | tail -5
```

검증 실패 시 자동으로 수정을 롤백하고 원인을 분석하여 재제안한다.

---

## STEP 4 — 최종 요약 출력 및 push

모든 지적 처리 완료 후 아래 순서대로 실행한다.

### 4-1. PR 모드: 자동 push

PR 모드(`/address-review --pr [번호]`)에서 수정 커밋이 1건 이상 발생한 경우, 요약 출력 **전에** Claude가 직접 push를 실행한다:

```bash
git push origin [브랜치명]
```

push 결과를 확인한 후 요약을 출력한다. 대화 모드에서는 push를 실행하지 않는다.

### 4-2. 요약 출력 형식

````markdown
## 리뷰 대응 요약

### 지적 목록
- [x] [A-RV-01] [지적 개요] — ✅ 대응 완료
- [x] [A-RV-02] [지적 개요] — ⚠️ 부분 대응 완료
- [ ] [A-RV-03] [지적 개요] — ❌ 대응 불필요

### 대응한 지적
| 지적 | 판정 | 수정 내용 |
|------|------|----------|
| [A-RV-01] [제목] | ✅ 대응 | [수정 개요] |
| [A-RV-02] [제목] | ⚠️ 부분 | [수정 개요 + 리뷰어 제안과 다른 점] |

### 🔒 보안 이슈 대응
| 지적 | security-style.md 근거 | 수정 내용 |
|------|------------------------|----------|
| [A-RV-XX] [제목] | §[섹션] [섹션명] | [수정 개요] |

### 대응하지 않은 지적
| 지적 | 이유 |
|------|------|
| [A-RV-03] [제목] | [기술적 근거 / 정책 불일치 / 트레이드오프 판단] |

### 검증 결과
- cargo fmt:    ✅ 통과
- cargo clippy: ✅ 통과 (0 warnings)
- cargo test:   ✅ [N]개 통과

### Push 결과 (PR 모드)
- git push: ✅ origin/[브랜치명] 에 푸시 완료

### 다음 단계
- [ ] PR 코멘트에 대응 회신: `/reply-review-rust [PR번호]` 실행
- [ ] 추가 논의가 필요한 지적: [A-RV-XX]
````

---

## 주의사항

- 리뷰어의 권위에 의존하지 않고 **coding-style.md 원칙과 코드·정책을 직접 검토하여** 판단한다
- 판단의 우선순위: **coding-style.md(1차) → security-style.md(2차) → test-style.md(보완) → CLAUDE.md 컨벤션**
- 대응하지 않는 경우, `coding-style.md §섹션` 또는 `security-style.md §섹션`을 인용하여 리뷰어가 납득할 수 있는 **기술적 근거를 반드시** 기재한다
- 수정으로 인해 새로운 문제(컴파일 에러, 테스트 실패, clippy 경고)가 발생하지 않도록 매 수정 후 검증한다
- coding-style.md 원칙, security-style.md 보안 정책, `test-style.md` 규칙과 충돌하는 리뷰어 제안은 해당 규칙을 우선한다
- **security-style.md 이슈는 심각도 등급에 무관하게 항상 🚫 Blocking으로 처리한다**
- PR 모드에서 모든 수정 커밋 완료 후 **STEP 4에서 자동으로 `git push origin [브랜치명]`을 실행**하고 Push 결과를 요약에 포함한다
- push 완료 후 `/reply-review-rust [PR번호]`로 각 리뷰 코멘트에 대응 회신한다
