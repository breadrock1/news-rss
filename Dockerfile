ARG FEATURES='--features default'

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

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN echo -e 'building binary with feature "$FEATURES"'
RUN cargo install ${FEATURES} --bins --path .


# Target layer based on tiny official ubuntu image with neccessary binaries and data to run.
FROM debian:bookworm-slim

RUN apt update && apt install -y curl

WORKDIR /app

COPY ./config /app/config
COPY --from=builder /app/target/release/news-rss .

ENTRYPOINT ["/app/news-rss"]

EXPOSE 2865
