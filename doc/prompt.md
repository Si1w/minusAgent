# prompt

Renders Context into LLM-ready messages with structured output format.

## PromptEngine (struct)

| Field | Type | Description |
|-------|------|-------------|
| `context` | `Context` | Conversation state to render |
| `structured_output` | `bool` | Whether to append JSON output format (default: true) |

### `new(context: Context) -> Self`

Creates engine with structured output enabled.

### `without_structured_output(self) -> Self`

Disables structured output format.

### `instruction() -> String`

Returns system prompt with optional output format appended.

### `render() -> Value`

Renders context into JSON messages array with sections:

1. **System** — System prompt + output format
2. **Skills** — Available skills as `### name` + description (if any)
3. **Question** — First user message from history (original question)
4. **Chat History** — All messages in history
5. **User** — Current user message

### Output format

When `structured_output` is true, appends action tag instructions:

```
<continue>your response</continue>
<stop>your response</stop>
```

## `render(ctx: &Context) -> Value`

Convenience function. Creates engine and renders.
