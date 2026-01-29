# minusAgent

A minimal LLM agent framework in Rust, built with YAGNI philosophy.

## Overview

minusAgent provides autonomous LLM-powered workflows through a composable node-based pipeline. It supports single prompts, chain-of-thought reasoning, and interactive conversations.

## Architecture

```
src/
├── cli/        # CLI commands & orchestration
├── core/       # Node trait (prep → exec → post pipeline)
├── llm/        # LLM API client
├── cot/        # Chain-of-Thought (Plan + Execute loop)
└── context/    # Message history & action state
```

**Core abstraction**: The `Node` trait defines an async pipeline — `prep()`, `exec()`, `post()` — that all processing units implement.

## Usage

```bash
# Single prompt
cargo run -- prompt "your question"

# Chain-of-Thought reasoning
cargo run -- cot "your question"

# Interactive multi-turn conversation
cargo run -- interactive
```

## Configuration

Set environment variables in `.env`:

```
LLM_API_KEY=<your-api-key>
LLM_BASE_URL=<optional, defaults to Mistral Codestral>
LLM_MODEL=<optional, defaults to codestral-2508>
```

## How CoT Works

1. **Plan** — breaks the question into a YAML task list
2. **Execute** — iterates through tasks until completion
3. **Output** — extracts the final answer from structured YAML response
