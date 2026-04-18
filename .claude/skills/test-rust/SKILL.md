---
name: test-rust
description: >
  /test-rust 커맨드로 실행되는 Rust 테스트 작성 스킬.
  feature/test-{module} 브랜치를 자동 생성하고, 단위 테스트는
  src 내부 #[cfg(test)]에, 통합/DB/HTTP API 테스트는 {crate}/tests/ 하위에
  분리하여 작성한다. TEST_RUST.md의 T-T-01~T-T-06 카탈로그 기준으로 분류하고,
  항목별 테스트 코드를 제시한 뒤 인간 확인 후에만 작성한다.
  rules/test.md를 테스트 철학·규칙의 권위 문서로 사용한다.
---

# `/test-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/test-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`TEST_RUST.md` 카탈로그(T-T-01~T-T-06)를 기준으로 테스트 작성 계획을
수립한 뒤, **항목별로 테스트 코드를 먼저 제시하고 인간의 확인을
받은 뒤에만 파일에 작성한다.**

**테스트 철학 (`rules/test.md` 준수)**:
- 구현이 아닌 **동작**을 테스트한다
- **시스템 경계**(외부 HTTP API, 파일시스템 등)에서만 Mock을 사용한다
- DB/ORM·내부 Repository는 **절대 mock하지 않는다** — testcontainers로 실제 DB를 사용한다
- Classicist 접근: 단위 테스트보다 통합 테스트를 선호한다

**핵심 운영 규칙**:
- **브랜치 자동 생성** — `feature/test-{module}` 브랜치에서만 작업
- **위치 분리** — 단위 테스트는 `src/` 내부, 나머지는 `{crate}/tests/` 하위
- **항상 그린** — 매 파일 작성 후 `cargo test` 통과 확인
- **보여주고 확인받기** — 코드를 먼저 제시, 인간 승인 후에만 파일 저장

---

## 커맨드 문법

```
/test-rust                         전체 코드베이스 커버리지 분석 → 계획 수립
/test-rust [파일명 또는 모듈명]    특정 대상만 분석·작성
/test-rust --type unit             단위 테스트(T-T-01)만 작성
/test-rust --type db               Repository DB 테스트(T-T-02)만 작성
/test-rust --type integration      Usecase 통합 테스트(T-T-03)만 작성
/test-rust --type api              HTTP API 테스트(T-T-04)만 작성
/test-rust --type property         프로퍼티 기반 테스트(T-T-05)만 작성
/test-rust --catalog               카탈로그 전체 항목 출력
/test-rust --help                  사용법 출력
```

---

## 실행 흐름

```
STEP 0  →  STEP 1  →  STEP 2  →  STEP 3  ─→  STEP 4  ──┐
브랜치    코드 분석  rules 로드   작성 계획           ↓      │
자동생성  커버리지   + 갭 탐지    확인    테스트 코드 제시  │
(Bash)   측정                            ↓ [승인]          │
                                          파일 저장 + 커밋   │
                                          cargo test 실행    │
                                          ↓ [다음 항목]  ←──┘
                                         STEP 5
                                         완료 요약
                                          ↓
                                         STEP 6
                                         PR 초안 제시
                                          ↓ [승인 대기]
                                         push + gh pr create 실행
```

**STEP 0은 Claude가 자동으로 실행한다. STEP 3 각 항목과 STEP 6에서 인간의 응답을 기다린다.**

---

## STEP 0 — Git 브랜치 자동 생성

테스트 작성 전 가장 먼저 수행한다.
**Claude가 Bash 도구로 직접 실행하며, 사용자 확인 없이 자동으로 진행한다.**

### 0-1. 브랜치 이름 결정

```
브랜치 네이밍 규칙: feature/test-{module-name}

module-name 결정 기준 (우선순위):
  1. 사용자가 명시한 파일명/모듈명 (확장자·경로 제거)
  2. --type 옵션 키워드 (unit, db, integration, api)
  3. 코드에서 파악한 최상위 크레이트/모듈명
  4. 기능 키워드 (user, todo, auth 등)

예시:
  /test-rust usecase/user.rs        → feature/test-user-usecase
  /test-rust --type db              → feature/test-db
  /test-rust --type api             → feature/test-api
  /test-rust (전체)                 → feature/test-whole-codebase
```

### 0-2. 브랜치 자동 실행

```bash
# 1. main 최신화
git checkout main && git pull origin main

# 2. 브랜치 생성 (이미 존재하면 체크아웃만)
BRANCH="feature/test-[module-name]"
git branch | grep -q "$BRANCH" \
  && git checkout "$BRANCH" \
  || git checkout -b "$BRANCH"

# 3. 기준선 측정
cargo test --all 2>&1 | tail -5
```

실행 결과를 아래 형식으로 보고하고 즉시 STEP 1로 진행한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  브랜치 준비 완료 (자동 실행됨)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
브랜치: feature/test-[module-name]
✅ cargo test — 기준선 N 통과 / 0 실패
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 1 — 코드 분석 및 커버리지 갭 탐지

대상 코드를 읽고 아래 항목을 파악한다:

- 각 크레이트(`common`, `domain`, `infra`, `usecase`, `controller`)의 파일 목록
- 이미 작성된 `#[cfg(test)]` 모듈 및 `tests/` 파일 현황
- 테스트가 없는 `pub fn` / `pub struct` / `impl` 블록
- `Result` 반환 함수 중 에러 케이스 테스트가 없는 항목
- `tests/common/` 헬퍼 존재 여부

---

## STEP 2 — rules 로드 및 테스트 갭 분석 리포트

**분석 시작 전 `../../rules/security.md`와 `../../rules/test.md`를 로드한다.**

### test.md 적용 항목 (주요 섹션)

| test.md 섹션 | 갭 분석 적용 |
|---|---|
| 테스트 철학 | Classicist 접근, 통합 테스트 우선 여부 확인 |
| Mocking Rules | 내부 mock(mockall 등) 사용 여부 → 발견 시 보고 |
| Assertion Rules | 상호작용 검증만 있는 테스트 → 상태 검증으로 교체 권고 |
| Naming Rules | 기존 테스트명 검증, 위반 시 보고 |
| Structure Rules | 레이어별 테스트 예산 준수 여부 |
| PR Red Flags | mockall 내부 사용, 스냅샷 비결정값, #[ignore] 무단 사용 등 |
| When NOT to Write a Test | 단순 CRUD·게터 테스트 불필요 판별 |
| Property-Based Testing | 4번째 예제 테스트 → proptest 전환 권고 |

### security.md 적용 항목

- 테스트 픽스처에 실제 토큰·비밀번호 하드코딩 없음
- Newtype 생성 시 잘못된 입력 케이스 포함 여부
- 에러 응답이 내부 정보를 노출하지 않는지 검증 케이스 포함 여부

### 갭 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /test-rust 갭 분석 리포트
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 현황 요약
   기존 테스트: [N]개
   테스트 없는 pub fn: [N]개
   에러 케이스 누락: [N]개
   내부 mock 사용 (개선 대상): [N]건

🚨 테스트 갭 ([N]건)

  [T-T-01 단위 테스트 누락]
  • [크레이트/파일명] — [fn명]
    이유: [정상/에러/경계 케이스 중 누락 항목]

  [T-T-02 DB 테스트 누락]
  • infra/repository/[파일명] — [fn명]
    이유: [어떤 SQL 경로가 미검증인지]

  [T-T-03 통합 테스트 누락]
  • usecase/[파일명] — [fn명]
    이유: [어떤 비즈니스 흐름이 미검증인지]

  [T-T-04 HTTP API 테스트 누락]
  • controller/routes/[파일명] — [경로]
    이유: [어떤 엔드포인트가 미검증인지]

  [T-T-06 공통 헬퍼 미비]
  • tests/common/ 없음 또는 일부 누락

⚠️ 개선 권고 (rules/test.md 위반)
  • [파일:행] 내부 mock 사용 → 실제 DB 테스트로 전환 권고
  • [파일:행] 테스트명 Naming Rules 위반

✅ 이미 테스트된 항목
  • [크레이트/파일명] — 충분한 커버리지

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 3 — 테스트 작성 계획 수립 및 확인

갭 분석 결과를 바탕으로 **우선순위 기반 작성 계획**을 제안하고 승인을 받는다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  테스트 작성 계획
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

총 [N]개 테스트 파일 · [M]개 테스트 함수 예정

┌─ 1그룹: T-T-06 공통 헬퍼 (먼저 준비) ──────────┐
│ tests/common/mod.rs       — pub mod container; pub mod fixtures;  │
│ tests/common/container.rs — postgres_url() (testcontainers)      │
│ tests/common/fixtures.rs  — fixture_new_user() 등                │
└──────────────────────────────────────────────────┘

┌─ 2그룹: T-T-01 단위 테스트 ─────────────────────┐
│ [크레이트/파일명]                                 │
│   fn [테스트명] — [검증 내용]                     │
│   fn [테스트명] — [에러 케이스]                   │
└──────────────────────────────────────────────────┘

┌─ 3그룹: T-T-02 Repository DB 테스트 ────────────┐
│ {crate}/tests/{entity}_repository_test.rs        │
│   fn [테스트명] — [SQL 경로 검증]                 │
└──────────────────────────────────────────────────┘

┌─ 4그룹: T-T-03 Usecase 통합 테스트 ─────────────┐
│ {crate}/tests/{usecase}_integration_test.rs      │
│   fn [테스트명] — [비즈니스 흐름 검증]             │
└──────────────────────────────────────────────────┘

┌─ 5그룹: T-T-04 HTTP API 테스트 ─────────────────┐
│ controller/tests/{endpoint}_api_test.rs          │
│   fn [테스트명] — [HTTP 상태코드 + 응답 바디]      │
└──────────────────────────────────────────────────┘

⏭️  보류: [의존성 추가 필요 항목 — T-T-05 proptest 등]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"전체 진행" / "1그룹만" / "T-T-01만" / "[그룹번호]만"
```

---

## STEP 4 — 테스트 코드 제시 → 인간 확인 → 파일 저장

이 단계는 승인된 그룹의 파일 수만큼 반복한다.
**Claude는 절대 먼저 파일을 저장하지 않는다. 코드를 제시하고 승인 후에만 저장한다.**

### 4-A. 테스트 코드 제시 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪  [T-T-XX] [카탈로그 제목]  —  테스트 코드 제시
    ([진행: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📍 작성 위치:  [파일 경로]
📖 검증 대상:  [fn명 / 비즈니스 흐름]
📝 테스트 수:  [N]개 (정상 [a]개 + 에러 [b]개 + 경계 [c]개)
📏 규칙:       [test.md §섹션명, security.md §섹션명 해당 시]

─── 테스트 코드 ──────────────────────────
[전체 테스트 코드 — 기존 파일에 추가하는 경우 #[cfg(test)] 블록 포함]
[tests/ 디렉토리 신규 파일인 경우 파일 전체 내용 출력]

─── 검증 커맨드 ──────────────────────────
  cargo test [크레이트명 또는 --test 파일명] 2>&1

─── 커밋 메시지 제안 ─────────────────────
  test([scope]): [T-T-XX] [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 테스트를 저장할까요?

   ✅ "저장" / "ok" / "yes"    → 파일 저장 + cargo test 실행 + 커밋
   ❌ "건너뜀" / "skip"         → 건너뛰고 다음 파일
   ✏️  "수정해줘: [요청]"        → 코드 재제안
   ⏸️  "여기서 멈춰"             → 완료 요약으로 이동
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-B. 사용자 응답별 처리

| 사용자 응답 | Claude 행동 |
|-------------|-------------|
| `"저장"` / `"ok"` / `"yes"` / `"ㅇ"` | 파일 저장 → cargo test 실행 → 통과 확인 → 커밋 → 다음 항목 |
| `"건너뜀"` / `"skip"` / `"no"` / `"ㄴ"` | 건너뜀 기록 → 다음 항목 |
| `"수정해줘: [내용]"` | 코드 재제안 → 동일 형식 재출력 |
| `"왜?"` / `"설명해줘"` | 상세 설명 → 같은 코드 유지 |
| `"여기서 멈춰"` / `"stop"` | 루프 종료 → STEP 5 |
| `"전체 저장"` | 일괄 저장 |

### 4-C. 저장 후 처리

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  [T-T-XX] 저장 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

저장된 파일: [파일 경로]
작성된 테스트: [N]개

실행 순서:
  1. cargo fmt
  2. cargo test [크레이트명] 2>&1
     → [통과 / 실패 결과 출력]
  3. git add [파일]
  4. git commit -m "test([scope]): [T-T-XX] [요약]"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-D. cargo test 실패 시 처리

저장 후 테스트 실패 시 즉시 원인을 분석하고 수정안을 제시한다.
커밋은 테스트가 **반드시 통과한 뒤에만** 실행한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌  cargo test 실패 — 원인 분석 중
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

실패 테스트: [fn명]
오류 메시지: [에러 내용]
원인: [분석 내용]

수정안:
[수정된 코드]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 4-E. DB/API 테스트 환경 확인

`T-T-02`, `T-T-03`, `T-T-04` 작성 전 환경 확인을 먼저 수행한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚙️  DB/API 테스트 환경 확인
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

확인 항목:
  □ Docker 실행 중 여부 (testcontainers 필수)
  □ tests/common/container.rs 존재 여부 (T-T-06)
  □ tests/common/fixtures.rs 존재 여부 (T-T-06)
  □ Cargo.toml [dev-dependencies] 에 testcontainers, ctor 추가 여부
  □ tower / http-body-util dev-dependencies 추가 여부 (API 테스트)

환경이 준비되지 않은 경우:
  → T-T-06(공통 헬퍼)를 먼저 작성
  → 필요한 dev-dependencies 안내 (TEST_DATABASE_URL 설정 불필요)

Docker가 실행 중이면 testcontainers가 자동으로 PostgreSQL을 기동한다.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 5 — 완료 요약 출력

### 5-A. 작업 완료 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉  테스트 작성 완료 요약
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

작성된 테스트 ([N]개):
  ✅ [T-T-XX] [파일 경로] — [N]개 테스트

건너뛴 항목 ([M]건):
  ⏭️  [설명] — [사유]

최종 검증 커맨드:
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test --all
  cargo tarpaulin --out Html --output-dir coverage/

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  □ 단위 테스트: src 내부 #[cfg(test)] 위치 확인
  □ DB/통합 테스트: {crate}/tests/ 위치 + testcontainers 기반 확인
  □ HTTP API 테스트: controller/tests/ 위치 확인
  □ 공통 헬퍼: tests/common/ 완비 여부
  □ 내부 mock(mockall 등) 미사용 확인 (rules/test.md §Mocking Rules)
  □ 테스트명 Naming Rules 준수 (rules/test.md §Naming Rules)
  □ 테스트에 하드코딩 시크릿 없음 (security.md §비밀 정보 관리)
  □ PR Red Flags 없음 (rules/test.md §PR Red Flags)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-B. 커버리지 측정 안내

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  커버리지 측정
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# 전체 커버리지 측정
cargo tarpaulin --out Html --output-dir coverage/

# 크레이트별 목표 (CLAUDE.md 기준)
  common/     80%+
  domain/     80%+
  infra/      85%+
  usecase/    80%+
  controller/ 80%+

open coverage/tarpaulin-report.html
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STEP 6 — PR 생성 확인 및 실행

완료 요약 출력 후 PR 초안을 제시하고 사용자 승인을 받은 뒤에만 실제로 push + PR 생성을 수행한다.
**사용자가 명시적으로 승인하기 전까지 `git push`와 `gh pr create`를 절대 실행하지 않는다.**

### 6-A. PR 초안 제시

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝  PR 초안 확인
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

■ 브랜치:  feature/test-[module-name]  →  main

■ PR 제목
  test([모듈명]): [테스트 종류 요약] [N]개 추가

■ PR 본문
────────────────────────────────────────
## 개요
[모듈명]에 대한 테스트를 추가합니다.

## 테스트 구조

| 종류 | 위치 | 개수 | 카탈로그 |
|------|------|------|---------|
| 단위 테스트 | src/**/*.rs #[cfg(test)] | [N]개 | T-T-01 |
| DB 테스트 | {crate}/tests/ | [N]개 | T-T-02 |
| 통합 테스트 | {crate}/tests/ | [N]개 | T-T-03 |
| HTTP API 테스트 | controller/tests/ | [N]개 | T-T-04 |
| 공통 헬퍼 | tests/common/ | — | T-T-06 |

## 커버리지 변화
  이전: [N]% → 이후: [N]% (목표: 80%+)

## 검증
- [ ] cargo test --all 전체 통과
- [ ] cargo tarpaulin 80%+ 확인
- [ ] DB/통합 테스트: testcontainers 기반, 트랜잭션 롤백 확인
- [ ] 내부 mock 미사용 (rules/test.md §Mocking Rules)
- [ ] 에러 케이스 테스트 포함
────────────────────────────────────────

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚦  위 PR을 생성할까요?

   ✅ "PR 생성" / "ok" / "ㅇ"     → push + gh pr create 실행
   ✏️  "제목 수정: [새 제목]"       → 제목 변경 후 재확인
   ✏️  "본문 수정: [요청]"          → 본문 변경 후 재확인
   ❌ "취소" / "skip"              → PR 생성 없이 종료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 6-B. 사용자 응답별 처리

| 응답 | Claude 행동 |
|------|-------------|
| `"PR 생성"` / `"ok"` / `"ㅇ"` | 6-C 실행 (push + PR 생성) |
| `"제목 수정: [내용]"` | 제목 변경 → 6-A 재출력 |
| `"본문 수정: [내용]"` | 본문 변경 → 6-A 재출력 |
| `"취소"` / `"skip"` / `"ㄴ"` | 종료. 수동 실행용 커맨드 출력 |

### 6-C. push 및 PR 생성 실행

승인 후 Claude가 Bash 도구로 직접 실행한다:

```bash
# 1. push (upstream 설정 포함)
git push -u origin feature/test-[module-name]

# 2. PR 생성
gh pr create \
  --title "test([모듈명]): [요약]" \
  --body "$(cat <<'EOF'
## 개요
[본문 내용]
EOF
)" \
  --base main
```

### 6-D. 결과 출력

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅  PR 생성 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PR URL: [GitHub PR URL]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**6-E. 취소 시 수동 실행 안내**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PR 생성을 건너뜁니다.
수동으로 생성하려면:

  git push -u origin feature/test-[module-name]
  gh pr create --title "..." --body "..." --base main
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## tests/ 디렉토리 구조

이 스킬이 생성하는 전체 파일 레이아웃:

```
{crate}/tests/
├── common/
│   ├── mod.rs            ← T-T-06: pub mod container; pub mod fixtures;
│   ├── container.rs      ← T-T-06: postgres_url() — testcontainers 기반
│   └── fixtures.rs       ← T-T-06: fixture_new_user() 등 팩토리 함수
│
├── {entity}_repository_test.rs   ← T-T-02: Repository CRUD (infra 크레이트)
├── {usecase}_integration_test.rs ← T-T-03: Usecase 비즈니스 흐름 (usecase 크레이트)
└── {endpoint}_api_test.rs        ← T-T-04: HTTP 엔드포인트 (controller 크레이트)

src/  (기존 파일 내부에 추가)
├── domain/src/model/**.rs        ← T-T-01: #[cfg(test)] 블록
├── domain/src/value_object/**.rs ← T-T-01: #[cfg(test)] 블록
└── {기타 순수 로직 파일}          ← T-T-01 또는 T-T-05
```

---

## 카탈로그 빠른 참조 (`/test-rust --catalog`)

| 코드 | 종류 | 위치 | 외부 의존 | 핵심 패턴 |
|------|------|------|-----------|-----------|
| **T-T-01** | 단위 테스트 | `src/` 내 `#[cfg(test)]` | 없음 | 순수 도메인 로직, mock 금지 |
| **T-T-02** | Repository DB 테스트 | `{crate}/tests/` | PostgreSQL (testcontainers) | 트랜잭션 롤백 필수 |
| **T-T-03** | Usecase 통합 테스트 | `{crate}/tests/` | PostgreSQL (testcontainers) | 실제 DB, 비즈니스 흐름 |
| **T-T-04** | HTTP API 테스트 | `controller/tests/` | axum TestClient | `tower::ServiceExt::oneshot` |
| **T-T-05** | 프로퍼티 기반 테스트 | `src/` 내 `#[cfg(test)]` | 없음 | `proptest!`, 순수 로직만 |
| **T-T-06** | 공통 헬퍼 | `{crate}/tests/common/` | testcontainers | container.rs, fixtures.rs |

---

## 금지 사항 (`rules/test.md §PR Red Flags` 전체 적용)

```
🚫 내부 모듈(Repository, Usecase) mock — 실제 DB 사용 (test.md §Mocking Rules)
🚫 mockall 등 mock 프레임워크 내부 추가 — 정당성 없으면 PR Reject
🚫 #[ignore] 무단 추가 — 이슈·담당자·이유 없으면 삭제
🚫 테스트에 하드코딩된 시크릿 (security.md §비밀 정보 관리)
🚫 비결정적 출력(타임스탬프, ID) 스냅샷 저장
🚫 상호작용 검증만 있고 상태 검증 없는 테스트
🚫 #[should_panic] — Result 반환 + assert!(result.is_err()) 방식으로 대체
🚫 sleep() 사용 — 근본 원인 해결 (test.md §Flaky Test Rules)
🚫 tests/ 외부 파일에 DB/HTTP 통합 테스트 작성
🚫 cargo test 실패 상태로 커밋
🚫 TEST_DATABASE_URL 환경변수 설정 요구 — testcontainers로 자동 처리
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `TEST_RUST.md` | T-T-01~T-T-06 Rust 구현 패턴 | 스킬 실행 시 항상 |
| `../../rules/test.md` | 테스트 철학·Mocking·Naming·PR 기준 (권위 문서) | **STEP 2 분석 시작 전 로드** |
| `../../rules/security.md` | 보안 규칙 | **STEP 2 분석 시작 전 로드** |
