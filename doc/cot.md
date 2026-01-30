# cot

Chain-of-Thought reasoning. Splits problem-solving into planning and execution.

## Plan (struct)

First stage — breaks a question into a todo list of atomic tasks.

| Field | Type | Description |
|-------|------|-------------|
| `llm` | `Llm` | LLM instance for API calls |
| `system_prompt` | `String` | Prompt template with `{question}` placeholder |

### `new(llm: Llm) -> Self`

Creates a Plan with the default planning prompt.

### `with_prompt(self, prompt: &str) -> Self`

Overrides the default system prompt.

### Node implementation

- **prep** — Injects the user's question into the system prompt. Builds the message array.
- **exec** — Delegates to `llm.exec()`.
- **post** — Parses the JSON response, extracts `action` field (`continue` / `stop` / tool name), updates context.

### Expected LLM output format

```json
{ "todos": "- [ ] task 1\n- [ ] task 2", "action": "continue" }
```

## Execute (struct)

Second stage — works through tasks one by one.

| Field | Type | Description |
|-------|------|-------------|
| `llm` | `Llm` | LLM instance for API calls |
| `system_prompt` | `String` | Prompt template with `{question}` and `{context}` placeholders |

### `new(llm: Llm) -> Self`

Creates an Execute with the default execution prompt.

### `with_prompt(self, prompt: &str) -> Self`

Overrides the default system prompt.

### Node implementation

- **prep** — Injects question and previous progress into the system prompt. Appends a `"Continue"` user message to satisfy API requirements.
- **exec** — Delegates to `llm.exec()`.
- **post** — Parses the JSON response, updates action. Defaults to `Stop` if parsing fails.

### Expected LLM output format

```json
{
  "current": "task being worked on",
  "result": "result for this task",
  "todos": "- [x] done\n- [ ] remaining",
  "action": "continue",
  "final": "final answer (required when action is stop)"
}
```

## ChainOfThought (struct)

Orchestrator that runs Plan once, then loops Execute until `Action::Stop`.

| Field | Type | Description |
|-------|------|-------------|
| `plan` | `Plan` | Planning stage |
| `execute` | `Execute` | Execution stage |

### `new(plan: Plan, execute: Execute) -> Self`

### `run(ctx: &mut dyn Context) -> Result<()>`

Runs the full chain: plan → execute loop → stop.
