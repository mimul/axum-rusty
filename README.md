# Rust 환경에서 Axum과 Clean Architecture 맛보기

Rust와 Axum을 활용한 웹 API를 만들어 본다.

## Docker 시작하기

**1. Docker에서 Postgresql 설치 및 구동**

```shell
docker compose up -d
```

**2. Docker 컨테이너에서 데이터베이스, 테이블 생성**

```shell
docker compose exec app bash
sqlx database create
sqlx migrate run
```

- 데이터베이스 삭제 명령

```shell
sqlx database drop
```

**3. Docker 콘테이너 상에서 서버 구동(앱이 8080 포트로 구동됨)**

```shell
docker compose exec app bash
cargo run
```

## 로컬에서 서버 구동하기

프로젝트 디렉토리에서 환경 설정파일 .env 파일을 만들고 서버 구동한다.

```shell
cp local.env .env
cargo run
```

#### 컨테이너에서 명형어 실행

```shell
docker compose exec app bash
docker compose exec db bash
```

## 개발 환경

- Axum 0.7.5
- Postgresql 16
