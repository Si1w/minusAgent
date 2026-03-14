# Transport Layer

## Definition

Thin adapter between external platforms and Session. Each transport only needs to:

- Receive user input → call `session.turn(input, on_event)`
- Handle `Event` callbacks for intermediate display (thinking, executing, output)
- Receive final answer → deliver to platform

No heavy abstraction needed — a transport is just a loop that bridges I/O.

## CLI Transport

The MVP transport. Interactive REPL with slash commands.

### Startup Flow

1. Load config from `~/.minusagent/config.json` (or create default on first run)
2. Load `.env` via `dotenvy` for API keys
3. Try to create a `Session` — if it fails (e.g. missing env var), REPL still starts
4. User can fix config via `/config` commands, then `/new` to create a session

### Event Display

Session emits `Event` callbacks during `turn()`:

| Event | Display |
|---|---|
| `Thinking(content)` | `[thinking] content` (dimmed) |
| `Executing(command)` | `[executing] command` (yellow) |
| `Output(content, true)` | Command output (dimmed) |
| `Output(content, false)` | `[error] content` (red) |

### Slash Commands

| Command | Description |
|---|---|
| `/help` | Show available commands |
| `/exit` | Exit the REPL |
| `/new` | Start a new session (fresh context) |
| `/skills` | List available skills |
| `/switch <name>` | Switch LLM, rebuild session with context preserved |
| `/config` | View current configuration |
| `/config set <key> <value>` | Set a config field (dotted path) |
| `/config add llm` | Add an LLM (interactive) |
| `/config remove llm <name>` | Remove an LLM by name |

## Future Transports

- **Discord / HTTP**: Same session interface, different I/O loop
