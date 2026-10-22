# Setting up Omni2API (for the AI agent doing it)

Your user said something like "set up omni2api" or "turn yourself into an API".
Do every step yourself; the user should only receive the result in step 4.
No server, database, domain or account is needed.

## 1. Install

macOS / Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/Juwan-Hwang/omni2api/main/install.sh | sh
export PATH="$HOME/.omni2api/bin:$PATH"
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Juwan-Hwang/omni2api/main/install.ps1 | iex
```

Both put `omni2api` and `oh-seat` in `~/.omni2api/bin`. If downloading a
release fails, build from source: `cargo build --release --no-default-features --bin omni2api`
in `server/` and `cargo build --release` in `cli/`.

## 2. Start it

```sh
omni2api up -d
```

It runs in the background, survives your shell, and prints:

- `base_url (local)` — `http://127.0.0.1:8080/v1`
- `base_url (public)` — `https://<random>.trycloudflare.com/v1`, reachable from anywhere
  (cloudflared is downloaded automatically on first run)
- `api_key` — the caller key
- `model` — `openhuman-human`

The same values are in `~/.omni2api/endpoint.json`. `omni2api status` prints
them again, `omni2api down` stops everything. Keys persist across restarts;
the quick-tunnel address changes on each `up`.

## 3. Check it

```sh
curl -s "$PUBLIC_BASE_URL/models" -H "Authorization: Bearer $CALLER_KEY"
```

should list `openhuman-human`.

## 4. Tell the user

Give them exactly this, filled in:

```
Base URL:  <public base_url>      (or <local base_url> on this machine)
API key:   <caller key>
Model:     openhuman-human
Works in any OpenAI-compatible tool (Chat Completions, Responses) and as an Anthropic base URL.
```

Never hand out the responder key: it is what answers.

## 5. Be the API

Requests now wait for whoever sits in the seat. If that is you:

```sh
oh-seat guide     # the full manual, read once
oh-seat next      # wait for the next request
oh-seat reply "…" # answer it
```

`oh-seat` finds the local endpoint and responder key by itself. The caller's
tools (shell, file edits…) are yours through `oh-seat tool`; see the guide.
The API answers only while something keeps running `oh-seat next`.

## Options

- Fixed domain: create a named tunnel in Cloudflare Zero Trust, then
  `OMNI2API_TUNNEL_TOKEN=… OMNI2API_PUBLIC_URL=https://api.example.com omni2api up -d`.
- Local only: `omni2api up -d --no-tunnel`. Another port: `--port 9000`.
- Fixed keys: `OMNI2API_CALLER_KEY` / `OMNI2API_RESPONDER_KEY`.
- A long-lived multi-user server with PostgreSQL: `omni2api serve` with
  `OPENHUMAN_BACKEND=postgres` and `DATABASE_URL` (see `deploy/`).

## Troubleshooting

- `omni2api up` (foreground) shows tunnel errors directly; the background
  log is `~/.omni2api/up.log`.
- No public URL: outbound access to Cloudflare is blocked; the local URL still works.
- 401: wrong key. Callers use `caller_key`, the seat uses `responder_key`.
