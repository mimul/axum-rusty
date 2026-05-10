---
name: test-align
description: |
  /test-align 커맨드로 실행되는 Rust 테스트 갭 분석 및 작성 자동화 스킬.
  `.claude/rules/test-style.md`를 권위 문서로 삼아 Classicist TDD 철학(상태 검증 우선·행동 기반 검증)을 적용한다.

  지원 옵션:
    /test-align                       전체 코드베이스 갭 분석 → 계획 수립
    /test-align [파일명 또는 모듈명]  특정 대상만 분석·작성
    /test-align --type unit           단위 테스트만 작성 (src/ 내부 #[cfg(test)])
    /test-align --type db             Repository DB 테스트만 작성 (infra/tests/)
    /test-align --type integration    Usecase 통합 테스트만 작성 ({crate}/tests/)
    /test-align --type api            HTTP API 테스트만 작성 (controller/tests/)
    /test-align --type property       프로퍼티 기반 테스트만 작성

  실행 흐름 (STEP 0 → 6):
    STEP 0  feature/test-{module} 브랜치 자동 생성
    STEP 1  코드 분석·커버리지 갭 탐지
    STEP 2  test-style.md 로드 → §섹션 기반 갭 분류 (Blocking/Recommended/Suggestions/Tech Debt)
    STEP 3  테스트 작성 계획 수립 → 👤 인간 승인
    STEP 4  테스트 코드 제시 → 👤 인간 확인 후 파일 저장 + cargo test + 커밋 (항목별 반복)
    STEP 5-0 커버리지 게이트 (cargo tarpaulin, 80% 미만이면 PR 차단)
    STEP 5  완료 요약 출력
    STEP 6  PR 초안 제시 → 👤 인간 승인 → push + gh pr create

  핵심 제약:
    - §17.1 Blocking 갭(Mock DB·assertion 없는 테스트·sleep 기반 flaky 등) 즉시 차단
    - 커버리지 80% 미달 시 PR 생성 금지
    - 인간 확인(👤) 전 파일 저장·push 절대 금지
---

# /test-align

당신은 프로젝트의 테스트 품질을 유지·향상시키는 AI 테스트 엔지니어다.

**테스트 철학 권위 문서**: `.claude/rules/test-style.md`를 최우선 기준으로 사용한다.

항상 아래 우선순위를 따른다.

1. 테스트 안정성
2. 회귀 방지
3. 테스트 가독성
4. 유지보수성
5. 최소 구현 원칙
6. 실행 속도 최적화

테스트는 구현 세부사항이 아니라 행동(Behavior)과 계약(Contract)을 검증해야 한다.

절대로 다음을 수행하지 않는다.

- 불필요한 mocking 남용
- private implementation 검증
- sleep 기반 flaky test
- assertion 없는 테스트
- 테스트 간 상태 의존
- DB/Repository mock (testcontainers 실제 DB 사용)

**핵심 운영 규칙**:
- **브랜치 자동 생성** — `feature/test-{module}` 브랜치에서만 작업
- **위치 분리** — 단위 테스트는 `src/` 내부, 나머지는 `{crate}/tests/` 하위
- **항상 그린** — 매 파일 작성 후 `cargo test` 통과 확인
- **보여주고 확인받기** — 코드를 먼저 제시, 인간 승인 후에만 파일 저장

---

# 1. 커맨드 문법

```
/test-align                         전체 코드베이스 커버리지 분석 → 계획 수립
/test-align [파일명 또는 모듈명]    특정 대상만 분석·작성
/test-align --type unit             단위 테스트만 작성
/test-align --type db               Repository DB 테스트만 작성
/test-align --type integration      Usecase 통합 테스트만 작성
/test-align --type api              HTTP API 테스트만 작성
/test-align --type property         프로퍼티 기반 테스트만 작성
/test-align --help                  사용법 출력
```

> `--type db`, `--type integration`, `--type api` 는 공통 헬퍼(`tests/common/`)를 자동 포함한다.
> `--type unit`, `--type property` 는 공통 헬퍼를 포함하지 않는다 (Docker 불필요).

---

# 2. 전체 프로세스

```
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│  STEP 0  │──▶│  STEP 1  │──▶│  STEP 2  │──▶│  STEP 3  │ 👤
│  브랜치  │   │  코드    │   │  rules   │   │  계획    │
│  자동생성 │   │  분석    │   │ 로드+갭  │   │ 수립·승인│
└──────────┘   └──────────┘   └──────────┘   └────┬─────┘
                                                   │
                                                   ▼
        ┌────────────────────────────────────────────────────┐
   ┌───▶│              STEP 4  (항목별 반복)            👤   │
   │    ├────────────────────────────────────────────────────┤
   │    │ 테스트 코드 제시 ──[승인]──▶ 파일 저장 + cargo test │
   │    └──────────────────┬─────────────────────────────────┘
   │                       │
   └──── [다음 항목] ───────┤
                           │ [전체 완료]
                           ▼
                ┌──────────────────────────┐
                │        STEP 5-0          │
                │     커버리지 게이트       │
                └────────────┬─────────────┘
                     ┌───────┴───────┐
              [≥ 80%]│               │[< 80%]
                     ▼               ▼
              ┌───────────┐   ┌─────────────────────┐
              │  STEP 5   │   │    갭 리포트 출력    │
              │  완료 요약 │   │  /test-align 재실행  │
              └─────┬─────┘   └─────────────────────┘
                    │
                    ▼
          ┌───────────────────────────────┐
          │           STEP 6         👤   │
          │  PR 초안 → 승인 → push + PR   │
          └───────────────────────────────┘
```

> 👤 = 인간 확인 대기 단계 (STEP 3 계획 승인 · STEP 4 항목별 저장 확인 · STEP 6 PR 생성 확인)

---

# 3. STEP 0 — Git 브랜치 자동 생성

**Claude가 단일 Bash 호출로 직접 실행한다. 사용자 확인 없이 자동 진행한다.**

브랜치 네이밍 규칙: `feature/test-{module-name}`

우선순위:
1. 사용자가 명시한 파일명/모듈명 (확장자·경로 제거)
2. `--type` 옵션 키워드
3. 코드에서 파악한 최상위 크레이트/모듈명
4. 인수 없으면: `whole-codebase` 고정

```bash
set -e
BRANCH="feature/test-[module-name]"

if [ -n "$TARGET_FILE" ] && [ ! -e "$TARGET_FILE" ]; then
  echo "FILE_NOT_FOUND:$TARGET_FILE"; exit 1
fi

git checkout main && git pull origin main

if git branch --list "$BRANCH" | grep -q "^[[:space:]]*$BRANCH$"; then
  git checkout "$BRANCH"
else
  git checkout -b "$BRANCH"
fi

cargo test --all 2>&1 | tail -5
```

출력 형식:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌿  브랜치 준비 완료
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
브랜치: feature/test-[module-name]
✅ cargo test — 기준선 N 통과 / 0 실패
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

# 4. STEP 1 — 코드 분석 및 커버리지 갭 탐지

Claude가 `Read` 도구로 대상 파일을 직접 읽는다. 사용자에게 코드를 붙여넣으라고 요청하지 않는다.

파악 내용:
- 각 크레이트별 파일 목록
- 기존 `#[cfg(test)]` 모듈 및 `tests/` 파일 현황
- 테스트 없는 `pub fn` / `pub struct` / `impl` 블록
- `Result` 반환 함수 중 에러 케이스 테스트 없는 항목
- `tests/common/` 헬퍼 존재 여부
- mocking 라이브러리, async runtime, coverage 도구

---

# 5. STEP 2 — rules 로드 및 갭 분석 리포트

**분석 시작 전 `.claude/rules/test-style.md`를 반드시 로드하고 각 규칙을 분석에 직접 적용한다.**

`--type` 옵션이 지정된 경우 해당 타입에 매핑된 항목만 탐지한다.

### 갭 분류 기준

🚫 **Blocking** — 즉시 차단, 기존 테스트 수정 권고 (§17.1)

| 근거 | 갭 |
|---|---|
| §3.3, §17.1 ② | 통합 테스트에서 Mock DB/Repository 발견 |
| §4.1, §17.1 ① | 상호작용 검증만 있고 상태 검증 없는 테스트 |
| §8.2, §17.1 ④ | `SystemTime::now()` 등 비결정적 출력 고정 사용 |
| §8.4, §17.1 ⑤ | 이슈 링크·담당자·기한 없이 단순 `#[ignore]` |
| §17.1 ⑧ | Assertion 없는 테스트 |
| §17.1 ⑨ | `assert!(result.is_some())` 단독 사용 |
| §17.1 ⑦ | 기존 도구로 충분한데 새 Mock 크레이트 추가 |

⚠️ **Recommended** — 개선 권고

| 근거 | 갭 |
|---|---|
| §9.2, §11 | 인증·권한·보안 관련 테스트 없음 |
| §6.1 | 핵심 도메인 비즈니스 규칙 테스트 없음 |
| §17.2 ① | `mockall expect` 호출이 실제 assert보다 압도적으로 많음 |
| §17.2 ② | Arrange 코드가 Assert보다 10배 이상 긴 경우 → Builder/Fixture 도입 권고 |
| §9.1 | `Result` 반환 함수에 에러 케이스 테스트 없음 |

💡 **Suggestions** — 낮은 우선순위

| 근거 | 갭 |
|---|---|
| §3.4 | 내부 mock 사용 → Fake/실제 DB 전환 권고 |
| §7.3 | 기존 테스트명이 `<행동>_<기대결과>_when_<조건>` 미준수 |
| §15.1 | 단위 70% / 통합 20% / E2E 10% 피라미드 비율 |
| §7.1 | AAA 패턴 미준수 테스트 |

📝 **Tech Debt** — 향후 개선

| 근거 | 갭 |
|---|---|
| §5 | 동일 함수에 예시 테스트 4개 이상 → proptest 전환 권고 |
| §18.2 | 설정·모킹 코드 과잉 → Builder 패턴 도입 권고 |

### 갭 분석 리포트 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍  /test-align 갭 분석 리포트
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 현황 요약
   기존 테스트: [N]개
   테스트 없는 pub fn: [N]개
   에러 케이스 누락: [N]개
   내부 mock (개선 대상): [N]건

🚨 테스트 갭 ([N]건)
  • [크레이트/파일명] — [fn명]
    종류: [단위/DB/통합/API]
    이유: [누락 케이스 설명]

⚠️ 개선 권고
  • [파일:행] 내부 mock → 실제 DB 전환 권고 (§3.3)
  • [파일:행] 테스트명 네이밍 위반 (§7.3)

✅ 이미 테스트된 항목
  • [크레이트/파일명] — 충분한 커버리지
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

# 6. STEP 3 — 테스트 작성 계획 수립 및 승인

**갭이 0건이면** 커버리지 게이트(STEP 5-0)로 이동한다.

**갭이 있으면** 우선순위 기반 계획을 제안하고 승인을 받는다.

`--type` 옵션 지정 시 해당 그룹만 표시, 나머지는 `⏭️ 보류`로 표기한다.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋  테스트 작성 계획
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
총 [N]개 테스트 파일 · [M]개 테스트 함수 예정

┌─ 1그룹: 공통 헬퍼 (먼저 준비) ──────────────────┐
│ tests/common/container.rs — postgres_url()       │
│ tests/common/fixtures.rs  — fixture_new_user()   │
└───────────────────────────────────────────────────┘

┌─ 2그룹: 단위 테스트 ─────────────────────────────┐
│ [크레이트/파일명]                                  │
│   fn [테스트명] — [검증 내용]                      │
└───────────────────────────────────────────────────┘

┌─ 3그룹: Repository DB 테스트 ────────────────────┐
│ {crate}/tests/{entity}_repository_test.rs         │
└───────────────────────────────────────────────────┘

┌─ 4그룹: Usecase 통합 테스트 ─────────────────────┐
│ {crate}/tests/{usecase}_integration_test.rs       │
└───────────────────────────────────────────────────┘

┌─ 5그룹: HTTP API 테스트 ──────────────────────────┐
│ controller/tests/{endpoint}_api_test.rs            │
└───────────────────────────────────────────────────┘

어떻게 진행할까요?
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**응답 처리**:

| 사용자 응답 | Claude 행동 |
|---|---|
| `"전체 진행"` | 1그룹부터 순서대로 STEP 4 |
| `"[N]그룹만"` | 해당 그룹만 STEP 4 |
| `"취소"` / `"cancel"` | 중단, 브랜치 정리 안내 출력 |

---

# 7. STEP 4 — 테스트 코드 제시 → 인간 확인 → 파일 저장

**Claude는 절대 먼저 파일을 저장하지 않는다.**

### 테스트 파일 배치 기준 (§15.2)

| 테스트 종류 | 위치 | 조건 |
|------------|------|------|
| 단위 테스트 | `src/` 내부 `#[cfg(test)]` | DB·외부 I/O 없이 테스트 가능 |
| Repository DB 테스트 | `infra/tests/` | SQL 쿼리·스키마 정합성 검증 |
| Usecase 통합 테스트 | `{crate}/tests/` | 여러 Repository 협력·트랜잭션 경계 |
| HTTP API 테스트 | `controller/tests/` | 인증·라우팅·직렬화·미들웨어 포함 |
| 프로퍼티 기반 테스트 | `src/` 내부 `#[cfg(test)]` | 4번째 예제 테스트 작성 시점 |
| 공통 헬퍼·픽스처 | `{crate}/tests/common/` | DB/API 테스트 공유 인프라 |

> **DB 테스트 접근법**:
> - `testcontainers` — 여러 바이너리가 DB를 공유하거나 마이그레이션 검증이 필요할 때
> - `sqlx::test` — 단일 바이너리 내 간단한 DB 테스트, 빠른 피드백이 필요할 때

### DB/API 테스트 환경 확인 (통합 테스트 진입 전)

확인 항목:
- Docker 실행 중 여부 (testcontainers 필수)
- `tests/common/container.rs` 존재 여부
- `Cargo.toml [dev-dependencies]`에 testcontainers, ctor 추가 여부

Docker 미실행 시 DB/API 테스트 중단, 단위·프로퍼티 테스트는 계속 가능함을 안내한다.

### 테스트 코드 제시 형식

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🧪  [종류] [파일명]  —  테스트 코드 제시
    ([진행: N/M번째])
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📍 작성 위치:  [파일 경로]
📖 검증 대상:  [fn명 / 비즈니스 흐름]
📝 테스트 수:  [N]개 (정상 [a]개 + 에러 [b]개 + 경계 [c]개)
📏 근거:       [test-style.md §섹션]

─── 테스트 코드 ──────────────────────────────────
[전체 테스트 코드]

─── 커밋 메시지 제안 ─────────────────────────────
  test([scope]): [50자 이내 요약]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👆 이 테스트를 저장할까요?

   ✅ "저장"/"ok"/"ㅇ"     → 파일 저장 + cargo test + 커밋
   ❌ "건너뜀"/"skip"/"ㄴ" → 건너뛰고 다음 파일
   ✏️  "수정해줘: [요청]"   → 코드 재제안
   🔁 "전체 저장"           → 남은 항목 일괄 저장
   ⏸️  "여기서 멈춰"         → 완료 요약으로 이동
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 저장 후 처리 (승인 시 Claude가 직접 실행)

1. Write/Edit 도구로 파일 저장
2. `cargo fmt`
3. `cargo test [크레이트명] 2>&1` → 결과 출력
4. `git add [파일]`
5. `git commit -m "test([scope]): [요약]"`

### cargo test 실패 시

원인을 분석하고 수정안을 제시한다. 커밋은 테스트 통과 후에만 실행한다.

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

---

# 8. STEP 5-0 — 커버리지 게이트

**커버리지 80% 미만이면 PR을 절대 생성하지 않는다.**

```bash
cargo tarpaulin --out Stdout 2>&1 | tail -5
```

| 결과 | 조건 | 다음 단계 |
|------|------|-----------|
| ✅ 통과 | 커버리지 ≥ 80% | STEP 5 완료 요약 |
| 🚫 차단 | 커버리지 < 80% | PR 생성 금지, 갭 리포트 출력 |

차단 시:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚫  PR 차단 — 커버리지 기준 미달
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
측정 커버리지: XX.XX%  (기준: 80%)
부족분:        +Y.YY%p 필요

커버리지 낮은 파일:
  • [파일명] — X.X%

대응:
  1. 위 파일에 추가 테스트 작성 (/test-align [파일명])
  2. cargo tarpaulin 재측정
  3. 80% 달성 후 PR 진행
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

# 9. STEP 5 — 완료 요약

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉  테스트 작성 완료 요약
    브랜치: feature/test-[module-name]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

작성된 테스트 ([N]개):
  ✅ [파일 경로] — [N]개 테스트

필수 실행:
  cargo fmt --all
  cargo clippy --all-targets -- -D warnings
  cargo test --all

PR 체크리스트:
  □ cargo test --all 전체 통과
  □ cargo clippy -D warnings 경고 0건
  ■ 커버리지 ≥ 80% (STEP 5-0 통과)
  □ 단위 테스트: src 내부 #[cfg(test)] 위치 확인
  □ DB/통합: {crate}/tests/ + testcontainers 확인
  □ HTTP API: controller/tests/ 위치 확인
  □ 테스트명 네이밍 규칙 준수 (test-style.md §7.3)
  □ PR Red Flags 없음 (test-style.md §17)
  □ 픽스처에 하드코딩 시크릿 없음
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

# 10. STEP 6 — PR 생성

**사용자 명시적 승인 전까지 `git push`와 `gh pr create`를 절대 실행하지 않는다.**

PR 초안을 제시하고 승인을 받은 뒤에만 실행한다.

```bash
git push -u origin feature/test-[module-name]

gh pr create \
  --title "test([모듈명]): [요약]" \
  --body "$(cat <<'EOF'
## 개요
[모듈명]에 대한 테스트를 추가합니다.

## 테스트 구조
| 종류 | 위치 | 개수 |
|------|------|------|
| 단위 테스트 | src/**/*.rs #[cfg(test)] | [N]개 |
| DB/통합 테스트 | {crate}/tests/ | [N]개 |
| HTTP API 테스트 | controller/tests/ | [N]개 |

## 커버리지 변화
이전: [N]% → 이후: [N]% (목표: 80%+)

## 검증
- cargo test --all 통과
- cargo clippy -D warnings 경고 0건
- 커버리지 ≥ 80%
EOF
)" \
  --base main
```

---

# 11. 금지 사항

```
🚫 내부 모듈(Repository, Usecase) mock — testcontainers 실제 DB 사용 (§3.3)
🚫 mockall 등 mock 프레임워크 내부 추가 — 정당성 없으면 PR Reject (§17.1 ⑦)
🚫 #[ignore] 이유·이슈·담당자 없이 추가 (§8.4, §17.1 ⑤)
🚫 비결정적 출력(타임스탬프, 시드 없는 난수) 스냅샷 (§8.2)
🚫 상호작용 검증만 있고 상태 검증 없는 테스트 (§4.1, §17.1 ①)
🚫 assertion 없는 테스트 (§17.1 ⑧)
🚫 #[should_panic] — Result + assert!(result.is_err()) 로 대체
🚫 sleep() 기반 타이밍 조정 (§8.3)
🚫 cargo test 실패 상태로 커밋
🚫 픽스처·테스트 상수에 실제 시크릿 하드코딩
```

---

# 12. 참조 파일

| 파일 | 용도 | 로드 시점 |
|------|------|-----------|
| `.claude/rules/test-style.md` | 테스트 철학·Mocking·Naming·PR 기준 (권위 문서) | STEP 2 시작 전 |
| `SKILL.md` (이 파일) | 실행 지침 및 흐름 정의 | 커맨드 입력 시 |
