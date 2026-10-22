<div align="center">

# Omni2API

### Anything to API — 万物皆可 2API

</div>

---

Omni2API turns anything that can sit at a terminal into an OpenAI-compatible
API that any other AI tool can call: an AI agent, a script, or a whole
toolchain.

The caller — Claude Code, Codex, OpenCode, Pi, Cline, Copilot, Trae, Qoder,
WorkBuddy, DeepSeek Harness, or anything that speaks OpenAI Chat Completions /
Responses or Anthropic Messages — points its `Base URL` at the server and picks
the model name the seat answers to. That name belongs to the seat: the user
can pin one (`omni2api up --model …`), otherwise the seat chooses its own
(`oh-seat name …`); until someone does, it is the placeholder `omni2api`. On
the other side, whatever sits in the seat drives the `oh-seat` CLI: it reads
each message, runs tools on the caller's machine through the caller's own
tool calls, and streams its answer back.

```
Caller agent (any AI CLI / IDE / desktop app)
      │  OpenAI Chat Completions · Responses · Anthropic Messages (HTTP/SSE)
      ▼
Omni2API server (Rust + Axum)
      │  /v1/seat/*  (long-poll event stream)
      ▼
Anything in the seat running `oh-seat` (an agent or a script)
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

---

## Repository layout

```
omni2api/
├── server/      # Rust + Axum server (`omni2api` binary): OpenAI/Anthropic ingress, seat API
├── cli/         # oh-seat, the responder CLI (single static Rust binary)
├── protocol/    # Wire protocol and internal model
├── sdks/        # SDK placeholders (python, rust, typescript)
├── tools/seat/  # Seat helpers and examples
├── deploy/      # Docker Compose, K8s, nginx, Anthropic bridge
└── docs/        # Architecture, quickstart, seat CLI, protocol
```

---

## Quick start

Tell your AI agent:

> Set up omni2api (https://github.com/Juwan-Hwang/omni2api) and make yourself the API.

It follows [SETUP.md](SETUP.md) and hands you a base URL (local and a public
`https://…` one), an API key and the model name. No server, database or
account needed.

By hand it is the same two commands:

```bash
curl -fsSL https://raw.githubusercontent.com/Juwan-Hwang/omni2api/main/install.sh | sh   # Windows: irm …/install.ps1 | iex
omni2api up -d        # prints base_url (local + public), api_key, model
oh-seat next          # sit in the seat: wait for a request (oh-seat guide for the manual)
```

Point any OpenAI-compatible tool at it:

| Setting | Value |
|---|---|
| Base URL | the `base_url` printed by `omni2api up` |
| API Key | the `api_key` printed by `omni2api up` |
| Model | the name printed by `omni2api up` (the seat's own; `omni2api` until it chooses one) |

```bash
curl -N "$BASE_URL/chat/completions" \
  -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
  -d '{"model":"omni2api","messages":[{"role":"user","content":"hello"}],"stream":true}'
```

The request holds open until the seat replies.

Optional: a fixed domain through a named Cloudflare tunnel, or a long-lived
server with PostgreSQL (`omni2api serve`, see `deploy/`). Details in
[SETUP.md](SETUP.md).

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
