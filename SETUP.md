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

First decide whether a tunnel is needed. Two things can be remote: the seat
(you, answering) and the callers (the tools sending requests).

| Seat | Callers | Run |
|---|---|---|
| this machine | this machine | `omni2api up -d --no-tunnel` (nothing leaves the machine) |
| this machine | anywhere | `omni2api up -d` |
| a cloud agent | anywhere | `omni2api up -d` on a machine with open internet; the cloud agent sets `OH_SEAT_URL=<public URL without /v1>` and `OH_SEAT_KEY=<responder key>` (check first: `curl <public base_url>/models`) |

If unsure, use the tunnel: it costs nothing and the local URL keeps working.

```sh
omni2api up -d
```

It runs in the background, survives your shell, and prints (plain `omni2api up`
detaches the same way when there is no terminal, so it never hangs a shell tool):

- `base_url (local)` — `http://127.0.0.1:8080/v1`
- `base_url (public)` — an `https://…/v1` address reachable from anywhere. It
  tries a Cloudflare quick tunnel first (cloudflared is downloaded on first
  run), then pinggy through the system `ssh` if Cloudflare is unreachable.
- `api_key` — the caller key
- `model` — the name callers use. Until someone chooses one it is the
  placeholder `omni2api`: the seat names itself in step 5, or the user pins
  a name with `omni2api up -d --model <name>` (a pinned name wins and the
  seat cannot change it).

The same values are in `~/.omni2api/endpoint.json`. `omni2api status` prints
them again, `omni2api down` stops everything. Keys persist across restarts;
the quick-tunnel address changes on each `up`.

## 3. Check it

```sh
curl -s "$PUBLIC_BASE_URL/models" -H "Authorization: Bearer $CALLER_KEY"
```

should list the current model name (`omni2api` until the seat names itself).

## 4. Tell the user

Give them exactly this, filled in:

```
Base URL:  <public base_url>      (or <local base_url> on this machine)
API key:   <caller key>
Model:     <the current model name — `oh-seat name` shows it; yours, once you have named yourself in step 5>
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

First, name yourself: callers address you by a model name, and it is yours
to choose. Run `oh-seat name` to see the current one. If the user asked for
a specific name, set exactly that (`oh-seat name <name>`); otherwise pick
one you like. Do this before step 4, so the user receives the real name.
If the name is reported as pinned, the user chose it at `up` time — use it
as it is.

## Options

- Fixed domain: create a named tunnel in Cloudflare Zero Trust, then
  `OMNI2API_TUNNEL_TOKEN=… OMNI2API_PUBLIC_URL=https://api.example.com omni2api up -d`.
- Local only: `omni2api up -d --no-tunnel`. Another port: `--port 9000`.
- Fixed keys: `OMNI2API_CALLER_KEY` / `OMNI2API_RESPONDER_KEY`.
- A model name of the user's choosing: `omni2api up -d --model <name>`
  (or `OMNI2API_MODEL`). Without it, the seat names itself.
- A long-lived multi-user server with PostgreSQL: `omni2api serve` with
  `OMNI2API_BACKEND=postgres` and `DATABASE_URL` (see `deploy/`).

## Troubleshooting

- `omni2api up` (foreground) shows tunnel errors directly; the background
  log is `~/.omni2api/up.log`.
- `note:` under the public URL, or no public URL at all: the network this
  machine is on filters outbound traffic (common in agent sandboxes and
  company networks: an egress proxy answering 403 for trycloudflare.com).
  That is the machine, not omni2api. Options, best first:
  1. Run `omni2api up` on a machine with normal internet access (the user's
     own computer). The seat can stay where you are only if your network lets
     you reach that machine's URL; otherwise sit in the seat on that machine too.
  2. `--tunnel pinggy` (ssh on port 443), if your filter allows it.
  3. A fixed domain the filter allows: named tunnel (`OMNI2API_TUNNEL_TOKEN`)
     or `omni2api serve` on a server.
  A URL marked unverified may still work from outside: the check runs from
  this same filtered machine. Ask the user to open `<public base_url>/models` in a
  browser: any JSON answer, even 401, means it is reachable.
- The user's tool says "could not reach" the public URL while it works from
  elsewhere: the user's own network cannot reach that tunnel domain (seen in
  mainland China with trycloudflare.com and pinggy). Tell the user to turn on
  their proxy, in TUN / system-wide mode so the AI tool's traffic goes
  through it too (a browser-only proxy is not enough), then retry. Long
  term: a fixed domain via a named tunnel.
- Tunnel logs: `~/.omni2api/tunnel-<provider>.log`. Free tunnels can die;
  `up` watches the one it handed out (by probing it, or — when this
  machine cannot reach its own URL — by watching the tunnel process) and
  opens a new one, rewriting endpoint.json. A reopened anonymous tunnel
  gets a NEW address, so re-read endpoint.json and give the user the new
  URL; callers already configured with the old one cannot be rescued
  remotely. For an address that must not change, use a named tunnel.
- cloudflared download fails (TLS-intercepting proxy, blocked GitHub): fetch
  the file named in the error another way and save it into `~/.omni2api/bin/`.
- 401: wrong key. Callers use `caller_key`, the seat uses `responder_key`.
