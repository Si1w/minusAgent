# llm

HTTP client for LLM API calls. Implements the `Node` trait.

## Llm (struct)

| Field | Type | Description |
|-------|------|-------------|
| `client` | `reqwest::Client` | HTTP client instance |
| `base_url` | `String` | API endpoint URL |
| `model` | `String` | Model identifier |
| `api_key` | `String` | Bearer token for authentication |

### `new(base_url: &str, model: &str, api_key: &str) -> Self`

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_url` | `&str` | Chat completions API endpoint |
| `model` | `&str` | Model name (e.g. `codestral-latest`) |
| `api_key` | `&str` | API key for Bearer auth |

### `exec_stream(prep_res, on_chunk) -> Result<Option<Value>>`

Streaming API call with chunk callback.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prep_res` | `Option<Value>` | Messages JSON array |
| `on_chunk` | `StreamCallback` | Callback invoked for each content chunk |

Returns `{ choices: [{ message: { content } }], interrupted }`. Press any key to interrupt.

## Node implementation

- **prep** — Calls `ctx.to_prompt()` to build the messages JSON array.
- **exec** — POSTs `{ model, messages }` to `base_url` with Bearer auth. Returns raw API response.
- **post** — Extracts `choices[0].message.content` and pushes it as assistant message. Returns `Action::Continue`.
