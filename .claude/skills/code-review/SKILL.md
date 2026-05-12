---
name: code-review
description: >
  /code-review 커맨드로 실행되는 Rust 코드 리뷰 스킬.
  coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로, security-style.md(소스 레벨 보안 체크리스트)를 2차 기준으로 Rust 코드를 분석한다.
  세 가지 모드를 지원한다:
    PR 모드          — MCP로 PR 정보(제목·설명·대상 브랜치) 확인 후 해당 브랜치를 체크아웃하고, git diff로 변경 파일만 정확히 추출하여 리뷰한다.
    로컬 변경 모드     — 인수 없이 실행 시 기본값. git diff HEAD로 staged + unstaged 변경사항을 자동 감지하여 즉시 리뷰한다.
    staged 모드      — --staged 옵션. git diff --cached로 staged 변경사항만 리뷰한다.
  이슈마다 coding-style.md 또는 security-style.md §섹션 근거와 Before/After를 제시하고 사용자 확인 후에만 수정을 적용한다.
  PR 모드에서는 해당 브랜치를 로컬에 체크아웃하여 리뷰한다.
---

# `/code-review` 커맨드 스킬

## 커맨드 문법

```
# PR 리뷰
/code-review --pr 42                   PR #42를 MCP로 조회 후 리뷰

# 로컬 변경사항 리뷰
/code-review                           staged + unstaged 변경사항 전체 리뷰 (기본값)
/code-review --staged                  staged(git add된) 변경사항만 리뷰
```

필요 시에만 아래 옵션을 추가한다.

```
--with-tests      리뷰 후 /test-align 명령으로 테스트 갭 분석 및 보완 수행
--with-security   리뷰 후 /security-full-scan + /security-scan 보안 스캔 수행해 갭 분석 및 보완 수행
```

- `--with-tests ` 명령은 `! ls ~/.claude/skills/`` 또는 `! ls .claude/skills/`를 입력해 `/test-align` 명령이 있는지 확인해 없으면 사용자에게 확인한다.
- `--with-security` 전제 조건: [claude-security-scan](https://github.com/mimul/claude-security-scan) 이 설치되어 있어야 한다. `! ls ~/.claude/skills/`` 또는 `! ls .claude/skills/`를 입력해 `/security-full-scan`, `/security-scan`이 있는지 확인해 없으면 사용자에게 확인한다.

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

# STEP 1 리뷰 대상 코드 수집

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

---

# STEP 2 coding-style.md · security-style.md 기준 분석

## 2-1 `.claude/rules/coding-style.md` 기준 분석

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

## 2-2 `.claude/rules/security-style.md` 기반 Security 점검

리뷰 과정에서 보안 속성이 유지되는지, 새 취약점이 노출되는지 확인한다.

| 섹션 | 주요 체크 |
|---|---|
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

---

# STEP 3 리뷰 분석 리포트 작성

## 3.1 분류 체계

분석 결과를 다음 4가지 카테고리로 분류한다:

| 분류 | 의미 | 대응 |
|------|------|------|
| **🚫 Blocking Issues** | 반드시 수정 필요 (보안, 버그, 아키텍처 위반) | 머지 전 필수 수정 |
| **⚠️ Recommended Changes** | 권장 개선 사항 (성능, 가독성, 베스트 프랙티스) | 가능하면 이번 PR에 반영 |
| **💡 Suggestions** | 선택적 개선 아이디어 (리팩토링, 최적화 기회) | 향후 고려 |
| **📝 Tech Debt** | 향후 개선이 필요한 기술 부채 | 별도 이슈로 추적 |

> **보안 체크:** 2-2에서 발견된 항목은 security-style.md 분류 기준으로 심각도를 결정한다. 🚫 Critical(인증 우회·RCE·SQL Injection·Secret 노출·Privilege Escalation) 및 ⚠️ High(Stored XSS·IDOR·SSRF·Unsafe Deserialization·Path Traversal)는 반드시 **🚫 Blocking Issues**로 분류한다. ⚡ Medium(정보 노출·Rate limiting 부재·보안 헤더 누락·Weak crypto)은 **⚠️ Recommended Changes**, ℹ️ Low(Logging 개선·Audit 부족)는 **📝 Tech Debt**로 분류한다.

## 3.2 리포트 출력 형식

이슈가 없는 분류는 출력에서 생략한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 코드 리뷰 결과 — {파일명 또는 PR #{번호}}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking Issues ({N}건)
  • [파일명:행번호] 이슈 제목
    → 근거   : coding-style.md §{섹션번호} {섹션명}   ← 코딩 스타일 위반 시
               security-style.md §{섹션번호} {섹션명}  ← 보안 이슈 시
    → 설명   : 무엇이 문제인가, 어떤 위험(보안·버그·아키텍처 위반)이 있는가
    → 영향   : API 영향, transaction 영향, concurrency 영향 등 구체적으로 기술
    → Before :
      ```rust
      // 문제 코드
      ```
    → After  :
      ```rust
      // 개선 코드
      ```

⚠️ Recommended Changes ({N}건)
  • [파일명:행번호] 이슈 제목
    → 근거   : coding-style.md §{섹션번호} {섹션명}   ← 코딩 스타일 위반 시
               security-style.md §{섹션번호} {섹션명}  ← 보안 이슈 시
    → 설명   : ...
    → Before / After : ...

💡 Suggestions ({N}건)
  • [파일명:행번호] 이슈 제목
    → 근거   : coding-style.md §{섹션번호} {섹션명}
    → 설명   : ...

📝 Tech Debt ({N}건)
  • [파일명:행번호] 이슈 제목
    → 설명   : ...
    → 추적   : 별도 이슈 생성 권장

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 요약: 🚫 {N}건 / ⚠️ {N}건 / 💡 {N}건 / 📝 {N}건
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> **보안 체크:** 2-2에서 발견된 보안 이슈는 `근거` 필드에 `security-style.md §{섹션번호} {섹션명}`으로 출처를 명시한다. 코딩 스타일 위반과 보안 이슈가 동시에 해당하는 경우 두 근거를 모두 기재한다.

---

# STEP 4 리뷰 수정 적용

리포트 출력 후 **사용자 확인을 받은 뒤에만** 코드 수정을 진행한다.

이슈가 없으면 사용자에게 보고하고 STEP 5로 바로 진행한다.

## 4.1 수정 범위 확인

수정 전 사용자에게 아래를 확인한다:
- 🚫 Blocking Issues: 머지 전 필수 수정. 사용자 동의 없이도 수정 진행.
- ⚠️ Recommended Changes: 권장 수정. 사용자에게 이번 PR에 반영할지 확인.
- 💡 Suggestions / 📝 Tech Debt: 이번에는 수정하지 않고 리포트로만 전달.

> **보안 체크:** 2-2에서 발견된 보안 이슈(인증 우회·RCE·SQL Injection·Secret 노출·IDOR·SSRF·Path Traversal 등)는 분류와 무관하게 **🚫 Blocking Issues**로 처리하며, 사용자 확인 없이도 수정을 진행한다. 보안 이슈가 수정 범위에 포함될 때 사용자에게 보안 위험 내용을 간략히 설명한다.

## 4.2 이슈별 수정 사이클

🚫 → ⚠️ (사용자가 동의한 항목만) 순서로, 이슈 하나씩 아래 사이클을 반복한다.

```
① 해당 파일 Read로 최신 내용 확인
② 코드 수정 (Edit)
③ cargo build              # 수정 직후 컴파일 오류 조기 확인
④ cargo test {관련_모듈}   # 해당 모듈 단위 smoke test
⑤ 이상 없으면 커밋
```

> **보안 체크:** 보안 관련 코드를 수정하는 경우 ②와 ③ 사이에 다음을 추가로 확인한다.
> - 인증·인가 로직 변경 시 → 수정 후에도 deny-default 원칙이 유지되는가?
> - 에러 처리 변경 시 → stack trace·내부 경로가 응답에 노출되지 않는가?
> - 로깅 코드 변경 시 → token·password·PII가 로그에 포함되지 않는가?
> - 의존성 추가·업데이트 시 → `cargo audit`으로 신규 CVE가 없는지 확인한다.

커밋 메시지 형식:
```
fix({scope}): {수정 내용 요약}
```

빌드/테스트 실패 시:
- 원인을 파악하고 수정한다.
- 동일 이슈에서 2회 이상 실패하면 사용자에게 보고하고 해당 이슈를 건너뛴다.

---

# STEP 5 최종 검증

STEP 4의 모든 수정이 끝난 뒤 전체를 한 번에 검증한다.

## 5.1 Linter & Formatter

```bash
cargo clippy -- -D warnings          # 전체 경고 확인 (경고를 오류로 처리)
cargo fmt --check                    # 포맷 위반 확인
cargo fmt                            # 포맷 자동 적용 (위반 있을 경우)
```

## 5.2 전체 테스트 및 커버리지

```bash
cargo test --all
cargo tarpaulin --out Html --output-dir coverage/   # 커버리지 확인 (목표: 80%+)
```

테스트 실패 시:
1. 어느 변경이 테스트를 깨뜨렸는지 특정한다.
2. behavior change 판단:
   - **의도하지 않은 변경** → 해당 커밋을 `git revert`하고 STEP 4로 돌아간다.
   - **의도한 변경** → 사용자에게 보고하고 확인 후 테스트를 갱신한다.
3. 수정 후 `cargo test --all`을 재실행해 통과를 확인한다.

## 5.3 테스트 보완 (`--with-tests` 옵션)

`--with-tests` 옵션이 있으면 `/test-align` 명령을 실행한다:
- characterization test
- regression test
- edge case test
- flaky test 개선

테스트가 부족한 경우 기존 동작을 먼저 캡처하고(characterization test), behavior preserving verification을 우선한다.

## 5.4 보안 스캔 (`--with-security` 옵션)

`--with-security` 옵션이 있으면 아래 순서로 실행한다:

```bash
cargo audit                          # 의존성 보안 취약점 확인
```

1. `/security-full-scan` 명령으로 정적 분석을 진행하고 결과를 반영한다.
2. `/security-scan` 명령으로 동적 분석을 진행한다.
   서버가 구동되지 않은 경우 `cargo run`으로 실행 후 재시도한다.

> **보안 체크:** STEP 2의 2-2 체크리스트에서 발견된 항목이 수정 후 해소되었는지 재검증한다. 특히 §1 Authentication·§2 Authorization·§3 Input Validation·§6 Cryptography & Secrets 항목은 정적 분석 결과와 대조하여 누락 여부를 확인한다.

---

# STEP 6 push 및 PR 제출

모드별로 처리가 다르다.

## PR 모드 (`--pr` 옵션)

이미 PR이 존재하므로 push만 한다. PR은 자동으로 업데이트된다.

```bash
git push origin {headBranch}
```

## 로컬 변경 모드 / staged 모드

새 브랜치가 없으면 생성한 뒤 push하고 PR을 생성한다.

```bash
# push
git push -u origin {브랜치명}

# PR 생성
gh pr create \
  --title "fix({scope}): {변경 요약}" \
  --body "$(cat <<'EOF'
## 리뷰 대응 요약

- coding-style.md §{섹션} 위반 수정
- ...

## 변경 파일

- `src/...`

## 테스트 플랜

- [ ] `cargo test --all` 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] `cargo fmt --check` 통과
- [ ] 커버리지 80% 이상 유지
EOF
)"
```

## 제출 후 확인

- [ ] CI 통과 여부 확인
- [ ] PR 설명에 리뷰 대응 내용 포함
- [ ] 미수정 💡/📝 항목은 별도 이슈로 등록 (필요 시)
