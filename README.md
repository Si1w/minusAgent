# minusAgent

A general-purpose ReAct agent framework in Rust. All capabilities (tool use, MCP, custom instructions) are packaged as skills following the [Agent Skills Specification](https://agentskills.io/specification).

## Architecture

```
┌─────────────────────────────────────────┐
│              Transport Layer            │
│         (CLI / Discord / HTTP)          │
│  Thin wrapper: input → session, output  │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│               Session                   │
│  Orchestrates agent, context, harness   │
│  ┌─────────────────────────────────┐    │
│  │  Context (message history)      │    │
│  └─────────────────────────────────┘    │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│             Agent (ReAct Loop)          │
│  LLM call → parse action → dispatch     │
└─────┬────────────────────────┬──────────┘
      │                        │
┌─────▼─────┐          ┌──────▼──────┐
│    LLM    │          │   Harness   │
│  (chat    │          │  (execute   │
│   API)    │          │   skill,    │
└───────────┘          │   observe)  │
                       └──────┬──────┘
                              │
                    ┌─────────▼─────────┐
                    │      Skills       │
                    │  (local/global/   │
                    │   built-in/MCP)   │
                    └───────────────────┘
```

## Module Plan

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Public API
├── core/
│   ├── mod.rs           # Node trait, Outcome
│   ├── context.rs       # Context: conversation message history
│   ├── agent.rs         # Agent: ReAct loop (Node)
│   ├── harness.rs       # Harness: command execution via Node pipeline
│   └── llm.rs           # LLM client (structured output, JSON Schema)
├── session/
│   └── mod.rs           # Session: orchestrates agent, context, harness
├── skill/
│   ├── mod.rs           # Skill trait, registry, discovery
│   └── loader.rs        # SKILL.md parser (frontmatter + body)
├── config/
│   └── mod.rs           # Config loading & management
└── transport/
    └── cli.rs           # CLI transport (MVP)
```

## Implementation Phases

### Phase 1: Foundation
- [x] Config module: load/validate `config.json`
- [x] Skill loader: parse SKILL.md (frontmatter + body)
- [x] Skill registry: discover and register skills from configured paths

### Phase 2: Agent Loop
- [x] LLM client: structured output with JSON Schema, ThoughtType enum
- [x] Context: conversation message history management
- [x] Harness: command execution via Node pipeline, dangerous command blocking
- [x] Agent ReAct loop: LLM call → parse response → dispatch skill → observe → loop
- [x] Session: orchestrator for agent, context, harness

### Phase 3: Session Persistence & Context Guard
- [ ] Session persistence: JSONL event log, create/switch/list sessions
- [ ] Context guard: prevent context overflow with 3-stage strategy
  - Stage 1: truncate oversized tool results (head-only, cap at 30% of budget)
  - Stage 2: LLM-powered summarization of older messages, keep recent 20%
  - Stage 3: hard error if still over budget
- [ ] Token estimation (1 token ≈ 4 chars) and context budget tracking

### Phase 4: CLI Transport
- [ ] CLI transport: interactive REPL with session commands (`/new`, `/list`, `/switch`)
- [ ] Context inspection command (`/context` with usage bar, `/compact` manual compression)
- [ ] Error handling: user interrupt (Ctrl+C) vs environment failure
- [ ] Config CLI commands: view/edit config

### Phase 5: Intelligence — Bootstrap & Memory
- [ ] Bootstrap loader: assemble system prompt from workspace `.md` files (SOUL, IDENTITY, TOOLS, USER, MEMORY, BOOTSTRAP)
- [ ] Skills injection: scan `workspace/skills/*/SKILL.md`, inject descriptions into system prompt
- [ ] Memory store: hybrid search pipeline
  - TF-IDF keyword search + hash-based vector approximation
  - Weighted fusion (vector 0.7 + keyword 0.3)
  - Time decay (exponential, based on file date)
  - MMR re-ranking (balance relevance vs diversity)
- [ ] Auto-recall: each user turn triggers memory search, inject relevant memories into system prompt

### Phase 6: Channels & Routing
- [ ] Channel abstraction: unified `InboundMessage` trait across platforms
- [ ] CLI channel implementation
- [ ] Additional channels: Telegram (long-polling, media group buffering, forum topics), Feishu (webhook), Discord
- [ ] Routing binding table: 5-tier priority routing (peer → guild → account → channel → default)
- [ ] Agent manager: multiple agent configs (id, name, personality, model, dm_scope)
- [ ] Gateway server: WebSocket JSON-RPC 2.0 protocol for external control

### Phase 7: Heartbeat & Cron
- [ ] Heartbeat runner: background thread for periodic autonomous checks
  - Preconditions: HEARTBEAT.md exists, interval elapsed, within active hours, not already running
  - Mutual exclusion with user conversation (user always takes priority)
- [ ] Cron service: scheduled tasks from `CRON.json`
  - Schedule types: `at` (one-shot), `every` (fixed interval), `cron` (cron expression)
  - Payload types: `agent_turn` (LLM call) or `system_event` (plain text)
  - Auto-disable after 5 consecutive failures

### Phase 8: Reliable Delivery
- [ ] Delivery queue: disk-persisted queue with atomic writes (tmp + rename + fsync)
- [ ] Retry with exponential backoff + 20% jitter (5s → 25s → 2min → 10min)
- [ ] Failed queue: move to `failed/` after max retries (5)
- [ ] Message chunking: split by platform limits (Telegram 4096, Discord 2000), prefer paragraph boundaries
- [ ] Recovery scan on startup

### Phase 9: Resilience
- [ ] 3-layer retry onion:
  - Layer 1 (Auth rotation): cycle API key profiles, cooldown per failure type (rate_limit 120s, auth 300s, timeout 60s)
  - Layer 2 (Overflow recovery): up to 3 context compression attempts
  - Layer 3 (Tool-use loop): standard ReAct loop with stop_reason check
- [ ] Fallback models: degrade to cheaper model (e.g. haiku) when all profiles exhausted
- [ ] Failure simulation: `/simulate-failure <reason>` for testing

### Phase 10: Concurrency
- [ ] Lane queue: named FIFO queues with configurable max concurrency
- [ ] Command queue: central dispatcher routing tasks to lanes
- [ ] Standard lanes: `main` (user), `cron` (scheduled), `heartbeat` (background), each max_concurrency=1
- [ ] Generation counter for restart recovery

### Phase 11: Advanced Extensions
- [ ] MCP skill wrapper
- [ ] Multi-agent skill execution

## Docs

- [Agent Loop](docs/agent-loop.md) — ReAct loop, structured output, outcome, observation, error handling
- [Skill](docs/skill.md) — Skill system, registration, progressive disclosure, MCP
- [Harness](docs/harness.md) — Execution environment
- [Session](docs/session.md) — Session management, persistence
- [Config](docs/config.md) — Configuration schema
- [Transport](docs/transport.md) — Transport layer (CLI, Discord, HTTP)