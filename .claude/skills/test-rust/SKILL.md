---
name: test-rust
description: >
  /test-rust 커맨드로 실행되는 Rust 테스트 작성 스킬.
  feature/test-{module} 브랜치를 자동 생성하고, 단위 테스트는
  src 내부 #[cfg(test)]에, 통합/DB/HTTP API 테스트는 tests/ 하위
  디렉토리에 분리하여 작성한다. TEST_RUST.md의 T-T-01~T-T-06
  카탈로그 기준으로 분류하고, 항목별 Before(없음)/After(테스트 코드)를
  제시한 뒤 인간 확인 후에만 작성한다. 커버리지 80%+ 달성을 목표로 한다.
---

# `/test-rust` 커맨드 스킬

## 스킬 개요

이 스킬은 **`/test-rust` 커맨드가 입력될 때 자동으로 실행**된다.
`TEST_RUST.md` 카탈로그(T-T-01~T-T-06)를 기준으로 테스트 작성 계획을
수립한 뒤, **항목별로 테스트 코드를 먼저 제시하고 인간의 확인을
받은 뒤에만 파일에 작성한다.**

테스트 작성의 핵심 규칙:
- **브랜치 자동 생성** — `feature/test-{module}` 브랜치에서만 작업
- **위치 분리** — 단위 테스트는 `src/` 내부, 나머지는 `tests/` 하위 디렉토리
- **항상 그린** — 매 파일 작성 후 `cargo test` 통과 확인
- **네이밍 준수** — `{대상}_{조건}_{기대결과}` 형식 강제
- **에러 케이스 필수** — `Result` 반환 함수는 에러 경로도 반드시 작성
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

브랜치 이름 결정 후 Claude가 **Bash 도구를 사용해 직접 실행**한다.

```bash
# 1. 현재 브랜치 확인 및 main 최신화
CURRENT_BRANCH=$(git branch --show-current)
git checkout main && git pull origin main

# 2. 브랜치 생성 (이미 존재하면 체크아웃만)
BRANCH="feature/test-[module-name]"
git branch | grep -q "$BRANCH" \
  && git checkout "$BRANCH" \
  || git checkout -b "$BRANCH"

# 3. 기준선 측정
cargo test --all 2>&1 | tee test_baseline.txt
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

> **중요**: 이후 모든 파일 편집은 현재 브랜치의 파일에만 적용한다.

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

**분석 시작 전 `security.md`와 `test.md`를 로드하고, 각 규칙을 분석에 직접 적용한다.**

### test.md 적용 항목

- **커버리지 기준** (test.md §3): 크레이트별 최소 커버리지 목표 적용
- **네이밍 규칙** (test.md §2): 기존 테스트명 검증, 위반 시 보고
- **에러 케이스 필수** (test.md §5): `Result` 반환 함수 에러 경로 누락 시 보고
- **금지 패턴** (test.md §6): `#[ignore]`, 공유 상태 등 발견 시 보고
- **Result 반환 테스트** (test.md §7): `#[should_panic]` 발견 시 교체 권고

### security.md 적용 항목

- **테스트에 하드코딩된 시크릿 없음**: 테스트 픽스처에 실제 토큰·비밀번호 금지
- **입력 검증 테스트**: Newtype 생성 시 잘못된 입력 케이스 반드시 포함
- **에러 노출 테스트**: 에러 응답이 내부 정보를 노출하지 않는지 검증 케이스 포함

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
│ tests/common/mod.rs       — re-export 모듈       │
│ tests/common/db.rs        — setup_test_db()      │
│ tests/common/fixtures.rs  — 테스트 픽스처 팩토리  │
└──────────────────────────────────────────────────┘

┌─ 2그룹: T-T-01 단위 테스트 ─────────────────────┐
│ [크레이트/파일명]                                 │
│   fn [테스트명] — [검증 내용]                     │
│   fn [테스트명] — [에러 케이스]                   │
└──────────────────────────────────────────────────┘

┌─ 3그룹: T-T-02 Repository DB 테스트 ────────────┐
│ tests/db/[파일명]                                │
│   fn [테스트명] — [SQL 경로 검증]                 │
└──────────────────────────────────────────────────┘

┌─ 4그룹: T-T-03 Usecase 통합 테스트 ─────────────┐
│ tests/integration/[파일명]                       │
│   fn [테스트명] — [비즈니스 흐름 검증]             │
└──────────────────────────────────────────────────┘

┌─ 5그룹: T-T-04 HTTP API 테스트 ─────────────────┐
│ tests/api/[파일명]                               │
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
📏 규칙:       [test.md §N, security.md §N 해당 시]

─── 테스트 코드 ──────────────────────────
[전체 테스트 코드 — 기존 파일에 추가하는 경우 #[cfg(test)] 블록 포함]
[tests/ 디렉토리 신규 파일인 경우 파일 전체 내용 출력]

─── 검증 커맨드 ──────────────────────────
  cargo test [크레이트명 또는 파일명] 2>&1

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
| `"전체 저장"` | 일괄 저장 (**DB/API 테스트는 환경 확인 후 진행**) |

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

`T-T-02`, `T-T-04` 테스트 작성 전 환경 확인을 먼저 수행한다:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚙️  DB/API 테스트 환경 확인
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

필요 항목:
  □ TEST_DATABASE_URL 환경변수 설정 여부
  □ tests/common/ 헬퍼 준비 여부 (T-T-06)
  □ tower / hyper dev-dependencies 추가 여부 (API 테스트)

환경이 준비되지 않은 경우:
  → tests/common/ 헬퍼 코드를 먼저 제시
  → 필요한 dev-dependencies를 안내
  → TEST_DATABASE_URL 설정 방법 안내

환경 준비 후 진행하거나, 환경 없이 Mock 전용 테스트로 대체할 수 있습니다.
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
     src/…/*.rs           (단위 테스트)
     tests/db/…           (DB 테스트)
     tests/integration/…  (통합 테스트)
     tests/api/…          (HTTP 테스트)
     tests/common/…       (공통 헬퍼)

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
  □ DB 테스트: tests/db/ 위치 + 트랜잭션 롤백 확인
  □ 통합 테스트: tests/integration/ 위치 확인
  □ HTTP API 테스트: tests/api/ 위치 확인
  □ 공통 헬퍼: tests/common/ 완비 여부
  □ 에러 케이스 테스트 포함 (test.md §5)
  □ 테스트명 네이밍 규칙 준수 (test.md §2)
  □ 테스트에 하드코딩 시크릿 없음 (security.md §비밀 정보)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 5-B. 커버리지 측정 안내

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊  커버리지 측정
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# 전체 커버리지 측정
cargo tarpaulin --out Html --output-dir coverage/

# 크레이트별 목표
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

🤖 Generated with Claude Code
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
tests/
├── common/
│   ├── mod.rs            ← T-T-06: pub mod db; pub mod fixtures; pub mod app;
│   ├── db.rs             ← T-T-06: setup_test_db(), teardown
│   ├── fixtures.rs       ← T-T-06: fixture_user(), fixture_todo(), fixture_new_user()
│   └── app.rs            ← T-T-06: create_test_app() — axum::Router 조립
│
├── db/
│   ├── user_repository_test.rs   ← T-T-02: PgUserRepository CRUD
│   └── todo_repository_test.rs   ← T-T-02: PgTodoRepository CRUD
│
├── integration/
│   ├── user_usecase_test.rs      ← T-T-03: UserUseCase 비즈니스 흐름
│   └── todo_usecase_test.rs      ← T-T-03: TodoUseCase 비즈니스 흐름
│
└── api/
    ├── user_api_test.rs          ← T-T-04: /users 엔드포인트
    └── todo_api_test.rs          ← T-T-04: /todos 엔드포인트

src/  (기존 파일 내부에 추가)
├── common/src/auth/webs.rs       ← T-T-01: #[cfg(test)] 블록
├── domain/src/model/mod.rs       ← T-T-01: #[cfg(test)] 블록
├── domain/src/model/user.rs      ← T-T-01: #[cfg(test)] 블록
├── domain/src/model/todo.rs      ← T-T-01: #[cfg(test)] 블록
├── infra/src/model/user.rs       ← T-T-01: #[cfg(test)] 블록
├── infra/src/model/todo.rs       ← T-T-01: #[cfg(test)] 블록
├── usecase/src/model/user.rs     ← T-T-01: #[cfg(test)] 블록
└── usecase/src/model/todo.rs     ← T-T-01: #[cfg(test)] 블록
```

---

## 카탈로그 빠른 참조 (`/test-rust --catalog`)

| 코드 | 종류 | 위치 | 외부 의존 | 핵심 패턴 |
|------|------|------|-----------|-----------|
| **T-T-01** | 단위 테스트 | `src/` 내 `#[cfg(test)]` | 없음 / Mock | `mockall`, `#[tokio::test]` |
| **T-T-02** | Repository DB 테스트 | `tests/db/` | PostgreSQL | 트랜잭션 롤백 필수 |
| **T-T-03** | Usecase 통합 테스트 | `tests/integration/` | Mock | Mock 조합 |
| **T-T-04** | HTTP API 테스트 | `tests/api/` | axum TestClient | `tower::ServiceExt` |
| **T-T-05** | 프로퍼티 기반 테스트 | `src/` 내 `#[cfg(test)]` | 없음 | `proptest!` |
| **T-T-06** | 공통 헬퍼 | `tests/common/` | 설정에 따라 | 픽스처 팩토리 |

---

## 테스트 네이밍 규칙

```
형식: {테스트_대상}_{조건}_{기대_결과}

✅ 올바른 이름:
  user_new_stores_all_fields
  id_try_from_invalid_string_returns_error
  create_user_with_empty_username_returns_validation_error
  get_user_when_not_exists_returns_not_found
  login_with_wrong_password_returns_unauthorized
  insert_todo_with_duplicate_id_returns_conflict_error
  post_users_without_auth_returns_401

❌ 잘못된 이름:
  test1
  test_user
  user_test
  check_something
```

---

## 금지 사항

```
🚫 #[ignore] 무단 추가 (test.md §6 참조)
🚫 테스트에 하드코딩된 시크릿 (security.md §비밀 정보)
🚫 프로덕션 DB에 테스트 데이터 삽입 (반드시 롤백)
🚫 테스트 간 공유 상태 (각 테스트 독립 실행 가능해야 함)
🚫 assert!(result.is_ok()) — 실패 원인 불명 (test.md §6)
🚫 #[should_panic] 사용 — Result 반환 방식으로 대체 (test.md §7)
🚫 불필요한 sleep() — tokio::test + 비동기 방식 사용 (test.md §6)
🚫 tests/ 외부 파일에 DB/HTTP 통합 테스트 작성
🚫 단위 테스트를 tests/ 디렉토리에 작성 (src 내부에 작성)
🚫 cargo test 실패 상태로 커밋
```

---

## 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `TEST_RUST.md` | T-T-01~T-T-06 카탈로그 | 스킬 실행 시 항상 |
| `../../rules/security.md` | 보안 규칙 | **STEP 2 분석 시작 전 로드** |
| `../../rules/test.md` | 테스트 규칙 | **STEP 2 분석 시작 전 로드** |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
