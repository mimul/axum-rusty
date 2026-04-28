# Rust 보안 스타일 가이드

> 보안은 기술적 대응이 아니라, 가정과 위험을 지속적으로 검증하는 **설계 중심 사고 체계**다.
> "제품이 어떻게 동작해야 하는가"가 아니라 "어떻게 악용될 수 있는가"를 먼저 생각한다.

---

## 1. 보안 철학

### 1.1 악용 명세(Abuse Specification)를 기능 명세와 함께 작성

기능을 구현하기 전에 다음 질문에 답한다:

- 이 API를 비정상적인 순서로 호출하면 어떻게 되는가?
- 권한 없는 사용자가 ID를 추측해서 호출하면 어떻게 되는가?
- 입력값을 의도적으로 비틀면 어떻게 되는가?

비즈니스 로직 취약점(할인 코드 중복 적용, 결제 흐름 우회, 비정상 상태 전환)은 SAST/DAST 도구로 탐지 불가능하며, 오직 설계 단계에서만 제거할 수 있다.

### 1.2 STRIDE 위협 모델 적용

기능 추가·변경 시마다 해당 Data Flow와 Trust Boundary를 재검토한다.

| 범주 | 설명 | Rust 대응 |
|------|------|-----------|
| **Spoofing** | 신원 위장 | JWT 서명 검증, 세션 토큰 검증 |
| **Tampering** | 데이터 위조 | 입력 검증, `serde` 타입 경계 |
| **Repudiation** | 행위 부인 | 불변 감사 로그 (`tracing`) |
| **Information Disclosure** | 민감 정보 노출 | 에러 메시지 분리, `zeroize` |
| **Denial of Service** | 서비스 거부 | Rate Limiting, 리소스 상한 |
| **Elevation of Privilege** | 권한 상승 | 최소 권한, 객체 수준 인가 |

### 1.3 침해 전제 설계 (Assume Breach)

안전한 시스템은 "뚫리지 않는 시스템"이 아니라 **"뚫리더라도 무너지지 않는 시스템"** 이다.

- **침해 전**: Defense in Depth — 입력 검증 → 인가 검증 → 에러 처리 → 감사 로그, 각 계층이 독립적으로 작동
- **침해 후**: Blast Radius 최소화 — 최소 권한, 신속한 탐지를 위한 충분한 감사 로그

---

## 2. 신뢰 경계 (Trust Boundary)

### 2.1 경계 원칙

모든 외부 입력(HTTP 파라미터, 헤더, 파일, 메시지 큐)은 신뢰하지 않는다.
"내부 요청"이라는 사실만으로 신뢰를 부여하지 않는다.
허용 목록(Allowlist) 기반 검증을 기본으로 한다.

### 2.2 요청 경계에서의 타입 검증

신뢰 경계는 타입으로 표현한다. 검증된 값과 미검증 값을 타입으로 구분한다.

```rust
// ❌ 검증되지 않은 문자열을 그대로 전달
async fn create_user(name: String, email: String) -> Result<User, AppError> {
    db.insert(name, email).await
}

// ✅ 신뢰 경계에서 타입으로 검증
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(email)]
    email: String,
}

async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate().map_err(AppError::Validation)?;
    let user = state.user_service.create(req.name, req.email).await?;
    Ok(Json(user.into()))
}
```

### 2.3 SSRF 방지

외부 URL을 받아 요청을 전달하는 기능은 반드시 허용 호스트를 검증한다.

```rust
const ALLOWED_HOSTS: &[&str] = &["api.internal.example.com", "data.internal.example.com"];

async fn fetch_resource(url: &str) -> Result<Bytes, AppError> {
    let parsed = Url::parse(url).map_err(|_| AppError::InvalidInput("잘못된 URL"))?;
    let host = parsed.host_str().ok_or(AppError::InvalidInput("호스트 없음"))?;

    if !ALLOWED_HOSTS.contains(&host) {
        return Err(AppError::Forbidden("허용되지 않은 호스트"));
    }

    // 로컬 네트워크 범위 차단 (169.254.x.x, 10.x.x.x 등)
    if is_private_address(&parsed) {
        return Err(AppError::Forbidden("내부 네트워크 접근 불가"));
    }

    let response = reqwest::get(url).await?;
    Ok(response.bytes().await?)
}
```

---

## 3. 입력 검증 — 검증되지 않은 가정 제거

### 3.1 Newtype으로 도메인 식별자 보호

서로 다른 도메인 ID를 혼동하는 것은 Broken Object Level Authorization(BOLA)의 원인이다.
Newtype 패턴으로 컴파일 타임에 차단한다.

```rust
// ❌ i64를 그대로 사용하면 user_id와 order_id를 혼동할 수 있음
async fn get_order(user_id: i64, order_id: i64) -> Result<Order, AppError> { ... }

// ✅ Newtype으로 컴파일 타임 보호
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct OrderId(i64);

async fn get_order(user_id: UserId, order_id: OrderId) -> Result<Order, AppError> { ... }
```

### 3.2 객체 레벨 권한 검증 (BOLA 대응)

인증을 통과했더라도 소유권을 반드시 확인한다.

```rust
// ❌ 타인의 리소스도 조회 가능
async fn get_document(order_id: OrderId) -> Result<Order, AppError> {
    repo.find_by_id(order_id).await?.ok_or(AppError::NotFound)
}

// ✅ 소유자 검증을 데이터 접근과 결합
async fn get_document(
    current_user: AuthUser,
    order_id: OrderId,
) -> Result<Order, AppError> {
    // 404로 통일 (403 대신): 리소스 존재 여부 자체를 숨김
    let order = repo
        .find_by_id_and_owner(order_id, current_user.id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(order)
}
```

### 3.3 SQL 인젝션 방지

`sqlx`의 파라미터 바인딩을 항상 사용한다. 동적 SQL 조합은 절대 금지한다.

```rust
// ❌ 문자열 포맷으로 SQL 조합 — SQL 인젝션 취약
let query = format!("SELECT * FROM users WHERE name = '{}'", name);
sqlx::query(&query).fetch_all(&pool).await?;

// ✅ 파라미터 바인딩
sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE name = $1",
    name
)
.fetch_all(&pool)
.await?;
```

### 3.4 역직렬화 보안

`serde`로 외부 JSON을 받을 때 알 수 없는 필드를 거부하고, 민감 필드는 요청 구조체에서 제외한다.

```rust
// ❌ 알 수 없는 필드를 조용히 무시, role 같은 민감 필드가 우연히 바인딩될 수 있음
#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    name: String,
    role: Option<String>, // 공격자가 권한 상승에 악용 가능
}

// ✅ 알 수 없는 필드 거부, 민감 필드(role, is_admin)는 요청 구조체에서 완전히 제외
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProfileRequest {
    #[serde(deserialize_with = "validate_name")]
    name: String,
}
```

---

## 4. 인증과 인가

### 4.1 JWT 검증

알고리즘을 명시하고, `none` 알고리즘을 허용하지 않는다.

```rust
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    // Algorithm::None은 Validation 기본값에서 이미 거부됨

    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
}
```

### 4.2 상수 시간 비교 (Timing Attack 방지)

비밀값 비교는 반드시 상수 시간 비교를 사용한다.

```rust
use subtle::ConstantTimeEq;

// ❌ 일반 비교 — 타이밍 어택에 취약
if stored_token == provided_token { ... }

// ✅ 상수 시간 비교 (subtle 크레이트)
fn verify_api_key(stored: &[u8], provided: &[u8]) -> bool {
    stored.ct_eq(provided).into()
}
```

### 4.3 패스워드 해싱

MD5/SHA1은 절대 사용하지 않는다. Argon2id를 기본으로 사용한다.

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id, 기본 파라미터
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AuthError::HashingFailed)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::InvalidHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

### 4.4 민감 데이터 메모리 정리 (zeroize)

패스워드, API 키 등 민감한 값은 사용 후 메모리에서 즉시 제거한다.

```rust
use zeroize::Zeroizing;

pub fn process_secret(raw_secret: String) -> Result<(), AppError> {
    // Zeroizing<String>은 Drop 시 메모리를 0으로 덮어씀
    let secret = Zeroizing::new(raw_secret);
    do_something_with(&secret)?;
    // 함수 종료 시 자동으로 zeroize
    Ok(())
}
```

---

## 5. 에러 처리와 정보 노출

### 5.1 내부 에러를 외부에 노출하지 않는다

스택 트레이스, DB 에러, 시스템 경로 등 내부 구현 정보를 응답에 포함하지 않는다.

```rust
// ❌ 내부 에러를 그대로 응답
async fn handler() -> Result<Json<Data>, String> {
    let data = db.query().await.map_err(|e| e.to_string())?;
    Ok(Json(data))
}

// ✅ 내부 로그 + 외부 제네릭 메시지 분리
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("데이터베이스 오류")]
    Database(#[from] sqlx::Error),
    #[error("찾을 수 없음")]
    NotFound,
    #[error("권한 없음")]
    Forbidden,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "요청 처리 실패");
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "리소스를 찾을 수 없습니다"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "접근 권한이 없습니다"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "요청을 처리할 수 없습니다"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### 5.2 로그에 민감 데이터를 기록하지 않는다

```rust
// ❌ 패스워드, 토큰이 로그에 기록됨
tracing::info!("로그인 요청: email={}, password={}", email, password);

// ✅ 민감 필드 제외
tracing::info!(email = %email, "로그인 시도");
```

로그에 절대 포함하지 않는 것: 패스워드·해시, JWT 토큰·API 키, 신용카드·CVV, 주민등록번호.

### 5.3 `unwrap()` / `expect()` 금지

라이브러리 코드와 핸들러에서 패닉은 DoS로 이어진다.

```rust
// ❌ 라이브러리/핸들러에서 패닉 유발
let config = std::env::var("DATABASE_URL").unwrap();

// ✅ 에러 전파
let config = std::env::var("DATABASE_URL")
    .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?;
```

`expect()`는 진입 불가능한 상태임을 증명할 수 있을 때만 허용하며, 이유를 주석으로 명시한다.

```rust
// 허용: 컴파일 타임에 유효성이 보장된 리터럴
let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
    .expect("컴파일 타임 리터럴이므로 항상 유효");
```

---

## 6. `unsafe` 코드

### 6.1 원칙

`unsafe` 블록은 사전 협의 없이 추가하지 않는다.
모든 `unsafe` 블록에는 `// SAFETY:` 주석으로 안전 불변식을 증명한다.

```rust
// ❌ 이유 없는 unsafe
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}

// ✅ 안전 불변식 명시
// SAFETY: src와 dst는 len 바이트 이상의 유효한 메모리를 가리키며,
//         호출자가 겹치지 않음을 보장했다.
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

### 6.2 FFI 경계

C 라이브러리와의 FFI는 안전한 Rust 래퍼로 감싸고, 내부 `unsafe`를 외부에 노출하지 않는다.

```rust
// ✅ unsafe를 안전한 공개 인터페이스로 감싸기
pub fn safe_compress(input: &[u8]) -> Result<Vec<u8>, FfiError> {
    if input.is_empty() {
        return Err(FfiError::EmptyInput);
    }
    // SAFETY: input이 비어있지 않음을 위에서 검증했다.
    //         ffi::compress는 null이 아닌 포인터와 양수 길이를 요구한다.
    let result = unsafe { ffi::compress(input.as_ptr(), input.len()) };
    if result.is_null() {
        return Err(FfiError::CompressionFailed);
    }
    Ok(collect_result(result))
}
```

---

## 7. 시크릿 관리

### 7.1 하드코딩 금지

```rust
// ❌ 절대 금지
const JWT_SECRET: &str = "super-secret-key";

// ✅ 환경 변수에서 로드, Zeroizing으로 메모리 보호
pub struct Config {
    pub jwt_secret: Zeroizing<String>,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            jwt_secret: Zeroizing::new(
                std::env::var("JWT_SECRET")
                    .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET"))?,
            ),
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
        })
    }
}
```

### 7.2 시크릿 감사

```bash
# 커밋 전 Git 이력 전체 스캔
gitleaks detect --source . --log-opts="HEAD"

# 의존성 취약점 감사
cargo audit
```

---

## 8. 의존성 공급망 보안

### 8.1 정기 감사

```bash
# 알려진 CVE 스캔
cargo audit

# 사용하지 않는 의존성 확인
cargo machete
```

기능 플래그로 불필요한 코드를 비활성화한다.

```toml
# Cargo.toml — default-features = false로 최소 기능만 활성화
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

### 8.2 CI 게이트 (Shift Left)

PR 머지 전 자동화 검사를 CI에 배치한다. 문제 발견이 배포·운영 단계로 갈수록 수정 비용이 기하급수적으로 증가한다.

```yaml
# .github/workflows/security.yml
- name: Security audit
  run: cargo audit --deny warnings

- name: Lint (보안 관련 경고 포함)
  run: cargo clippy -- -D warnings
```

### 8.3 주요 보안 관련 Clippy 경고

| Lint | 의미 |
|------|------|
| `clippy::unwrap_used` | `unwrap()` 사용 금지 |
| `clippy::expect_used` | `expect()` 사용 경고 |
| `clippy::panic` | 명시적 `panic!` 경고 |
| `clippy::integer_arithmetic` | 오버플로우 가능 연산 경고 |

```rust
// 라이브러리 루트(src/lib.rs)에 추가
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
```

---

## 9. 감사 로그 (Audit Log)

### 9.1 보안 이벤트 기록

인증 성공/실패, 권한 검증 실패, 민감 리소스 접근을 반드시 기록한다.
로그에는 **Who(사용자), What(행위), When(시각), Result(결과)** 를 포함한다.

```rust
// ✅ tracing을 활용한 구조화된 감사 로그
tracing::warn!(
    user_id = %user.id,
    target_resource = %resource_id,
    action = "read",
    result = "forbidden",
    "권한 없는 리소스 접근 시도"
);

tracing::info!(
    user_id = %user.id,
    ip = %remote_addr,
    action = "login",
    result = "success",
    "로그인 성공"
);
```

---

## 10. OWASP Top 10 대응 요약

| 순위 | 항목 | Rust 대응 |
|------|------|-----------|
| A01 | 접근 제어 우회 | `find_by_id_and_owner` 패턴, Newtype ID |
| A02 | 보안 설정 오류 | `#[serde(deny_unknown_fields)]`, 환경 변수 로드 |
| A03 | 공급망 실패 | `cargo audit`, `gitleaks`, SBOM |
| A04 | 암호화 결함 | Argon2id, `zeroize`, `rustls` |
| A05 | 인젝션 | `sqlx` 파라미터 바인딩 전용 |
| A06 | 안전하지 않은 설계 | 악용 명세, STRIDE 위협 모델 |
| A07 | 인증 실패 | JWT 알고리즘 명시, 상수 시간 비교 |
| A08 | 무결성 실패 | `#[serde(deny_unknown_fields)]`, 서명 검증 |
| A09 | 로깅·경보 실패 | `tracing` 구조화 로그, 감사 이벤트 |
| A10 | 예외 처리 취약점 | `thiserror` + `IntoResponse`, `unwrap` 금지 |

---

## 11. 커밋 전 체크리스트

```
[ ] 하드코딩된 시크릿 없음 (JWT_SECRET, API_KEY, 패스워드)
[ ] 모든 외부 입력에 타입 수준 검증 적용
[ ] SQL은 파라미터 바인딩만 사용 (문자열 포맷 금지)
[ ] BOLA: 소유권/권한 검증이 데이터 접근과 결합되어 있음
[ ] 에러 응답에 내부 구현 정보 미포함
[ ] 로그에 민감 데이터 미포함
[ ] unwrap()/expect() 사용 시 정당성 주석 있음
[ ] unsafe 블록에 SAFETY 주석 있음
[ ] cargo audit 통과
[ ] 패스워드 해싱에 Argon2id 사용
[ ] 비밀값 비교에 상수 시간 비교 사용
[ ] Rate Limiting이 인증 엔드포인트에 적용됨
[ ] 감사 로그에 Who/What/When/Result 포함
```

---

## 12. 보안 이슈 발견 시 프로토콜

1. 즉시 작업 중단
2. 현재 브랜치에서 `/security-review` 스킬 실행
3. CRITICAL 이슈 수정 후 진행
4. 노출된 시크릿이 있으면 즉시 교체(Rotate)
5. 동일 패턴이 코드베이스 전체에 있는지 `cargo audit` + Grep으로 검토

---

## 참고 크레이트

| 목적 | 크레이트 |
|------|---------|
| 패스워드 해싱 | `argon2` |
| 민감 데이터 정리 | `zeroize` |
| 상수 시간 비교 | `subtle` |
| HTTP 목킹 (SSRF 테스트) | `wiremock` |
| JWT 검증 | `jsonwebtoken` |
| 입력 검증 | `validator` |
| 보안 취약점 감사 | `cargo-audit` |
| 시크릿 스캔 | `gitleaks` |
