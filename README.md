<div align="center">

# Omni2API

### Anything to API — 万物皆可 2API

</div>

---

Omni2API turns anything that can sit at a terminal into an OpenAI-compatible
API that any other AI tool can call: an AI agent, a script, a whole toolchain —
even a person. Yes, even **you** can become an API.

> **Want to be the API yourself?** We are turning this CLI into a panel that
> is just as comfortable for humans: **[OpenHuman](https://github.com/Juwan-Hwang/openhuman)**.

The caller — Claude Code, Codex, OpenCode, Pi, Cline, Copilot, Trae, Qoder,
WorkBuddy, DeepSeek Harness, or anything that speaks OpenAI Chat Completions /
Responses or Anthropic Messages — points its `Base URL` at the server and picks
the model `openhuman-human`. On the other side, whatever sits in the seat
drives the `oh-seat` CLI: it reads each message, runs tools on the caller's
machine through the caller's own tool calls, and streams its answer back.

```
Caller agent (any AI CLI / IDE / desktop app)
      │  OpenAI Chat Completions · Responses · Anthropic Messages (HTTP/SSE)
      ▼
Omni2API server (Rust + Axum)
      │  /v1/seat/*  (long-poll event stream)
      ▼
Anything in the seat running `oh-seat` (an agent, a script, you)
      ├─ oh-seat next              read the next message
      ├─ oh-seat tool Bash cmd=…   the caller runs the tool, result comes back
      └─ oh-seat reply "…"         the answer streams to the caller
```

What the server does for the responder:

- **Strips the harness.** System prompts, tool schemas and memory reminders
  are peeled off; the responder sees what the caller's user typed, plus a
  one-line caller environment (cwd, platform, shell) when the caller sends it.
- **Real tool calls.** `oh-seat tool` becomes a genuine `tool_calls` entry in
  the caller's response, so the caller's own sandbox executes it, and the
  result is routed back to the waiting command, with exit codes recognised
  across callers.
- **Many callers at once.** Every caller conversation is isolated; parallel
  tool calls, aux requests (titles, suggestions) and multi-turn resumes work.
- **Images** from callers and from tool results are saved locally for the
  responder.

The identifiers in the code (`openhuman-*`, model `openhuman-human`, `OH_SEAT_*`)
come from [OpenHuman](https://github.com/Juwan-Hwang/openhuman), the runtime this
project shares its server with, and are kept for compatibility.

---

## Repository layout

```
omni2api/
├── server/      # Rust + Axum server: OpenAI/Anthropic ingress, seat API, storage
├── cli/         # oh-seat, the responder CLI (single static Rust binary)
├── protocol/    # Wire protocol and internal model
├── sdks/        # SDK placeholders (python, rust, typescript)
├── tools/seat/  # Seat helpers and examples
├── deploy/      # Docker Compose, K8s, nginx, Anthropic bridge
└── docs/        # Architecture, quickstart, seat CLI, protocol
```

---

## Quick start

### Run the server

The default in-memory backend needs no external services:

```bash
cd server
OPENHUMAN_BACKEND=memory OPENHUMAN_PORT=8080 cargo run
```

For PostgreSQL set `OPENHUMAN_BACKEND=postgres` and `DATABASE_URL`; see
`server/.env.example` and `deploy/`.

### Sit in the responder seat

```bash
cd cli && cargo build --release
export OH_SEAT_URL=http://localhost:8080
export OH_SEAT_KEY_FILE=~/.openhuman/seat_key   # a responder-scope key
oh-seat guide                                   # the full manual, written for agents
oh-seat next                                    # wait for a caller
```

### Point a caller at it

| Setting | Value |
|---|---|
| Base URL | `http://localhost:8080/v1` |
| API Key | a caller-scope key |
| Model | `openhuman-human` |

```bash
curl -N http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer YOUR_KEY" -H "Content-Type: application/json" \
  -d '{"model":"openhuman-human","messages":[{"role":"user","content":"hello"}],"stream":true}'
```

The request holds open until the responder replies.

---

## Caller compatibility

Tested end to end with Claude Code, Codex, OpenCode, Pi, Reasonix, Cline,
GitHub Copilot CLI (BYOK), Grok CLI, ZCode, WorkBuddy, Trae, Qoder and
DeepSeek Harness. Known caller-side limits: OpenCode shell results carry no
exit code; Copilot BYOK does not forward images; Cursor CLI has no custom
endpoint.

Give callers the largest context window they offer and turn off automatic
compaction: the responder keeps the whole conversation.

---

## Docs

- `docs/seat-cli.md` — oh-seat reference
- `docs/DEVELOPMENT.md` — building and testing
- `docs/protocol.md`, `protocol/` — wire specification

## License

MIT — see [LICENSE](LICENSE).
