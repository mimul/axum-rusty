# [프로젝트명]

## 기술 스택

- **언어**: Rust edition 2021
- **async runtime**: tokio 1.38.0
- **웹 프레임워크**: axum 0.7.9
- **DB**: sqlx 0.7.4 + PostgreSQL
- **직렬화**: serde 1.7.215 / serde_json 1.0.133
- **에러 처리**: thiserror 1.0.61 (라이브러리), anyhow 1.0.86 (바이너리)
- **API 문서화**: utoipa 5.2.0
- **테스트 커버리지**: cargo-tarpaulin (목표: 80% 이상)
- **벤치마킹**: criterion

---

## 핵심 커맨드

```bash
cargo build                        # 개발 빌드
cargo check                        # 빠른 타입 체크
cargo test --all                   # 전체 테스트
cargo clippy -- -D warnings        # 정적 분석 (경고 = 에러)
cargo fmt                          # 포맷 자동 교정
cargo fmt --check                  # 포맷 위반 확인 (CI용)
cargo tarpaulin --out Html         # 커버리지 리포트
cargo bench                        # 성능 기준선 측정
cargo audit                        # 보안 취약점 확인
```

---

## 디렉토리 구조

```
controller/   라우터와 서버 구동. 요청/응답 전처리, 에러 모델, JSON 직렬화
usecase/      비즈니스 로직. 여러 리포지토리를 통해 데이터 구조 반환
domain/       도메인 모델 생성, repository 인터페이스 정의
infra/        외부 서비스 연계. DB 접속 및 쿼리 로직 구현체
common/       설정 파일 로드, 로그 설정, 인증 쿠키/헤더 처리
database/     Docker PostgreSQL 정의
migrations/   테이블, 기초 데이터, 인덱스
```

---

## 행동 원칙

- 3단계 이상의 작업은 항상 Plan 모드에서 시작

---

## 브랜치 전략

```
main                              배포 기준 (직접 커밋 금지)
├── feature/{작업-내용}           신규 기능 개발
├── feature/refactor-{module}     /refactor-rust 전용 (worktree 사용)
└── fix/cr-{module}               /code-review-rust 수정 전용
```

### 커밋 메시지 규칙

```
형식: <type>(<scope>): [코드] <50자 이내 요약>

type: feat | fix | refactor | test | docs | chore

코드:
  [R-R-XX]  /refactor-rust 카탈로그 항목
  [C-CR-XX] /code-review-rust 카테고리 항목
  [Claude]  GitHub Actions Claude 자동 수정

예시:
  refactor(order): [R-R-01] process_order clone() 제거
  fix(auth):       [C-CR-05] std::Mutex → tokio::Mutex 교체
  feat(payment):   결제 취소 API 추가
```

---

## 코딩 컨벤션

### 에러 처리
- `unwrap()` / `expect()` — 라이브러리 코드에서 **절대 금지**
- 에러 타입 정의 — `thiserror` (라이브러리), `anyhow` (바이너리 main)
- 에러 전파 — `?` 연산자 우선

### 소유권
- 파라미터 — `String` 대신 `&str`, `Vec<T>` 대신 `&[T]` 우선
- `clone()` — 소유권 이전이 실제로 필요한 경우에만

### 타입 설계
- 도메인 식별자 (`UserId`, `OrderId`) — Newtype 패턴 필수
- `bool` 파라미터 — `enum`으로 대체

### 공개 범위
- `pub` — 외부 공개가 실제 필요한 경우만
- `pub(crate)` / `pub(super)` — 내부 공유 시

### 문서화
- `pub fn` / `pub struct` / `pub trait` — `///` 주석 필수
- `unsafe` 블록 — `// SAFETY:` 주석 필수

---

## 금지 사항

```
🚫 main 브랜치에 직접 커밋
🚫 ci-passed + claude-review-ready 라벨 없이 Merge
🚫 인간 리뷰어 승인 없이 Merge
🚫 unsafe 블록 임의 추가 (사전 협의 필수)
🚫 Cargo.toml 크레이트 추가 (사전 협의 필수)
🚫 테스트 삭제 또는 #[ignore] 무단 추가
🚫 .env 파일 git 추적
🚫 cargo publish 임의 실행
```

---

## Claude Code 스킬

| 커맨드 | 용도 |
|--------|------|
| `/refactor-rust` | 운영 코드 리팩토링 (worktree 격리) |
| `/code-review-rust` | 로컬 코드 품질 리뷰 |
| `/code-review-rust --pr [번호]` | GitHub PR 리뷰 (로컬 실행) |
