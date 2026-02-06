# cot

Chain-of-Thought reasoning. Splits problem-solving into planning and iterative execution.

## Thought (struct)

Single thinking step. Wraps an LLM call with action parsing.

| Field | Type | Description |
|-------|------|-------------|
| `llm` | `Llm` | LLM instance for API calls |

### Node implementation

- **prep** — Calls `ctx.to_prompt()`.
- **exec** — Delegates to `llm.exec()`.
- **post** — Parses JSON response, extracts `action` field, updates context.

## ChainOfThought (struct)

Orchestrator: plan once, then loop thinking steps until `Action::Stop` or max turns.

| Field | Type | Description |
|-------|------|-------------|
| `thought` | `Thought` | Thinking step executor |
| `plan_prompt` | `String` | Template with `{question}`, `{max_turns}` |
| `thinking_prompt` | `String` | Template with `{question}`, `{task}`, `{todos}`, `{thinking}` |
| `max_turns` | `usize` | Maximum iterations (default: 10) |

### `new(llm: Llm) -> Self`

Creates with default prompts.

### `with_max_turns(self, n: usize) -> Self`

Sets maximum iterations.

### `run(ctx: &mut Context) -> Result<Action>`

1. Injects plan prompt and runs initial planning step
2. Loops: extracts next task from `todos[0]`, runs thinking step
3. Stops when `action` is `stop` or max turns reached

### Expected LLM output (planning)

```json
{ "task": "first task", "thinking": "...", "todos": ["task 1", "task 2"], "action": "continue" }
```

### Expected LLM output (thinking)

```json
{ "thinking": "...", "todos": ["remaining"], "answer": "final (when stop)", "action": "continue/stop" }
```
