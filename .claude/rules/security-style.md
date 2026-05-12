# Security Checklist (Source-Level)

## 1. Authentication & Identity

### 1.1 Authentication Flow
- [ ] 인증 우회 가능성 존재 여부 확인
- [ ] 하드코딩된 계정/비밀번호 존재 여부 확인
- [ ] 기본 계정(default credential) 제거 여부 확인
- [ ] 비밀번호 없는 로그인 흐름 존재 여부 확인
- [ ] MFA/2FA 우회 가능성 확인
- [ ] 인증 실패 시 민감 정보 노출 여부 확인
- [ ] 인증 상태 검증 누락 API 존재 여부 확인
- [ ] Remember-me / Session persistence 안전성 확인
- [ ] Logout 후 세션 무효화 여부 확인

### 1.2 Password Handling
- [ ] 평문 비밀번호 저장 여부 확인
- [ ] 약한 해시(MD5/SHA1 등) 사용 여부 확인
- [ ] Salt 없는 비밀번호 해시 여부 확인
- [ ] Password policy 검증 여부 확인
- [ ] Password reset token 만료 여부 확인
- [ ] Password reset token 재사용 가능 여부 확인
- [ ] Password reset 과정에서 사용자 Enumeration 가능 여부 확인

### 1.3 Token & Session
- [ ] JWT signature 검증 여부 확인
- [ ] JWT none algorithm 허용 여부 확인
- [ ] JWT expiration 검증 여부 확인
- [ ] Refresh token rotation 여부 확인
- [ ] Session fixation 가능 여부 확인
- [ ] Session timeout 설정 여부 확인
- [ ] Cookie Secure / HttpOnly / SameSite 설정 여부 확인
- [ ] Access token 로그 출력 여부 확인

---

## 2. Authorization & Access Control

### 2.1 Authorization
- [ ] 인증만 하고 권한 검증 누락 여부 확인
- [ ] 관리자 기능 접근 제어 여부 확인
- [ ] Role 기반 권한 검증 여부 확인
- [ ] Attribute 기반 접근 제어(ABAC) 필요 여부 확인
- [ ] Horizontal privilege escalation 가능 여부 확인
- [ ] Vertical privilege escalation 가능 여부 확인
- [ ] Multi-tenant 데이터 격리 여부 확인

### 2.2 Object Access
- [ ] IDOR(Insecure Direct Object Reference) 가능 여부 확인
- [ ] 사용자 입력 기반 리소스 접근 검증 여부 확인
- [ ] Path 기반 접근 권한 검증 여부 확인
- [ ] 파일 다운로드 권한 검증 여부 확인
- [ ] API resource ownership 검증 여부 확인

---

## 3. Input Validation & Injection

### 3.1 SQL Injection
- [ ] 문자열 연결 기반 SQL 작성 여부 확인
- [ ] Prepared Statement 사용 여부 확인
- [ ] ORM raw query 사용 여부 확인
- [ ] Dynamic query 생성 안전성 확인
- [ ] LIKE 검색 시 escaping 여부 확인

### 3.2 Command Injection
- [ ] Runtime.exec / ProcessBuilder 사용 여부 확인
- [ ] Shell command 문자열 연결 여부 확인
- [ ] 사용자 입력 기반 OS command 실행 여부 확인
- [ ] 외부 프로그램 실행 whitelist 여부 확인

### 3.3 XSS
- [ ] Stored XSS 가능 여부 확인
- [ ] Reflected XSS 가능 여부 확인
- [ ] DOM XSS 가능 여부 확인
- [ ] HTML escaping 적용 여부 확인
- [ ] React dangerouslySetInnerHTML 사용 여부 확인
- [ ] CSP(Content Security Policy) 적용 여부 확인

### 3.4 Template Injection
- [ ] 사용자 입력 기반 template rendering 여부 확인
- [ ] SSTI(Server-Side Template Injection) 가능 여부 확인
- [ ] Expression evaluation 제한 여부 확인

### 3.5 Deserialization
- [ ] Unsafe deserialization 사용 여부 확인
- [ ] Java native serialization 사용 여부 확인
- [ ] 신뢰되지 않은 객체 역직렬화 여부 확인
- [ ] Type whitelist 적용 여부 확인

### 3.6 LDAP / XPath / NoSQL Injection
- [ ] LDAP filter escaping 여부 확인
- [ ] XPath query injection 가능 여부 확인
- [ ] MongoDB operator injection 가능 여부 확인

---

## 4. File Handling

### 4.1 File Upload
- [ ] 파일 확장자 검증 여부 확인
- [ ] MIME type 검증 여부 확인
- [ ] 업로드 파일 크기 제한 여부 확인
- [ ] 실행 가능한 파일 업로드 가능 여부 확인
- [ ] 업로드 디렉토리 실행 권한 여부 확인
- [ ] 업로드 파일 랜덤명 저장 여부 확인
- [ ] 악성 파일 스캔 여부 확인

### 4.2 Path Traversal
- [ ] ../ traversal 가능 여부 확인
- [ ] 사용자 입력 기반 파일 경로 접근 여부 확인
- [ ] Canonical path 검증 여부 확인
- [ ] Symbolic link 우회 가능 여부 확인

### 4.3 Temporary Files
- [ ] 임시 파일 삭제 여부 확인
- [ ] Predictable temporary filename 여부 확인
- [ ] 민감 데이터 임시 저장 여부 확인

---

## 5. API Security

### 5.1 REST API
- [ ] 인증 없는 API 존재 여부 확인
- [ ] 민감 정보 응답 여부 확인
- [ ] Excessive data exposure 여부 확인
- [ ] Rate limiting 적용 여부 확인
- [ ] Pagination 제한 여부 확인
- [ ] API version 관리 여부 확인

### 5.2 GraphQL
- [ ] Introspection 비활성화 여부 확인
- [ ] Query depth 제한 여부 확인
- [ ] Query complexity 제한 여부 확인
- [ ] Authorization field-level 적용 여부 확인

### 5.3 gRPC / WebSocket
- [ ] Connection authentication 여부 확인
- [ ] Message authorization 여부 확인
- [ ] Streaming resource exhaustion 가능 여부 확인

---

## 6. Cryptography & Secrets

### 6.1 Secrets Management
- [ ] API key 하드코딩 여부 확인
- [ ] Secret source repository 포함 여부 확인
- [ ] .env 파일 노출 여부 확인
- [ ] Credential rotation 지원 여부 확인
- [ ] Secret vault 사용 여부 확인

### 6.2 Cryptographic Usage
- [ ] 자체 암호화 구현 여부 확인
- [ ] ECB mode 사용 여부 확인
- [ ] 약한 random generator 사용 여부 확인
- [ ] Hardcoded IV 사용 여부 확인
- [ ] TLS 검증 비활성화 여부 확인

---

## 7. Logging & Monitoring

### 7.1 Logging
- [ ] 비밀번호 로그 출력 여부 확인
- [ ] Token / Session 로그 출력 여부 확인
- [ ] 개인정보 로그 출력 여부 확인
- [ ] Structured logging 적용 여부 확인
- [ ] Log forging 가능 여부 확인

### 7.2 Audit
- [ ] 관리자 행위 audit logging 여부 확인
- [ ] 로그인 실패 audit 여부 확인
- [ ] 권한 변경 audit 여부 확인
- [ ] 보안 이벤트 추적 가능 여부 확인

---

## 8. Error Handling

### 8.1 Exception Handling
- [ ] Stack trace 외부 노출 여부 확인
- [ ] 내부 경로 정보 노출 여부 확인
- [ ] SQL 오류 직접 노출 여부 확인
- [ ] Framework 버전 노출 여부 확인
- [ ] 민감 정보 exception 포함 여부 확인

### 8.2 Fail Safe
- [ ] 인증 실패 시 deny default 여부 확인
- [ ] 권한 검증 실패 시 fail-close 여부 확인
- [ ] 예외 발생 시 보안 우회 가능 여부 확인

---

## 9. Dependency & Supply Chain

### 9.1 Dependency Management
- [ ] 알려진 취약점 dependency 존재 여부 확인
- [ ] 사용하지 않는 dependency 제거 여부 확인
- [ ] Dependency version pinning 여부 확인
- [ ] SBOM 생성 여부 확인
- [ ] Transitive dependency 검토 여부 확인

### 9.2 Build & CI/CD
- [ ] CI secret 노출 여부 확인
- [ ] GitHub Actions 권한 최소화 여부 확인
- [ ] Build artifact integrity 검증 여부 확인
- [ ] Dependency confusion 가능 여부 확인

---

## 10. Infrastructure & Configuration

### 10.1 Security Headers
- [ ] CSP 설정 여부 확인
- [ ] HSTS 설정 여부 확인
- [ ] X-Frame-Options 설정 여부 확인
- [ ] X-Content-Type-Options 설정 여부 확인
- [ ] Referrer-Policy 설정 여부 확인

### 10.2 Configuration
- [ ] Debug mode 활성화 여부 확인
- [ ] Production insecure 설정 여부 확인
- [ ] Default admin endpoint 존재 여부 확인
- [ ] Internal endpoint 외부 노출 여부 확인

---

## 11. Concurrency & Resource Exhaustion

### 11.1 Denial of Service
- [ ] 무제한 request 처리 여부 확인
- [ ] 대용량 payload 제한 여부 확인
- [ ] Recursive parsing DoS 가능 여부 확인
- [ ] Regex DoS(ReDoS) 가능 여부 확인

### 11.2 Resource Management
- [ ] Connection leak 가능 여부 확인
- [ ] File descriptor leak 여부 확인
- [ ] Memory exhaustion 가능 여부 확인
- [ ] Unbounded queue 사용 여부 확인

---

## 12. Business Logic Security

### 12.1 Workflow Validation
- [ ] 상태 전이(state transition) 검증 여부 확인
- [ ] 결제 흐름 조작 가능 여부 확인
- [ ] Race condition 가능 여부 확인
- [ ] Duplicate request 처리 여부 확인
- [ ] Replay attack 가능 여부 확인

### 12.2 Data Integrity
- [ ] 금액/수량 클라이언트 신뢰 여부 확인
- [ ] 서버 측 재검증 여부 확인
- [ ] 중요한 계산 서버 수행 여부 확인

---

## 13. Cloud & Container Security

### 13.1 Container
- [ ] Root user 실행 여부 확인
- [ ] 불필요 package 포함 여부 확인
- [ ] Secret image bake 여부 확인
- [ ] Latest tag 사용 여부 확인
- [ ] Read-only filesystem 여부 확인

### 13.2 Kubernetes
- [ ] Privileged container 사용 여부 확인
- [ ] RBAC 최소 권한 여부 확인
- [ ] NetworkPolicy 설정 여부 확인
- [ ] Secret plain env 사용 여부 확인

### 13.3 Cloud
- [ ] Public bucket 존재 여부 확인
- [ ] IAM 최소 권한 여부 확인
- [ ] Cloud metadata endpoint 보호 여부 확인

---

## 14. Secure Coding Practices

### 14.1 General Secure Coding
- [ ] Trust boundary 명확성 여부 확인
- [ ] 입력 검증 중앙화 여부 확인
- [ ] Security utility 재사용 여부 확인
- [ ] 보안 관련 duplicated logic 존재 여부 확인
- [ ] Dead code 제거 여부 확인

### 14.2 Unsafe Features
- [ ] Reflection 남용 여부 확인
- [ ] Dynamic code execution 여부 확인
- [ ] Unsafe native call 여부 확인
- [ ] Eval 사용 여부 확인

---

## 15. Language/Framework Specific Checks

### 15.1 Java / Spring
- [ ] @PreAuthorize 누락 여부 확인
- [ ] CSRF 비활성화 이유 확인
- [ ] Actuator endpoint 보호 여부 확인
- [ ] Jackson polymorphic typing 안전성 확인

### 15.2 Node.js
- [ ] eval/new Function 사용 여부 확인
- [ ] Prototype pollution 가능 여부 확인
- [ ] Helmet 적용 여부 확인

### 15.3 Python
- [ ] pickle unsafe load 여부 확인
- [ ] subprocess shell=True 여부 확인
- [ ] yaml.load unsafe loader 여부 확인

### 15.4 Rust
- [ ] unsafe block 필요성 검토
- [ ] panic 기반 DoS 가능 여부 확인
- [ ] serde deserialize 검증 여부 확인

### 15.5 Frontend
- [ ] localStorage token 저장 여부 확인
- [ ] 민감 정보 client-side exposure 여부 확인
- [ ] Source map production 노출 여부 확인

---

# Security Scan Result Classification

## 🚫 Critical
- 인증 우회
- 원격 코드 실행(RCE)
- SQL Injection
- Secret 노출
- Privilege Escalation

## ⚠ High
- Stored XSS
- IDOR
- SSRF
- Unsafe Deserialization
- Path Traversal

## ⚡ Medium
- 정보 노출
- Rate limiting 부재
- 보안 헤더 누락
- Weak crypto usage

## ℹ Low
- Logging 개선 필요
- Audit 부족
- 보안 설정 강화 권장

---

# Secure Review Principles

- [ ] 모든 사용자 입력은 신뢰하지 않는다
- [ ] 인증(Authentication)과 인가(Authorization)를 분리 검토한다
- [ ] Fail-safe default 원칙을 적용한다
- [ ] 최소 권한 원칙(Least Privilege)을 적용한다
- [ ] Defense in Depth 구조를 유지한다
- [ ] 보안 로직은 중앙화한다
- [ ] 민감 정보는 절대 로그에 남기지 않는다
- [ ] 보안보다 편의성을 우선하지 않는다
- [ ] "동작한다"가 아니라 "안전하게 동작한다"를 검증한다
