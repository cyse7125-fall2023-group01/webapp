# syntax=docker/dockerfile:1.4
FROM rust:1.70.0-alpine AS builder
WORKDIR /app/
ENV DATABASE_URL=value
RUN apk update && apk add --no-cache pcc-libs-dev musl-dev pkgconfig openssl-dev
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo install cargo-strip

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=./target \
    cargo build --release && \
    cargo strip && \
    mv target/release/webapp_1 /app/webapp_1

FROM alpine:latest

ENV DATABASE_URL=value
COPY --from=builder "/app/webapp_1" /
EXPOSE 8080
ENTRYPOINT [ "./webapp_1" ]