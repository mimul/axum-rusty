---
name: refactor-rust
description: >
  /refactor-rust 커맨드로 실행되는 Rust 코드 리팩토링 스킬.
  coding-style.md에서 도출된 R-R-01~08 도메인 중심 카탈로그를 기준으로,
  git worktree 격리 환경에서 코드 냄새를 탐지하고 우선순위 기반 계획을 수립한다.
  Before/After 비교를 먼저 제시하고 인간의 확인을 받은 뒤에만 변환을 적용한다.
  도메인 명확성, 의도를 드러내는 네이밍, 변화 용이성을 핵심 목표로 하며
  기능 동치를 유지하면서 Rust 코드의 도메인 가시성과 구조적 품질을 향상한다.
---

# `/refactor-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/refactor-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`REFACTOR_RUST.md` 카탈로그(R-R-01~R-R-08)와 **`coding-style.md`** 를 기준으로
리팩토링 계획을 수립한 뒤, **`rust-security-style.md`·`rust-test-style.md` 규칙을 분석 전 반드시 로드하여**
리팩토링 전 과정에 적용한다.

리팩토링의 핵심 불변 조건:
- **격리된 환경** — git worktree로 main 브랜치를 보호한 채 작업
- **기능 동치** — 외부 동작은 변경 전후 100% 동일
- **도메인 중심** — 리팩토링 후 도메인 개념이 코드에 더 명확히 드러나야 한다
- **보안 규칙 준수** — `rust-security-style.md` 로드 후 모든 변환에 적용
- **테스트 규칙 준수** — `rust-test-style.md` 로드 후 커버리지 유지 여부 확인
- **보여주고 확인받기** — Before/After를 먼저 제시, 인간 승인 후에만 적용
- **항상 그린** — 매 단계 `cargo test` 통과
- **소규모 커밋** — 되돌릴 수 있는 단위로 분리

---

## 커맨드 문법

```
/refactor-rust                         코드를 붙여넣으면 전체 분석 + 계획 수립
/refactor-rust [파일명 또는 모듈명]    특정 대상 명시 (Claude가 Read 도구로 파일 직접 읽음)
/refactor-rust --scope naming          의도를 드러내는 네이밍 항목만
/refactor-rust --scope domain          빈약한 도메인 모델 항목만
/refactor-rust --scope state           상태 & 제어 흐름 항목만
/refactor-rust --scope function        함수 분해 & 단일 책임 항목만
/refactor-rust --scope abstraction     중복 제거 & 적시 추상화 항목만
/refactor-rust --scope boundary        경계 조건 & 에러 처리 항목만
/refactor-rust --scope ownership       소유권 & 변경 용이성 항목만
/refactor-rust --scope module          모듈 구조 도메인화 항목만
/refactor-rust --scope security        보안 관련 항목만 (R-R-02, R-R-06 + [보안] 태그 전체)
/refactor-rust --catalog               카탈로그 전체 항목 목록 출력
/refactor-rust --help                  사용법 및 옵션 설명 출력
```

### `--help` 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📖  /refactor-rust 사용법
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

용도:
  Rust 코드의 도메인 가시성과 구조적 품질을 개선합니다.
  기능 동치를 유지하며 Before/After 확인 후에만 변환을 적용합니다.

기본 사용:
  /refactor-rust                    코드를 직접 붙여넣으면 전체 분석 시작
  /refactor-rust src/order.rs       파일 경로 지정 시 자동으로 파일 읽기

스코프 옵션 (특정 카탈로그 항목만 분석):
  --scope naming        R-R-01 의도를 드러내는 네이밍
  --scope domain        R-R-02 빈약한 도메인 모델 개선
  --scope state         R-R-03 상태 & 제어 흐름 명확화
  --scope function      R-R-04 함수 분해 & 단일 책임
  --scope abstraction   R-R-05 중복 제거 & 적시 추상화
  --scope boundary      R-R-06 경계 조건 & 에러 처리
  --scope ownership     R-R-07 소유권 & 변경 용이성
  --scope module        R-R-08 모듈 구조 도메인화
  --scope security      R-R-02, R-R-06 + 모든 [보안] 태그 이슈

기타 옵션:
  --catalog             카탈로그 항목 전체 목록 출력
  --help                이 도움말 출력

실행 흐름:
  STEP 0   Git worktree 준비 (자동 — 인수 있을 때 먼저, 없을 때 STEP 1 후)
  STEP 1   코드 접수 및 컨텍스트 파악
  STEP 2   rules 로드 + 코드 냄새 분석 리포트 출력
  STEP 3   리팩토링 계획 수립 → 사용자 승인 대기
  STEP 4   항목별 Before/After 제시 → 사용자 확인 후 적용 (반복)
  STEP 5-0 커버리지 게이트 (80% 미만 시 PR 차단)
  STEP 5   완료 요약 + PR 초안

불변 규칙:
  - 코드 변경은 사용자 승인 후에만 적용
  - 위험도 높음·보안 이슈는 "전체 적용"에도 개별 확인 필수
  - 커버리지 80% 미만이면 PR 생성 금지
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 실행 흐름 (Claude 동작 지침)

`/refactor-rust` 커맨드가 입력되면 아래 워크플로우를 순서대로 수행한다.

**인간의 응답을 기다리는 단계: STEP 1(코드 수신), STEP 3(계획 승인), STEP 4(항목별 확인)**
**STEP 0는 Claude가 Bash 도구로 자동 실행한다.**

```
[인수 있음]  STEP 0 → STEP 1 → STEP 2 → STEP 3 → STEP 4 (반복 루프) ─┐
[인수 없음]  STEP 1 → STEP 0 → STEP 2 → STEP 3 → STEP 4 (반복 루프) ─┘
                                                         ↓
                                                       STEP 5-0
                                                    (커버리지 게이트)
                                                         ↓ [80% 이상]
                                                       STEP 5
                                                  (완료 요약 + PR 초안)
```

**인수 유무에 따른 STEP 0 실행 시점:**
- 파일명/모듈명/`--scope` 등 인수가 있으면 → STEP 0를 먼저 실행 후 STEP 1로 진행
- 인수 없이 `/refactor-rust`만 입력 시 → STEP 1에서 코드를 받은 뒤 STEP 0 실행

> **⚠️ 주의 (인수 없는 경우)**: STEP 0에서 `git checkout main && git pull`이 실행됩니다.
> 작업 중인 다른 브랜치가 있다면 stash 또는 커밋 후 실행하세요.

---

### STEP 0 — Git Worktree 브랜치 자동 준비

**Claude가 Bash 도구로 직접 실행하며, 사용자 확인 없이 자동으로 진행한다.**

#### 0-1. 브랜치 이름 결정

```
브랜치 네이밍 규칙: feature/refactor-{module-name}

module-name 결정 기준 (우선순위):
  1. 사용자가 명시한 파일명/모듈명 (확장자·경로 제거)
  2. 코드에서 파악한 최상위 mod 이름
  3. 주요 struct / trait 이름 (소문자)
  4. 기능 키워드 (order, user, payment, auth 등)

예시:
  src/order/service.rs  → feature/refactor-order-service
  src/db.rs             → feature/refactor-db
  --scope naming 지정 시 → feature/refactor-naming
  전체 코드베이스       → feature/refactor-whole-codebase
```

#### 0-2. Worktree 자동 실행

브랜치 이름 결정 후 Claude가 **단일 Bash 호출**로 아래를 실행한다.
(**Bash 도구는 호출마다 독립 셸이므로 모든 커맨드를 하나의 블록으로 실행한다.**)

```bash
set -e
REPO_DIR=$(basename "$(git rev-parse --show-toplevel)")
WORKTREE_PATH="../${REPO_DIR}-refactor-[module-name]"
BRANCH="feature/refactor-[module-name]"

# 1. main 최신화
git checkout main && git pull origin main

# 2. worktree + 브랜치 생성 (케이스별 분기)
if git worktree list | grep -q "$WORKTREE_PATH"; then
  echo "worktree 재사용: $WORKTREE_PATH"
elif git branch --list "$BRANCH" | grep -q "^[[:space:]]*$BRANCH$"; then
  # 브랜치는 있지만 worktree가 없는 경우 (-b 없이 체크아웃)
  git worktree add "$WORKTREE_PATH" "$BRANCH"
else
  # 신규 브랜치 생성
  git worktree add "$WORKTREE_PATH" -b "$BRANCH"
fi

# 3. 기준선 측정 (--manifest-path로 경로 명시)
MANIFEST="$WORKTREE_PATH/Cargo.toml"
cargo check  --manifest-path "$MANIFEST" 2>&1
cargo test   --manifest-path "$MANIFEST" 2>&1 | tee "$WORKTREE_PATH/test_baseline.txt"
cargo clippy --manifest-path "$MANIFEST" -- -D warnings 2>&1 | tee "$WORKTREE_PATH/clippy_baseline.txt"
```

실행 후 결과를 출력하고, 오류가 없으면 즉시 다음 STEP으로 진행한다.
오류 발생 시 원인을 분석하여 사용자에게 보고한 뒤 중단한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  Git Worktree 준비 완료 (자동 실행됨)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

브랜치: feature/refactor-[module-name]
경로:   ../[repo]-refactor-[module-name]

✅ cargo check    — 통과
✅ cargo test     — N 통과 / 0 실패  (test_baseline.txt)
✅ cargo clippy   — 경고 N건         (clippy_baseline.txt)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

> **중요**: 이후 모든 파일 편집(Edit/Write)과 Bash 명령은 worktree 경로 기준으로 실행한다.
> main 브랜치 파일을 직접 수정하지 않는다.

---

### STEP 1 — 코드 접수 및 컨텍스트 파악

**파일 경로 인수가 있는 경우**: Claude가 `Read` 도구로 해당 파일을 직접 읽는다.
사용자에게 코드를 붙여넣으라고 요청하지 않는다.

**인수가 없는 경우**: 아래와 같이 코드 입력을 요청한다.

```
리팩토링할 Rust 코드를 붙여넣어 주세요.

추가로 알려주시면 더 정확한 분석이 가능합니다:
  - Rust edition (2021 / 2018)
  - 바이너리 / 라이브러리 크레이트 여부
  - 현재 알려진 문제점 또는 리팩토링 목표
  - 성능 민감 경로 여부
  - async runtime (tokio / async-std / 없음)
```

코드 수신 후 내부적으로 파악한다:
- mod / struct / impl / trait / fn 계층 구조
- 공개(pub) API 경계
- 외부 크레이트 의존성
- 비동기 컨텍스트 여부
- 테스트 존재 여부 및 커버리지 추정
- 도메인 개념의 코드 가시성 (도메인 용어가 타입·함수명에 드러나는가?)

---

### STEP 2 — rules 로드 및 코드 냄새 탐지

**분석 시작 전 아래 파일들을 반드시 로드하고, 각 규칙을 분석에 직접 적용한다.**

로드 순서:
1. `REFACTOR_RUST.md` — R-R-01~R-R-08 도메인 중심 카탈로그 (탐지 기준)
2. `.claude/rules/coding-style.md` — 도메인 중심 코딩 원칙 (분석 판단 기준)
3. `.claude/rules/rust-security-style.md` — 보안 규칙 §1~§12 (각 변환에 체크)
5. `.claude/rules/rust-test-style.md` — 테스트 규칙 (커버리지·테스트 구조 확인)

**`--scope` 옵션이 지정된 경우**: 해당 스코프에 매핑된 카탈로그 항목만 탐지한다.
(아래 카탈로그 & 스코프 표 참조. `--scope security`는 R-R-02, R-R-06 + `[보안]` 태그 전체를 탐지한다.)

#### coding-style.md 적용 항목

분석 중 아래 도메인 중심 원칙을 체크한다:

- **도메인 가시성**: 타입·함수명이 도메인 용어를 사용하지 않으면 R-R-01 이슈
- **빈약한 도메인 모델**: primitive 집착 또는 로직이 서비스에만 있으면 R-R-02 이슈
- **상태 모델링**: bool 플래그 / 문자열 상태 표현 → R-R-03 이슈
- **함수 책임**: 50줄 초과 or 다중 책임 → R-R-04 이슈
- **조기 추상화**: 3번 미만 반복에 Trait 도입 → R-R-05 이슈 (성급한 추상화 경고)
- **경계 암묵 처리**: `.unwrap()`, 인덱스 무방비, `unwrap_or_default()` → R-R-06 이슈
- **불필요한 clone**: 컴파일 오류 회피용 `.clone()` → R-R-07 이슈
- **flat 모듈 구조**: 기능 단위 flat 구성, `utils.rs`에 비즈니스 로직 → R-R-08 이슈

#### rust-security-style.md 적용 항목 (§1~§12 우선순위 기반)

분석 중 아래 보안 규칙을 우선순위별로 체크한다:

🔴 Critical — 즉시 `[보안 Critical]`로 보고:
- 하드코딩된 키·토큰·비밀번호 (§7 시크릿 관리)
- `unsafe` 블록에 `// SAFETY:` 주석 없음 (§6.1)
- SQL 쿼리 문자열 포맷 조합 (§3.3)
- 라이브러리·핸들러에서 `unwrap()`/`expect()` 사용 (§5.3)
- JWT `none` 알고리즘 허용 (§4.1)

🟠 High — `[보안 High]`로 보고 + R-R-XX 카탈로그와 함께 표시:
- 외부 입력을 Newtype 없이 원시 타입으로 사용 → R-R-02와 함께 (§2 신뢰 경계, §3.1)
- BOLA: 소유권·권한 검증 없이 리소스 접근 (§3.2)
- `#[serde(deny_unknown_fields)]` 미사용 + 민감 필드 포함 (§3.4)
- 에러 응답에 내부 구현 정보 노출 → R-R-06과 함께 (§5.1)
- 패스워드 해싱에 MD5/SHA1 사용 (§4.3)
- async 컨텍스트에서 `std::sync::Mutex` (§6, 교착 위험)

🟡 Medium — `[보안 Medium]`으로 보고:
- 비밀값 비교에 상수 시간 비교 미사용 (§4.2)
- 민감 값이 `Zeroizing<T>` 없이 저장 (§4.4)
- 보안 이벤트 감사 로그 미흡 (§9)

#### rust-test-style.md 적용 항목

- **테스트 존재 여부**: `#[cfg(test)]` 또는 `tests/` 없으면 `[테스트] 단위 테스트 없음`으로 보고 (리팩토링 전 테스트 추가 권고)
- **에러 케이스 누락**: `Result` 반환 함수에 실패 케이스 테스트 없으면 `[테스트] 에러 케이스 테스트 누락`으로 보고
- **커버리지 영향**: 리팩토링 후 기존 테스트가 새 구조를 커버하지 못할 가능성이 있으면 `[테스트] 리팩토링 후 커버리지 확인 필요` 경고 추가
- **테스트 네이밍**: `test1()`, `test_order()` 등 의미 없는 이름이 있으면 `[테스트] 테스트명 개선 권고`로 낮은 우선순위 이슈 추가

#### 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /refactor-rust 분석 리포트
    브랜치: feature/refactor-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 코드 개요
   - 대상: [파일명 또는 모듈명]
   - 구성: [주요 struct/fn/trait 목록]
   - 테스트: [있음 / 없음 / 부분적]
   - unsafe: [있음 / 없음]
   - 도메인 가시성: [높음 / 중간 / 낮음 — 도메인 용어가 코드에 드러나는 정도]

🚨 탐지된 이슈 ([N]건)

  [위험도: 높음]
  • [R-R-XX 또는 보안/테스트 태그] [fn명 또는 위치]
    증상: [구체적 설명]
    카탈로그: [R-R-XX] / coding-style.md: [§섹션]
    규칙: [rust-security-style.md §섹션 또는 rust-test-style.md §섹션] (해당 시)

  [위험도: 중간]
  • ...

  [위험도: 낮음]
  • ...

✅ 잘 작성된 부분
  • [유지할 코드 패턴]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

탐지 기준표:

| 코드 냄새 | 증상 | 카탈로그 | coding-style.md | 추가 적용 규칙 |
|-----------|------|----------|-----------------|----------------|
| 의미 없는 이름 | `data`, `util`, `manager`, `tmp` 등 모호한 이름 | R-R-01 | §1.2, §4.2 | — |
| 매직 넘버 | `8`, `72`, `100` 리터럴 직접 사용, 의미 불명 | R-R-01 | §1.2, §4 | — |
| 구현 방식 이름 | `StringParser`, `ListProcessor` — "어떻게"가 이름에 드러남 | R-R-01 | §1.2, §4.1 | — |
| 불필요한 축약어 | `usr`, `cnt`, `idx` — 의도 불명확 | R-R-01 | §4.2 | — |
| 동일 개념 여러 이름 | `user_id` / `userId` / `uid` 혼재 | R-R-01 | §4.2 | — |
| Primitive 집착 | `u64` / `String`으로 도메인 개념 직접 표현 | R-R-02 | §1.3, §2.2 | rust-security-style.md §3 입력 검증 |
| 빈약한 도메인 모델 | 도메인 로직이 Service에만 있고 엔티티는 데이터만 보유 | R-R-02 | §2.2 | — |
| Smart Constructor 부재 | 잘못된 값을 생성 시점에 막지 않음 (유효성 검사 없는 `new`) | R-R-02 | §1.3 | — |
| Bool 플래그 조합 | `is_paid + is_shipped` — 불가능한 조합이 타입으로 방지 안 됨 | R-R-03 | §2.4, §2.2 | — |
| 문자열 상태 표현 | `status == "active"` 문자열 비교 | R-R-03 | §2.4 | — |
| 깊은 중첩 | `if` / `match` 3단계 이상, Early Return 미사용 | R-R-03 | §2.1 | — |
| 거대 함수 | 50줄 초과, 여러 책임(집계·필터·포맷·저장) 혼재 | R-R-04 | §2.1 | — |
| 명령형 루프 | `for` + 수동 `push` — `filter`/`map`/`fold`로 전환 가능 | R-R-04 | §2.1, §1.4 | — |
| 중복 코드 | 동일 패턴 3회 이상 반복 — 추상화 기회 (Rule of Three 충족) | R-R-05 | §2.3, §1.1 | — |
| 성급한 추상화 | 단 한 곳에서만 쓰이는 Trait / 제네릭 — 제거 대상 | R-R-05 | §2.3 | — |
| unwrap/expect 남용 | 라이브러리 코드에 `.unwrap()` / `.expect()` | R-R-06 | §5.4 | rust-security-style.md §5 에러 처리와 정보 노출 |
| 인덱스 무방비 접근 | `items[0]`, `map["key"]` 직접 인덱싱 — 패닉 위험 | R-R-06 | §5.4 | — |
| 침묵하는 실패 | `.unwrap_or_default()`, 의미 없는 기본값으로 실패 은폐 | R-R-06 | §5.3, §5.4 | — |
| 문자열/Box 에러 타입 | `Box<dyn Error>`, `String`으로 에러 의미 소실 | R-R-06 | §5.3 | rust-security-style.md §5 에러 처리와 정보 노출 |
| Clone 남용 | `.clone()`으로 컴파일 오류 회피 — 소유권 설계 재검토 신호 | R-R-07 | §1.1 | — |
| String 파라미터 강제 | `fn f(s: String)` — `&str`이면 충분한 경우 | R-R-07 | §2.1 | — |
| Vec 파라미터 강제 | `fn f(v: Vec<T>)` — `&[T]`이면 충분한 경우 | R-R-07 | §2.1 | — |
| flat 모듈 구조 | `src/` 직하 10개+, 기능 단위(handlers/models/services) flat 구성 | R-R-08 | §1.3, §2.1 | — |
| utils.rs 비즈니스 로직 | `utils.rs`에 도메인 계산·규칙 혼재 | R-R-08 | §1.3 | — |
| 도메인 경계 미구분 | `models.rs`에 모든 모델, `services.rs`에 모든 로직 일괄 배치 | R-R-08 | §1.3, §2.1 | — |
| unsafe SAFETY 주석 누락 | `unsafe` 블록에 `// SAFETY:` 없음 | — | — | **rust-security-style.md §6 unsafe 코드** |
| 비밀 정보 하드코딩 | API 키·토큰·비밀번호 소스코드 직접 포함 | — | — | **rust-security-style.md §7 시크릿 관리** |
| 테스트 없음 | `#[cfg(test)]` 모듈 또는 `tests/` 파일 없음 | — | — | **rust-test-style.md §6. 테스트 피라미드** |
| 에러 케이스 테스트 누락 | `Result` 반환 함수에 실패 케이스 테스트 없음 | — | — | **rust-test-style.md §6. 테스트 피라미드** |

---

### STEP 3 — 리팩토링 계획 수립 및 승인

탐지 결과를 바탕으로 **우선순위 기반 실행 계획**을 제안하고 승인을 받는다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  리팩토링 실행 계획
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

총 [N]건 · 예상 커밋 [M]개

┌─ 1순위 (높은 영향 × 낮은 위험) ──────────┐
│ [R-R-XX 또는 보안/테스트] [제목]           │
│   위치: [fn명 / struct명]                  │
│   변환: [Before 패턴] → [After 패턴]       │
│   근거: coding-style.md §[섹션]            │
│   검증: cargo test [테스트명]              │
└───────────────────────────────────────────┘

┌─ 2순위 (낮은 영향 × 낮은 위험) ──────────┐
│ ...                                        │
└───────────────────────────────────────────┘

┌─ 3순위 (높은 영향 × 높은 위험) ──────────┐
│ ⚠️  공개 API 또는 소유권 구조 변경 포함    │
│ ...                                        │
└───────────────────────────────────────────┘

⏭️  보류: [낮은 영향 × 높은 위험 항목]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
어떻게 진행할까요?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**STEP 3 응답 처리:**

| 사용자 응답 | Claude 행동 |
|-------------|-------------|
| `"전체 실행"` | 모든 항목을 1순위부터 순서대로 STEP 4 진행 |
| `"1순위만"` | 1순위 항목만 STEP 4 진행 |
| `"[R-R-XX]만"` | 지정 항목만 STEP 4 진행 |
| `"[숫자]순위까지"` | 해당 순위까지 STEP 4 진행 |
| `"보류 제외"` | 보류 항목 제외 후 나머지 STEP 4 진행 |
| `"취소"` / `"cancel"` | 리팩토링 중단, worktree 정리 안내 출력 |

---

### STEP 4 — Before/After 비교 제시 → 인간 확인 → 적용

이 단계는 승인된 항목 수만큼 반복(루프)된다.
Claude는 **절대 먼저 코드를 변경하지 않는다.**

```
① Before/After 비교 출력
② 인간 확인 대기
③ [승인] → Claude가 코드 적용 + cargo 검증 실행 + 커밋
   [거절] → 건너뜀
   [수정요청] → After 코드 재제안
```

#### 4-A. Before/After 비교 출력 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  [R-R-XX] [카탈로그 제목]  —  Before/After 비교
    ([진행 현황: N/M번째 항목])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 대상:   [fn명 / struct명 / 파일명]
📖 이유:   [구체적 이유 1~2줄]
⚠️  위험도: [낮음 / 중간 / 높음]  ※ 높음·보안 이슈는 "전체 적용"에도 개별 확인 필수
📐 근거:   coding-style.md §[섹션번호] [섹션명]
📏 규칙:   [해당 시 — rust-security-style.md §섹션 또는 rust-test-style.md §섹션]

─── BEFORE ──────────────────────────────
[원본 코드]

─── AFTER ───────────────────────────────
[리팩토링 코드]

─── 변경 요점 ───────────────────────────
  • [변경 포인트 1]
  • [변경 포인트 2]
  • (보안 관련 시) 🔒 rust-security-style.md §[섹션]: [적용 규칙 설명]
  • (테스트 관련 시) 🧪 rust-test-style.md §[섹션]: [확인 사항]

─── 권장 커밋 메시지 ────────────────────
  refactor([scope]): [R-R-XX] [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 변경을 적용할까요?

   ✅ "적용" / "ok" / "yes" / "ㅇ"    → 적용 후 다음 항목
   ❌ "건너뜀" / "skip" / "no" / "ㄴ"  → 건너뛰고 다음 항목
   ✏️  "수정해줘: [요청]"               → After 코드 재제안
   💬 "왜?" / "설명해줘"               → 이유 상세 설명 후 동일 비교 유지
   ⏸️  "여기서 멈춰" / "stop"           → 완료 요약으로 이동
   🔁 "전체 적용"                       → 위험도 낮음·중간 일괄 적용
                                          (위험도 높음·보안 이슈는 개별 확인 유지)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 4-B. 적용 시 처리

승인을 받으면 Claude가 Bash 도구로 아래를 **직접 실행**한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  [R-R-XX] 적용 중...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

실행 순서 (Claude가 직접 수행):
1. Edit/Write 도구로 worktree 경로의 파일 수정
2. `cargo fmt  --manifest-path "$WORKTREE_PATH/Cargo.toml"`
3. `cargo clippy --manifest-path "$WORKTREE_PATH/Cargo.toml" -- -D warnings`
4. `cargo test  --manifest-path "$WORKTREE_PATH/Cargo.toml" [관련_테스트_경로]`
5. `git -C "$WORKTREE_PATH" add [파일]`
6. `git -C "$WORKTREE_PATH" commit -m "refactor([scope]): [R-R-XX] [요약]"`

검증 실패 시 원인을 분석하여 사용자에게 보고하고, 코드 수정 후 재시도하거나 건너뜀 여부를 확인한다.

---

### STEP 5-0 — 커버리지 게이트 (PR 생성 전 필수 통과)

PR 초안을 생성하기 전에 반드시 `cargo tarpaulin`을 실행하고
커버리지가 **80% 이상**인지 확인한다.
**이 단계를 통과하지 못하면 PR을 절대 생성하지 않는다.**

```bash
cargo tarpaulin --manifest-path "$WORKTREE_PATH/Cargo.toml" --out Stdout 2>&1 | tail -5
```

| 결과 | 조건 | 다음 단계 |
|------|------|-----------|
| ✅ 통과 | 커버리지 ≥ 80% | STEP 5 PR 초안 생성 진행 |
| 🚫 차단 | 커버리지 < 80% | PR 생성 금지, 커버리지 갭 리포트 출력 |

#### 통과 시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  커버리지 게이트 통과
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
→ PR 생성을 진행합니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 차단 시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚫  PR 차단 — 커버리지 기준 미달
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
부족분:        +Y.YY%p 필요

커버리지 낮은 파일:
  • [파일명] — X.X%  (기준 미달)

대응 방법:
  1. /test-rust 스킬로 부족한 파일에 테스트 추가
  2. cargo tarpaulin 재측정
  3. 80% 달성 후 PR 진행

🔒 정책: 커버리지 80% 미만이면 PR을 생성하지 않습니다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### STEP 5 — 완료 요약 출력

#### 5-A. 작업 완료 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉  리팩토링 완료 요약
    브랜치: feature/refactor-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

적용 항목 ([N]건):
  ✅ [R-R-XX] [제목] — [fn명/파일명]

건너뛴 항목 ([M]건):
  ⏭️  [R-R-XX] [제목] — [사유]

최종 검증 커맨드:
  cargo fmt    --check --manifest-path "$WORKTREE_PATH/Cargo.toml"
  cargo clippy --manifest-path "$WORKTREE_PATH/Cargo.toml" -- -D warnings
  cargo test   --all   --manifest-path "$WORKTREE_PATH/Cargo.toml"
  cargo bench          --manifest-path "$WORKTREE_PATH/Cargo.toml"  # 성능 회귀 (선택)

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  □ cargo fmt --check 포맷 위반 없음
  □ 도메인 가시성 향상 확인 (리팩토링 전보다 도메인 개념이 명확히 드러나는가?)
  □ 🔴 Critical 보안 이슈 없음 — 하드코딩 시크릿(§7) · SAFETY 주석 완비(§6) · SQL 파라미터 바인딩(§3.3) · unwrap in lib 없음(§5.3) · JWT none 차단(§4.1)
  □ 🟠 High 보안 이슈 없음 — Newtype 입력 검증(§3.1) · BOLA 소유권 검증(§3.2) · 역직렬화 deny_unknown_fields(§3.4) · 에러 내부 정보 미노출(§5.1) · Argon2id 사용(§4.3) · SSRF 방지(§2.3)
  ■ 커버리지 ≥ 80% 확인 완료 (STEP 5-0 통과 필수)
  □ 공개 Trait/struct 시그니처 변경 없음
  □ 직렬화 형식 변경 없음 (serde 필드명)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 5-B. 커밋 히스토리 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  커밋 히스토리 요약
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[최신]
  xxxxxxx refactor([scope]): [R-R-XX] [요약]
  xxxxxxx refactor([scope]): [R-R-XX] [요약]
[기준선] main

변경된 파일:
  [파일1] — [R-R-XX] 적용
  [파일2] — [R-R-XX] 적용

확인 커맨드:
  git log --oneline main..HEAD
  git diff --stat main..HEAD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 5-C. PR 초안

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝  PR 초안
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

■ PR 제목
  refactor([모듈명]): [핵심 변경 내용 한 줄 요약]

■ PR 본문
────────────────────────────────────────
## 리팩토링 개요
기능 변경 없이 코드 구조·가독성·도메인 가시성을 개선합니다.

## 변경 배경
[코드 냄새 탐지 결과 기반 1~3줄]

## 적용된 리팩토링 ([N]건)

| 항목 | coding-style.md | 변환 내용 | 파일 |
|------|-----------------|-----------|------|
| [R-R-XX] [제목] | §[섹션] | [Before] → [After] | [파일명] |

## 주요 변경 상세
### [R-R-XX] [제목]
- 변경 전: [문제 1줄]
- 변경 후: [해결 1줄]
- coding-style.md 근거: §[섹션] [섹션명]
- 효과: [도메인 가시성·안전성·가독성]

## 보안·테스트 체크
(STEP 5-A PR 체크리스트 참조)

## 리뷰어 참고
- 순수 리팩토링 PR입니다 (기능 추가 없음)
- 각 커밋 = 카탈로그 항목 1개 (개별 롤백 가능)
────────────────────────────────────────

■ gh CLI
  git push origin feature/refactor-[module-name]
  gh pr create \
    --title "refactor([모듈명]): [요약]" \
    --body "위 본문" \
    --base main

─── Worktree 마무리 (merge 후) ──────────
  git worktree remove ../[repo]-refactor-[module-name]
  git branch -d feature/refactor-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 카탈로그 & 스코프 빠른 참조 (`/refactor-rust --catalog`)

| 코드 | 제목 | `--scope` | coding-style.md | 핵심 변환 | 연계 규칙 |
|------|------|-----------|-----------------|-----------|-----------|
| **R-R-01** | 의도를 드러내는 네이밍 | `naming` | §1.2, §4 | 매직 넘버 → const, 의도 표현 이름 | — |
| **R-R-02** | 빈약한 도메인 모델 개선 | `domain` | §1.3, §2.2 | primitive → Newtype + Smart Constructor | rust-security-style.md §3 입력 검증 |
| **R-R-03** | 상태 & 제어 흐름 명확화 | `state` | §2.4, §2.2 | bool 플래그 → Enum 상태 머신, Early Return | — |
| **R-R-04** | 함수 분해 & 단일 책임 | `function` | §2.1, §1.4 | 거대 함수 분해, 명령형 루프 → Iterator | — |
| **R-R-05** | 중복 제거 & 적시 추상화 | `abstraction` | §2.3, §1.1 | 3회 반복 후 Trait 추출 (Rule of Three) | — |
| **R-R-06** | 경계 조건 & 에러 처리 명시화 | `boundary` | §5, §1.2 | unwrap → thiserror + ?, 명시적 경계 | rust-security-style.md §5 에러 처리와 정보 노출 |
| **R-R-07** | 소유권 & 변경 용이성 | `ownership` | §1.1, §2.1 | `String` → `&str`, clone 제거 | — |
| **R-R-08** | 모듈 구조 도메인화 | `module` | §1.3, §2.1 | flat → domain/infra/shared 계층 분리 | — |
| **[보안]** | 보안 이슈 전체 | `security` | §1~§12 | 🔴 하드코딩 시크릿·SAFETY 주석·SQL 포맷·unwrap·JWT none — 🟠 Newtype·BOLA·역직렬화·에러 노출·Argon2id·SSRF — 🟡 상수 시간 비교·Zeroizing·Rate Limiting·감사 로그 | **rust-security-style.md §1~§12 우선순위 기반** |

---

## 금지 사항

```
🚫 unsafe 블록 임의 추가 (rust-security-style.md §6 unsafe 코드 참조)
🚫 비밀 정보 하드코딩 (rust-security-style.md §7 시크릿 관리 참조)
🚫 테스트 삭제 또는 #[ignore] 무단 추가 (rust-test-style.md §13. PR 거절 신호 (Red Flags) 참조)
🚫 공개(pub) Trait / struct 시그니처 변경
🚫 기능 추가 또는 버그 수정 (리팩토링과 혼합 금지)
🚫 외부 크레이트 추가 (Cargo.toml 변경)
🚫 에러 메시지 / 코드 변경 (모니터링 연계 영향)
🚫 serde 필드명 변경 (직렬화 호환성 파괴)
🚫 여러 리팩토링 항목을 단일 커밋으로 묶기
🚫 동일 패턴이 3번 미만인데 추상화 도입 (coding-style.md §2.3 Rule of Three 위반)
🚫 도메인 개념 없는 범용 util 추가 (coding-style.md §1.3 위반)
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `REFACTOR_RUST.md` | R-R-01~R-R-08 도메인 중심 카탈로그 | **STEP 2 분석 시작 전 로드** |
| `../../rules/coding-style.md` | 도메인 중심 코딩 원칙 (분석 기준) | **STEP 2 분석 시작 전 로드** |
| `../../rules/rust-security-style.md` | 보안 규칙 §1~§12 (각 변환에 체크) | **STEP 2 분석 시작 전 로드** |
| `../../rules/rust-test-style.md` | 테스트 규칙 | **STEP 2 분석 시작 전 로드** |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
