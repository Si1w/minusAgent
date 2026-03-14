# minusAgent

A general-purpose ReAct agent framework in Rust. All capabilities (tool use, MCP, custom instructions) are packaged as skills following the [Agent Skills Specification](https://agentskills.io/specification).

## Quick Start

### 1. Configure

On first run, a default config is created at `~/.minusagent/config.json`. You can also configure interactively inside the REPL.

Set your API key as an environment variable. Either export it directly:

```bash
export LLM_API_KEY="your-api-key"
```

Or create a `.env` file in the project directory (loaded automatically via `dotenvy`):

```
LLM_API_KEY=your-api-key
```

### 2. Run

```bash
cargo run
```

If the API key is missing or config is incomplete, the REPL still starts — use `/config` commands to fix it, then `/new` to create a session.

### 3. Supported Models

The agent uses `response_format: json_schema` for structured output. Only models that support this format are compatible:

- **Mistral** (codestral, mistral-large, etc.)
- **OpenAI** (gpt-4o, gpt-4o-mini, etc.)

Models that only support `json_object` or plain text (e.g. doubao/volces) are **not yet supported**.

### 4. Usage

Type a message to chat with the agent. The agent thinks step-by-step (ReAct) and can execute shell commands when needed.

#### REPL Commands

| Command | Description |
|---|---|
| `/help` | Show available commands |
| `/exit` | Exit the REPL |
| `/new` | Start a new session (fresh context) |
| `/skills` | List available skills |
| `/models` | List configured LLMs |
| `/switch <name>` | Switch to a different LLM (preserves context) |
| `/config` | View current configuration |
| `/config set <key> <value>` | Set a config field (dotted path, e.g. `agent.max_steps 30`) |
| `/config add llm` | Add a new LLM provider (interactive) |
| `/config remove llm <name>` | Remove an LLM provider by name |

#### Example Session

```
> What is 15 + 27?
[thinking] I need to calculate the sum of 15 and 27...

42

> /switch mistral-large
Switched to 'mistral-large'. Session rebuilt.

> /config set agent.max_steps 30
Set agent.max_steps = 30
```

## Architecture

Three nested loops drive the agent:

```
REPL loop (CLI transport)          ← read input → display answer → next turn
└── Orchestrator loop (Session)    ← Agent ↔ Harness dispatch until Completed
    └── CoT loop (Agent)           ← LLM calls, handles UseSkill/Continue internally,
                                     returns Execute or Completed to Session
```

## Module Plan

```
src/
├── main.rs              # CLI entry point (dotenvy, config load, REPL)
├── lib.rs               # Public API
├── core/
│   ├── mod.rs           # Node trait, Action enum
│   ├── context.rs       # Context: conversation history, skill catalog, Outcome
│   ├── agent.rs         # Agent: ReAct loop (UseSkill/Continue internal, Execute/Completed to Session)
│   ├── prompt.rs        # PromptEngine: system prompt builder, skill instruction loader
│   ├── harness.rs       # Harness: command execution via Node pipeline
│   └── llm.rs           # LLM client (structured output via Node pipeline)
├── session/
│   └── mod.rs           # Session: orchestrator with Event callbacks for transports
├── skill/
│   └── mod.rs           # SkillRegistry, SkillMeta, SKILL.md parser
├── config/
│   └── mod.rs           # Config: load, save, set, add/remove/promote LLM
└── transport/
    └── cli.rs           # CLI transport: REPL, slash commands, config management
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
- [x] Agent ReAct loop: LLM call → parse action → return to Session for dispatch
- [x] Action enum: `UseSkill` (load instructions), `Execute` (shell command), `Continue` (think), `Completed` (answer)
- [x] Session: orchestrator for agent, context, harness

### Phase 3: Context Guard
- [ ] Context guard: overflow protection wrapping LLMClient, token tracking via API usage
- [ ] Three-stage recovery: truncate observations → LLM-powered compact → fail
- [ ] Proactive compact: triggers when token usage exceeds threshold
- [ ] Session persistence: JSONL event log, create/switch/list sessions

### Phase 4: CLI Transport
- [x] CLI transport: interactive REPL with `/help`, `/exit`, `/new`, `/skills`
- [x] LLM switching: `/switch <name>` with context preservation
- [x] Config management: `/config`, `/config set`, `/config add llm`, `/config remove llm`
- [x] Graceful degradation: REPL starts even if session init fails (missing API key, etc.)
- [x] `.env` auto-loading via `dotenvy`
- [x] Event callbacks: `Thinking`, `Executing`, `Output` for transport display
- [ ] Context inspection command (`/context` with usage bar, `/compact` manual compression)
- [ ] Error handling: user interrupt (Ctrl+C) vs environment failure

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

- [Agent Loop](docs/agent-loop.md) — ReAct loop, Action enum, agent ↔ session boundary
- [Skill](docs/skill.md) — Skill system, registration, progressive disclosure, MCP
- [Harness](docs/harness.md) — Execution environment
- [Session](docs/session.md) — Session management, context guard, persistence
- [Config](docs/config.md) — Configuration schema
- [Transport](docs/transport.md) — Transport layer (CLI, Discord, HTTP)