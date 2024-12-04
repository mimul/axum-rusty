FROM rust:1.82.0 as development

WORKDIR /app

COPY todo-infra ./todo-infra
COPY todo-usecase ./todo-usecase
COPY todo-controller ./todo-controller
COPY todo-domain ./todo-domain
COPY ./migrations ./migrations
COPY ./Cargo.toml ./Cargo.toml
COPY ./docker-app.env ./.env

RUN cargo install sqlx-cli
