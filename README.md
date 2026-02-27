# minusAgent

A minimal LLM agent framework in Rust

## Architecture

```
src/
├── core/           # Node trait, Context, Config, PromptEngine, Skill
├── feature/        # Agent, LLM client, Command harness
├── interface/      # CLI, Session
└── instructions/   # System prompt
```

**Core abstraction**: The `Node` trait defines an async pipeline — `prep()`, `exec()`, `post()` — that all processing units implement.

**Agent loop**: The `Agent` orchestrates LLM reasoning and command execution in a loop (max 10 iterations), stopping when the LLM returns a completed answer.

## Quick Start

```bash
# Initialize config
minusagent init

# Edit your config
vim ~/.minusagent/config.toml

# Start a new session
minusagent new

# Start with a different LLM
minusagent new --llm openai
```

## Configuration

Config file: `~/.minusagent/config.toml`

```toml
[default]
name = "codestral"
model = "codestral-latest"
base_url = "https://codestral.mistral.ai/v1/chat/completions"
api_key = "your-api-key"
max_tokens = 4096

# Optional: additional LLMs for --llm switching
[[llm]]
name = "openai"
model = "gpt-4"
base_url = "https://api.openai.com/v1/chat/completions"
api_key = "sk-xxx"
max_tokens = 4096
```

## Testing

```bash
cargo test
```
