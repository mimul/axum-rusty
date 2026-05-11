---
name: code-review
description: >
  /code-review 커맨드로 실행되는 Rust 코드 리뷰 스킬.
  coding-style.md(도메인 중심 진화형 코딩 원칙)를 1차 판단 기준으로 Rust 코드를 분석한다.
  일곱 가지 모드를 지원한다:
    PR 모드          — MCP로 PR 정보(제목·설명·대상 브랜치) 확인 후 전용 worktree를 생성하고, git diff로 변경 파일만 정확히 추출하여 리뷰한다.
    로컬 변경 모드     — 인수 없이 실행 시 기본값. git diff HEAD로 staged + unstaged 변경사항을 자동 감지하여 즉시 리뷰한다.
    staged 모드      — --staged 옵션. git diff --cached로 staged 변경사항만 리뷰한다.
  이슈마다 coding-style.md §섹션 근거와 Before/After를 제시하고 인간 확인 후에만 수정을 적용한다.
---

# `/code-review` 커맨드 스킬


# 커맨드 문법

```
# PR 리뷰
/code-review --pr 42                   PR #42를 MCP로 조회 후 리뷰

# 로컬 변경사항 리뷰
/code-review                           staged + unstaged 변경사항 전체 리뷰 (기본값)
/code-review --staged                  staged(git add된) 변경사항만 리뷰
```

필요 시에만 아래 옵션을 추가한다.

```bash
--with-tests
--with-security
```

## Test 정책

`--with-tests` 옵션이 있으면 Claude는 6.2.2 `/test-align` 명령을 수행한다.

## 보안 정책

`--with-security` 옵션이 있으면 Claude는: 6.3 Security Scan을 수행한다.

**전제 조건으로 [claude-security-scan](https://github.com/mimul/claude-security-scan)** 이 설치되어야 한다.

# STEP 0 사전 조건 체크

리뷰 시작 전 아래를 순서대로 확인한다. 하나라도 실패하면 사용자에게 보고하고 중단한다.

```bash
git branch --show-current   # 현재 브랜치 확인 (main이면 중단 — CLAUDE.md: main 직접 커밋 금지)
cargo build                 # 빌드 통과 여부 확인
cargo test --all            # 기존 테스트 baseline 확인
```

확인 항목:
- [ ] 현재 브랜치가 main이 아님
- [ ] `cargo build` 통과
- [ ] `cargo test --all` 통과 (리뷰 전 baseline 확보)


# STEP 1  리뷰 대상 코드 접수
- 브랜치 최신 상태 확인
- 소스 변경 사항 수집을 위해 커맨드 문법별로 git 커맨드 제시

# STEP 2 `.claude/rules/coding-style.md` 로드 및 단락 분석

- `.claude/rules/coding-style.md` 단락별 체크리스트별로 분석


# 4. 리뷰 분석 리포트(리뷰 대상 항목)

## 4.1 리뷰 분석 분류 체계

리뷰 분석 결과는 다음 4가지 카테고리로 분류한다:

| 분류 | 의미 | 대응 |
|------|------|------|
| **🚫 Blocking Issues** | 반드시 수정이 필요한 항목 (보안, 버그, 아키텍처 위반) | 머지 전 필수 수정 |
| **⚠️ Recommended Changes** | 권장 개선 사항 (성능, 가독성, 베스트 프랙티스) | 가능하면 이번 PR에 반영 |
| **💡 Suggestions** | 선택적 개선 아이디어 (리팩토링, 최적화 기회) | 향후 고려 |
| **📝 Tech Debt** | 향후 개선이 필요한 기술 부채 | 별도 이슈로 추적 |

---

## 4.2 리뷰 분석 리포트 형식

1. **코드 위치**: 파일명과 라인 번호를 명시 (예: `src/infra/user.rs:42`)
2. **관련 근거**: coding-style.md [섹션명]
2. **문제 설명**: 무엇이 문제인지 명확히 설명
3. **개선 방안**: 개선된 코드 예시 제공(Before/After 비교 출력)
4. **우선순위**: 각 항목의 우선순위(`🚫`/`⚠️`/`💡`/`📝`) 명시

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚨 발견된 이슈 ([N]건)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚫 Blocking Issues ([N]건)
  • [파일명:행번호] [이슈 제목]
    → 분류 : coding-style.md: [섹션명]
    → 설명 : [설명]
    → 사유 : 분류 기준에 근거하여 사유 설명

⚠️ Recommended Changes ([N]건)
  • ...

💡 Suggestions ([N]건)
  • ...

📝 Tech Debt ([N]건)
  • ...


# 5. 리뷰 수행

 - 작업 단위를 작게 쪼개고, 브랜치는 feature/review-{작업단위} 형태로 만든다. 각 단위마다 리뷰 내용 수정 → 테스트 → 커밋을 반복한다.


6. Verification & Cleanup

## 6.1 Linter & Formatter 실행

아래를 순서대로 실행하고 결과를 개선한다.

```bash
cargo clippy --fix --allow-dirty     # 자동 수정 가능한 항목 수정
cargo clippy -- -D warnings          # 잔존 경고 확인 (경고를 오류로 처리)
cargo fmt                            # 포맷 자동 적용
```

## 6.2 전체 테스트 실행

### 6.2.1 반드시 수행:

- Unit Test
- Integration Test
- E2E Test
- Regression Test

실패 시:
1. 원인 분석: 어느 변경이 테스트를 깨뜨렸는지 특정한다.
2. behavior change 판단:
   - **의도하지 않은 behavior change** → 해당 커밋을 `git revert`하고 STEP 5로 돌아간다.
   - **의도한 behavior change** → 사용자에게 보고하고 확인을 받은 뒤 테스트를 갱신한다.
3. 재실행: 수정 후 `cargo test --all`을 다시 실행해 통과를 확인한다.

### 6.2.2 `/test-align` 명령을 실행(옵션 : `--with-tests` 옵션이 있을 경우 수행함)

- characterization test
- regression test
- edge case test
- flaky test 개선

을 함께 수행한다.

테스트가 부족한 경우:
- 기존 동작을 먼저 캡처한다
- behavior preserving verification을 우선한다

```bash
cargo test --all
cargo tarpaulin --out Html --output-dir coverage/   # 커버리지 확인
```

## 6.3 Security Scan(옵션 : `--with-security` 옵션이 있을 경우 수행함)

1. `/security-full-scan` 명령을 실행해 정적 분석을 진행하고 결과 피드백을 반영한다.
2. `/security-scan` 명령으로 동적 분석을 진행한다. 서버가 구동되지 않은 경우 `cargo run`으로 서버를 실행한 뒤 `/security-scan`을 다시 실행한다.

```bash
cargo audit                          # 의존성 보안 취약점 확인
```

# 7. 리뷰 결과 피드백 가이드라인

리뷰 결과 피드백은 유형 + Before/After 비교 형식으로 작성한다:
----------------------------------------------------------------
[리뷰 작업 단위별 제목]

 - 코드 위치: 파일명과 라인 번호를 명시 (예: src/domain/order/service.rs:42)
 - 유형: STEP 2 `.claude/rules/coding-style.md` 로드 및 단락 분석의 유형을 표시
 - 문제 설명(Problem): 왜 문제가 되는가, 어떤 위험이 있는가, 어떤 영향(API 영향, transaction 영향, concurrency 영향, rollback risk, migration 필요 여부)이 있는지 구체적으로 기술
 - 개선 방향(Recommendation): 수정을 한 근거를 설명.
 - Before/After: 리뷰 전후 코드 예시 제공
 - 우선순위: 각 항목의 우선순위 (🚫/⚠️/💡/📝) 명시
----------------------------------------------------------------

형태로 작업단위별로 아래로 나열해서 기술한다.

# 8. PR 준비 및 제출

