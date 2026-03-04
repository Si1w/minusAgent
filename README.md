# minusAgent

A minimal LLM agent framework in Rust

## Feature Track

- [x] Agent Loop
- [x] Harness
- [~] Session Management
- [ ] Skills
- [ ] Channel
- [ ] Gateway
- [x] Memory
- [ ] Heartbeat
- [ ] Delivery
- [ ] Resilience
- [ ] Concurrency
- [ ] Safety Guards

## Architecture

```
src/
├── core/           # Action, Context, Node trait (all core types in one place)
├── config/         # Config, path management (base_dir, config_path, sessions_dir)
├── agent/          # Agent loop, LLM client
│   └── llm.rs      # LLM HTTP client (implements Node)
├── memory/         # Session persistence (save/load/list)
├── prompt/         # PromptEngine, system prompt
├── session/        # Session, Harness
│   └── harness.rs  # Command executor (implements Node)
└── cli/            # CLI entry point, input loop, command dispatch
```

**Core abstraction**: The `Node` trait defines an async pipeline — `prep()`, `exec()`, `post()` — implemented by `LLM` and `Harness`.

**Agent loop**: `Agent` calls `LLM` in a loop until the action is not `Running` (i.e., `Completed` or `Execute`), bounded by `max_iterations`.

**Session**: `Session` handles a single query by alternating between `Agent` (LLM reasoning) and `Harness` (command execution) until `Completed`. The input loop lives in the CLI layer.

```
CLI (user input loop, command dispatch)
  ├── Session.query() (agent/harness loop per query)
  │     └── Agent (LLM loop, bounded by max_iterations)
  │           └── Harness (command execution, triggered by Execute action)
  └── Memory (save/load/list via /save, /list, resume commands)
```

**Safety**: `Harness` blocks a configurable blacklist of destructive commands (e.g. `rm -rf /`, `mkfs`) before prompting the user for approval.

## Quick Start

```bash
# Initialize config
minusagent init

# Edit your config
vim ~/.minusagent/config.json

# Start a session (default)
minusagent

# Or explicitly
minusagent new

# Start with a specific LLM
minusagent --llm codestral-latest

# Save session (inside a running session)
> /save

# List saved sessions
minusagent list

# Resume a saved session
minusagent resume 20260304_153021
```

## Configuration

Config file: `~/.minusagent/config.json`

```json
{
  "agent": {
    "max_iterations": 10,
    "default_llm": "codestral-latest"
  },
  "llm": [
    {
      "model": "codestral-latest",
      "base_url": "https://codestral.mistral.ai/v1/chat/completions",
      "api_key": "your-api-key",
      "max_tokens": 4096
    }
  ]
}
```

## LLM Response Format

The LLM must respond with JSON in the following format:

```json
{
  "thought": {
    "thought_type": "Planning | Solving | GoalSetting",
    "content": "..."
  },
  "action": "Running | Completed | Execute",
  "command": "shell command here",
  "answer": "final answer here"
}
```

- `command` is required when `action` is `Execute`
- `answer` is required when `action` is `Completed`

## Testing

```bash
cargo test
```
