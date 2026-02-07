# cot

Chain-of-Thought reasoning. Splits problem-solving into planning and iterative execution.

## Thought (struct)

Single thinking step. Wraps an LLM call with action parsing.

| Field | Type | Description |
|-------|------|-------------|
| `llm` | `Llm` | LLM instance for API calls |

### Node implementation

- **prep** — Calls `prompt::render(ctx)`.
- **exec** — Delegates to `llm.exec()`.
- **post** — Parses `<action>` tags from response, extracts body, updates context.

## ChainOfThought (struct)

Orchestrator: plan once, then loop thinking steps until `Action::Stop` or max turns.

| Field | Type | Description |
|-------|------|-------------|
| `thought` | `Thought` | Thinking step executor |
| `plan_prompt` | `String` | Loaded from `skills/plan/SKILL.md` |
| `thinking_prompt` | `String` | Loaded from `skills/thinking/SKILL.md` |
| `max_turns` | `usize` | Maximum iterations (default: 10) |

### `new(llm: Llm) -> Self`

Creates with prompts loaded from skill files.

### `with_max_turns(self, n: usize) -> Self`

Sets maximum iterations.

### `run(ctx: &mut Context) -> Result<Action>`

1. Saves original question to history (for Question section in prompt)
2. Sets `plan_prompt` as system prompt, runs initial planning step
3. Loops:
   - Extracts body from last response
   - Pops last response from history
   - Sets `thinking_prompt` as system prompt
   - Sets previous body as new user message
   - Runs thinking step
4. Stops when action is `stop` or max turns reached

### Expected LLM output

```
<continue>reasoning and remaining tasks</continue>
<stop>final answer</stop>
```
