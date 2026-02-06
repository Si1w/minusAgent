# cli

Entry point and command dispatch. Uses `clap` for argument parsing.

## Commands (enum)

| Subcommand | Parameters | Description |
|------------|------------|-------------|
| `prompt` | `text: String` | Single streaming LLM call |
| `cot` | `text: String`, `--max-turns: Option<usize>` | Chain-of-Thought reasoning |
| `interactive` | `--cot: bool` | Multi-turn conversation. Use `--cot` for CoT mode. Type `exit` to quit |

## Environment Variables

| Variable | Required | Default |
|----------|----------|---------|
| `LLM_API_KEY` | Yes | — |
| `LLM_BASE_URL` | No | `https://codestral.mistral.ai/v1/chat/completions` |
| `LLM_MODEL` | No | `codestral-latest` |

## Interactive (struct)

Multi-turn conversation handler.

| Field | Type | Description |
|-------|------|-------------|
| `llm` | `Llm` | LLM instance |
| `cot` | `Option<ChainOfThought>` | CoT executor (if `--cot` enabled) |

### `new(llm: Llm, cot: bool) -> Self`

### `run(ctx: &mut Context) -> Result<()>`

Loops: read input → call LLM (streaming or CoT) → print response → update history.
