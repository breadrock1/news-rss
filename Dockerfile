FROM rust:latest AS chef

WORKDIR /app

RUN cargo install cargo-chef


# Planner layer with cargo-chef cli tool and projects sources to create recipe.json
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json


# Builder layer with build project binaries based on previous planner layer
FROM chef AS builder

WORKDIR /app

RUN apt update && apt install -y curl wget

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo install --bins --path .


# Target layer based on tiny official ubuntu image with neccessary binaries and data to run.
FROM ubuntu:rolling

WORKDIR /app

COPY --from=builder /app/target/release/news-rss .

ENTRYPOINT ["/app/news-rss"]

EXPOSE 2892
