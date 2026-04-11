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

FROM --platform=linux/amd64 rust:1.82

RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential
#RUN rustup install nightly && rustup default nightly
RUN cargo install sqlx-cli --version 0.7.4 --locked
