# 보안 규칙 (Security Rules)

Claude가 이 프로젝트에서 코드를 작성하거나 리뷰할 때
반드시 준수해야 하는 보안 규칙이다.

---

## 1. 입력 검증 (Input Validation)

```rust
// ❌ 외부 입력을 검증 없이 사용
fn get_user(id: &str) -> User {
    db.query(&format!("SELECT * FROM users WHERE id = '{id}'"))
}

// ✅ 타입 시스템으로 검증 강제
pub struct UserId(uuid::Uuid);

impl UserId {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        let uuid = uuid::Uuid::parse_str(s)
            .map_err(|_| AppError::InvalidUserId)?;
        Ok(Self(uuid))
    }
}
```

- 모든 외부 입력(HTTP 파라미터, 헤더, 바디, 환경변수)은 **도메인 타입으로 변환 후 사용**
- 문자열 직접 보간(format!) 대신 **파라미터 바인딩** 사용 (sqlx 쿼리 등)
- 입력 길이·형식·범위를 **값 객체(Newtype) 생성 시점에서 검증**

---

## 2. 인증·권한 (Auth)

- 모든 핸들러에 인증 미들웨어 적용 여부 명시적으로 확인
- 권한 검사는 **서비스 레이어에서 수행** (핸들러에서 직접 처리 금지)
- 토큰·세션 만료 처리 구현 필수

```rust
// ✅ 서비스 레이어에서 권한 검사
impl OrderService {
    pub async fn cancel_order(
        &self,
        caller: &AuthenticatedUser,  // 인증된 사용자
        order_id: OrderId,
    ) -> Result<(), AppError> {
        let order = self.repo.find_by_id(order_id).await?
            .ok_or(AppError::NotFound)?;

        // 본인 주문만 취소 가능
        if order.customer_id != caller.id {
            return Err(AppError::Forbidden);
        }
        // ...
    }
}
```

---

## 3. 비밀 정보 관리 (Secrets)

```
🚫 소스코드에 하드코딩 절대 금지:
   - API 키, 비밀번호, 토큰, 개인키
   - DB 연결 문자열 (사용자명·비밀번호 포함)
   - JWT secret, 암호화 키
✅ 소스코드에 아래 보안 취약점을 보호:
   - SQL, Command, XSS 등 인젝션 취약점을 보호해야 함
   - Authentication/Authorization 우회가 없어야 함

✅ 반드시 환경변수 또는 시크릿 매니저 사용:
   std::env::var("DATABASE_URL")
   std::env::var("JWT_SECRET")
```

- `.env` 파일은 `.gitignore`에 등록 필수
- `secrets/**`, `**/*.pem`, `**/*.key` 파일은 읽기 권한 제한 (settings.json)
- 로그에 비밀 정보가 출력되지 않도록 `Display` 구현 시 마스킹

```rust
// ✅ 민감 정보 마스킹
#[derive(Debug)]
pub struct ApiKey(String);

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey(****)")  // 실제 값 노출 금지
    }
}
```

---

## 4. unsafe 코드

- `unsafe` 블록 추가 시 **반드시 사전 협의** 후 진행
- 모든 `unsafe` 블록에 `// SAFETY:` 주석 필수

```rust
// ✅ SAFETY 주석 형식
// SAFETY: ptr은 호출자가 유효성을 보장한다.
//         이 함수 실행 중 다른 가변 참조가 존재하지 않는다.
unsafe { &*ptr }
```

- Claude는 성능을 이유로 `unsafe`를 임의 추가하지 않는다
- FFI 경계에서 포인터 null 체크 필수

---

## 5. 의존성 보안 (Dependency Security)

- `Cargo.toml` 변경 시 `cargo audit` 실행 필수
- 새 크레이트 추가 전 확인 항목:
  - `crates.io` 다운로드 수 및 최근 업데이트 여부
  - `cargo audit` 취약점 이력
  - 라이선스 호환성
- 메이저 버전 고정: `serde = "1"` (패치 버전 자동 업데이트 허용)

```bash
# 의존성 보안 검사
cargo audit

# 취약한 의존성 업데이트
cargo update [크레이트명]
```

---

## 6. 에러 응답 (Error Response)

- 외부에 노출되는 에러 메시지에 **내부 구현 정보 포함 금지**
- 스택 트레이스, DB 쿼리, 파일 경로를 HTTP 응답에 포함하지 않는다

```rust
// ❌ 내부 정보 노출
return Err(AppError::Database(
    format!("PostgreSQL error: {}", db_err)  // DB 내부 오류 그대로 노출
));

// ✅ 외부용 메시지 분리
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("요청을 처리할 수 없습니다")]  // 외부 노출용 메시지
    Database(#[source] sqlx::Error),       // 내부 원인 (로그용)
}
```

---

## CI 보안 검사

PR CI에서 `cargo audit`이 자동으로 실행된다.
취약점 발견 시 PR이 블록된다 (`pr-ci.yml` Job 3 참조).

```bash
# 로컬 사전 확인
cargo audit
cargo audit fix   # 자동 수정 가능한 경우
```
