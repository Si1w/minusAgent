# context (Demo)

Manages conversation state throughout the pipeline.

## ChatContext (struct)

Concrete implementation of `Context`.

| Field | Type | Description |
|-------|------|-------------|
| `messages` | `Vec<Message>` | Ordered list of conversation messages |
| `action` | `Action` | Current execution flow control state |

### `new() -> Self`

Creates an empty context with `Action::Continue`.

### `with_system(content: impl Into<String>) -> Self`

Creates a context pre-loaded with a system message.

## Message (struct)

A single message in the conversation.

| Field | Type | Description |
|-------|------|-------------|
| `role` | `Role` | Who sent this message |
| `content` | `String` | Message body |
| `name` | `Option<String>` | Optional sender name |
| `tool_call_id` | `Option<String>` | Tool call identifier (for `Tool` role) |

### Factory methods

| Method | Parameters | Description |
|--------|------------|-------------|
| `system(content)` | `impl Into<String>` | Create a system message |
| `user(content)` | `impl Into<String>` | Create a user message |
| `assistant(content)` | `impl Into<String>` | Create an assistant message |
| `tool(tool_call_id, content)` | `impl Into<String>, impl Into<String>` | Create a tool response message |

## Role (enum)

| Variant | Description |
|---------|-------------|
| `System` | System-level instruction |
| `User` | User input |
| `Assistant` | LLM response |
| `Tool` | Tool execution result |

## Action (enum)

Controls execution flow. Default: `Continue`.

| Variant | Description |
|---------|-------------|
| `Continue` | Keep processing |
| `Stop` | Halt execution |
| `CallTool(String)` | Invoke an external tool by name |

## Context (trait)

Interface for conversation state management.

| Method | Signature | Description |
|--------|-----------|-------------|
| `messages()` | `&[Message]` | Get all messages |
| `push(msg)` | `Message` | Append a message |
| `extend(msgs)` | `Vec<Message>` | Append multiple messages |
| `action()` | `&Action` | Get current action |
| `set_action(action)` | `Action` | Update current action |
| `last()` | `Option<&Message>` | Get the last message |
| `last_content()` | `Option<&str>` | Get the last message's content |
| `to_vec()` | `Vec<Value>` | Serialize all messages to JSON |
