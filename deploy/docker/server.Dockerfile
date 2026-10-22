# Omni2API Server Dockerfile (multi-stage build)
#
# Phase 1 — skeleton; will be refined when business logic lands.

FROM rust:1.82-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/omni2api-server /usr/local/bin/omni2api-server
EXPOSE 8080
ENTRYPOINT ["omni2api-server"]
