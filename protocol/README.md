# Omni2API Protocol Specification

This directory is the **single source of truth** for the Omni2API internal data model
and its compatibility mapping to the OpenAI API.

---

## Architecture Principle

Omni2API does **not** define a new agent-facing protocol.

External clients (Codex, Claude Code, Trae, any OpenAI-compatible agent) connect via:

```
Base URL  +  API Key
```

They speak **OpenAI-compatible API** — nothing new to learn.

Internally, Omni2API maintains its own data model for:

- Server storage (PostgreSQL)
- Realtime synchronization (NATS JetStream / WebSocket)
- Multi-client support
- Human client rendering (Flutter)

```
┌─────────────────────────────────┐    OpenAI API     ┌──────────────────────┐    Internal     ┌────────────────┐
│  External User's Agent Client   │ ──────────────→  │  Omni2API Server     │ ──────────────→ │  Human Client   │
│  (Codex, Trae, Cursor, etc.)    │                  │  (translation layer)  │                │  (Flutter)      │
└─────────────────────────────────┘                  └──────────────────────┘                 └────────────────┘
                                                             ↑
                                                        Internal Model
                                                        (this directory)
```

---

## Directory Structure

```
protocol/
├── README.md                    # This file
├── internal-model/
│   ├── identity.md              # HumanIdentity + AgentIdentity
│   ├── conversation.md          # Conversation model
│   ├── message.md               # Internal message format
│   ├── event.md                 # Event types for WS / NATS
│   └── capability.md            # Capability descriptors (NOT tool approval)
├── openai-compat/
│   ├── chat-completions.md      # Internal model ↔ OpenAI Chat Completions API
│   └── responses.md             # Internal model ↔ OpenAI Responses API (future)
└── examples/
    ├── 01-agent-sends-message.md   # Agent → Omni2API (OpenAI format → internal)
    ├── 02-internal-storage.md      # How Omni2API stores the message
    └── 03-human-replies.md         # Human reply → internal → OpenAI format → Agent
```

---

## Core Design Rules

1. **`assistant` ≠ AI.** In OpenAI's API, `role: assistant` means "the API responder."
   In Omni2API, the API responder is the **Human Endpoint**, and the `user` role
   maps to the **Agent Client**. This mapping is context-dependent and explicitly
   documented in `openai-compat/`.

2. **No tool approval.** Capabilities describe what a participant *can do*,
   not what an agent is *allowed to do*. Agent tools (terminal, filesystem, etc.)
   belong to the agent's own environment.

3. **Protocol-first, UI-agnostic.** The internal model does not reference
   Flutter, Material, or any specific client framework.

4. **Multi-participant.** A conversation can be `Agent ↔ Human` or `Human ↔ Human`.
   The model must not hardcode a two-party assumption.

---

## Status

**Phase 2 — Internal Data Model Design.** All files listed above exist
and are kept in sync with the reference implementation
(`server/src/model/`, `server/src/openai_compat.rs`,
`server/src/ws/protocol.rs`). When the wire format changes, the spec
and the code change in the same commit.
