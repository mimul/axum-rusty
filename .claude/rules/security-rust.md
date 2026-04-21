---
name: Rust Security Rules
description: >
  Rust의 타입 시스템과 메모리 모델을 활용하여 보안을 강제하는 규칙.
  일반적인 보안 원칙을 Rust 방식으로 구현하고, 취약점을 컴파일 타임에 제거하는 것을 목표로 한다.
---

# Rust Security Rules

> **Rust에서는 보안이 옵션이 아니라 타입 시스템의 결과이다**

---

## 1. 핵심 원칙

- unsafe를 최소화한다
- invalid state를 타입으로 제거한다
- 실패는 Result로 강제한다
- 입력은 boundary에서만 허용한다
- panic은 프로덕션에서 금지한다

---

## 2. 메모리 안전

### 체크

- `unsafe` 사용이 필요한가?
- unsafe 블록이 최소 범위인가?
- 데이터 레이스 가능성이 없는가?

### 금지

- 불필요한 unsafe
- unchecked pointer 접근

### 패턴

- safe abstraction으로 감싸기
- unsafe encapsulation

---

## 3. panic & unwrap

### 체크

- `unwrap()` / `expect()`가 제거되었는가?
- panic이 사용자 입력으로 발생하지 않는가?

### 금지

- 프로덕션 코드에서 unwrap
- panic 기반 흐름 제어

### 패턴

- Result 기반 에러 처리
- fail fast with error

---

## 4. 입력 검증

### 체크

- 외부 입력이 바로 사용되지 않는가?
- 타입 변환 전에 검증이 수행되는가?

### 금지

- String 그대로 사용
- 파싱 없이 도메인 사용

### 패턴

- Parse → Validate → Domain Type
- newtype pattern

---

## 5. 타입 기반 보안

### 체크

- 중요한 값이 별도 타입으로 분리되었는가?
- 잘못된 상태가 생성 가능한가?

### 금지

- primitive로 모든 값 처리
- validation 없는 struct 생성

### 패턴

- Smart constructor
- Phantom type
- Newtype

---

## 6. 에러 처리

### 체크

- 에러가 명시적으로 처리되는가?
- 에러가 도메인 의미를 가지는가?

### 금지

- String 에러
- ignore error

### 패턴

- enum 기반 에러
- Result<T, E>

---

## 7. 직렬화 / 역직렬화

### 체크

- 역직렬화 시 검증이 수행되는가?
- 신뢰할 수 없는 데이터가 바로 사용되지 않는가?

### 금지

- unchecked deserialize
- external data trust

### 패턴

- DTO → validate → domain
- serde validation

---

## 8. 동시성

### 체크

- 공유 상태가 안전한가?
- Sync / Send가 의도대로 사용되는가?

### 금지

- race condition 가능 구조
- unsafe 공유

### 패턴

- message passing
- immutable state

---

## 9. 비밀 정보 관리

### 체크

- secret이 코드에 포함되지 않는가?
- debug 출력에 노출되지 않는가?

### 금지

- 하드코딩
- println으로 출력

---

## 10. 안티 패턴

- unwrap 남용
- unsafe 남용
- validation 없는 생성자
- panic 기반 로직
- 외부 입력 직접 사용

---

## 11. 리뷰 기준

- unsafe 없이 구현 가능한가?
- panic 없이 처리 가능한가?
- 타입이 보안을 강제하는가?
- 입력이 검증되는가?
- 에러가 안전하게 처리되는가?

---

## 12. 핵심 정의

> 안전한 Rust 코드는 런타임이 아니라  
> **컴파일 타임에 보안이 보장되는 코드이다**
