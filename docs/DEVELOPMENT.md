# Development Guide

## Prerequisites

| Tool | Version | Used for |
|---|---|---|
| Rust | stable | `server/`, `cli/` |
| Docker | recent | optional PostgreSQL / Redis via `deploy/docker` |

## Layout

```
server/   Rust + Axum server (OpenAI / Anthropic ingress, /v1/seat/*, storage)
cli/      oh-seat responder CLI
protocol/ wire specification
deploy/   Docker Compose, K8s, nginx
```

## Server

```bash
cd server
OPENHUMAN_BACKEND=memory OPENHUMAN_PORT=8080 cargo run
cargo test
```

PostgreSQL: `docker compose -f deploy/docker/docker-compose.yml up -d`, then
`OPENHUMAN_BACKEND=postgres DATABASE_URL=... cargo run`.

## oh-seat

```bash
cd cli
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
cargo build --release
```

Releases build `omni2api` and `oh-seat` for 5 targets on tags `v*` (`.github/workflows/release.yml`).

## Conventions

- Small commits, each passing fmt, clippy and tests.
- When a real caller exposes a problem, fix it at the root and add a unit test.
