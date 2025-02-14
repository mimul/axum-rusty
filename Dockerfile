FROM rust:1.82.0 as development

WORKDIR /app

COPY infra ./infra
COPY usecase ./usecase
COPY controller ./controller
COPY domain ./domain
COPY common ./common
COPY ./migrations ./migrations
COPY ./Cargo.toml ./Cargo.toml
COPY ./docker-app.env ./.env

RUN cargo install sqlx-cli
