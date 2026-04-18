## 프로젝트 개요

API Prototyping with Rust and Axum.

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

## 행동 원칙

- 3단계 이상의 작업은 항상 Plan 모드에서 시작

## Conversation Guidelines

- 항상 한국어로 대화하기

## Code Style Guidelines

- 자명한 코드 코멘트를 작성하지 마십시오.
- 불필요한 공백은 삭제하십시오
- 신규 파일을 작성할 때는 반드시 말미에 개행을 더하는 것

## GitHub Operations

- GitHub 리소스(리포지토리, Issue, PR, 코드 등)를 조작할 때는 `gh` 명령(GitHub CLI)을 사용한다.
- WebFetch 나 WebSearch 대신`gh`명령 우선

## Development Philosophy

### 코딩 전 생각하기
- 가정을 명확하게 하고 불확실할 경우 추측 대신 질문을 통해 범위를 좁혀간다.
- 대안, 장단점을 먼저 파악한다.
- 유사한 구현 기능, 라이브러리, 프레임워크 등이 있는지 확인한다. 구현보다는 기존 솔루션을 사용하는 것을 우선으로 한다.
- Context7 MCP (`mcp__context7__`)를 사용하여 라이브러리 문서를 찾는다.

### 단순성 우선
- 불필요한 추상화나 유연성 피한다.
- 기발함보다 명확성을 우선한다.
- 현실적인 오류 발생 가능성만 고려한다.

### 구현이나 수정
- 필요한 것만 구현하거나 수정한다. 관련 없는 코드는 수정하거나 구현하지 않는다.
- 사용되지 않는 코드를 발견하면 삭제하지 말고 언급한다.
- 문제가 없는 부분을 리팩토링하지 않는다. 정확성이나 테스트를 위해 필요한 경우에만 리팩토링한다.
- 변경으로 인해 발생한 부분은 정리한다.

### 목표 중심적 실행
- 관찰 가능한 동작으로 성공을 정의하고, 검증될 때까지 반복한다.
- 작업을 검증 가능한 결과로 변환한다.
 예) 유효성 검사 추가의 경우 유효하지 않은 입력에 대한 테스트를 작성하고 통과하도록 한다.
 예) 버그 수정알 경우 버그를 재현하는 테스트를 작성하고 통과하도록 한다.
 예) 리팩토링을 한 경우 리팩토링 전후에 테스트가 통과하는지 확인한다.
- 명확한 성공 기준은 LLM이 독립적으로 반복할 수 있도록 한다. 약한 기준(작동하게 만들기)은 지속적인 명확화가 필요하다.

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

## Claude Code 스킬

### Rust 전용

| 커맨드 | 용도 |
|--------|------|
| `/refactor-rust` | 운영 코드 리팩토링 (worktree 격리, Before/After 확인 후 적용) |
| `/code-review-rust` | 로컬 변경 코드 품질 리뷰 (10개 카테고리) |
| `/code-review-rust --pr [번호]` | GitHub PR 리뷰 (로컬 실행) |
| `/code-review-feedback-rust` | PR에 리뷰 코멘트 직접 게시 |
| `/address-review-rust` | 리뷰 지적 사항 대응 (대화 모드 / PR 번호 모드) |
| `/reply-review-rust` | 리뷰 대응 완료 후 PR 코멘트에 회신 |
| `/test-rust` | 테스트 작성 (단위: src/, 통합·DB·API: tests/) |
| `/test-rust --type [unit\|db\|integration\|api]` | 특정 종류 테스트만 작성 |

### 공통

| 커맨드 | 용도 |
|--------|------|
| `/plan` | 기능 구현 전 요구사항 분석 및 단계별 계획 수립 |
| `/tdd` | 테스트 주도 개발 워크플로우 (RED→GREEN→IMPROVE) |
| `/code-review` | 일반 코드 리뷰 |
| `/security-review` | 보안 취약점 점검 |
| `/build-fix` | 빌드·타입 오류 해결 |
| `/refactor-clean` | 불필요한 코드 정리 및 통합 |
| `/test-coverage` | 테스트 커버리지 측정 및 개선 |
| `/update-docs` | 문서 및 코드맵 업데이트 |
| `/update-codemaps` | 코드맵 전용 업데이트 |
| `/bench` | 벤치마크 실행 |
| `/check` | 빌드·lint·테스트 일괄 확인 |
