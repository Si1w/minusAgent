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
- [ ] Error handling: user interrupt vs environment failure

### Phase 3: Persistence & CLI
- [ ] Session persistence (optional): save/load JSON files
- [ ] CLI transport: interactive REPL
- [ ] Config CLI commands: view/edit config

### Phase 4: Advanced
- [ ] MCP skill wrapper
- [ ] Multi-agent skill execution
- [ ] Additional transports (Discord, HTTP)

## Docs

- [Agent Loop](docs/agent-loop.md) — ReAct loop, structured output, outcome, observation, error handling
- [Skill](docs/skill.md) — Skill system, registration, progressive disclosure, MCP
- [Harness](docs/harness.md) — Execution environment
- [Session](docs/session.md) — Session management, persistence
- [Config](docs/config.md) — Configuration schema
- [Transport](docs/transport.md) — Transport layer (CLI, Discord, HTTP)