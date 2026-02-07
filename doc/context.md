# context

Conversation state: `{system_prompt, user_message, history, skills, action}`.

## Context (struct)

| Field | Type | Description |
|-------|------|-------------|
| `system_prompt` | `Option<String>` | System-level instruction |
| `user_message` | `Option<String>` | Current user input |
| `history` | `Vec<Message>` | Conversation history |
| `skills` | `Vec<Skill>` | Available skills |
| `action` | `Action` | Current execution state |

### Methods

| Method | Description |
|--------|-------------|
| `new()` | Creates empty context |
| `set_system_prompt(prompt)` | Sets system prompt |
| `set_user_message(msg)` | Sets user message |
| `push_history(msg)` | Appends to history |
| `last_content()` | Gets last message content |

## Message (struct)

| Field | Type | Description |
|-------|------|-------------|
| `role` | `Role` | Message sender |
| `content` | `Value` | Message body (JSON) |

### Factory methods

| Method | Description |
|--------|-------------|
| `system(content)` | Create system message |
| `user(content)` | Create user message |
| `assistant(content)` | Create assistant message (takes `Value`) |

## Role (enum)

`System` | `User` | `Assistant`

## Action (enum)

| Variant | Description |
|---------|-------------|
| `Continue` | Keep processing (default) |
| `Stop` | Halt execution |
| `CallTool(String)` | Invoke external tool |
