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
| `model` | `&str` | Model name (e.g. `codestral-2508`) |
| `api_key` | `&str` | API key for Bearer auth |

## Node implementation

- **prep** — Converts `ctx.to_vec()` messages into a JSON array.
- **exec** — POSTs `{ model, messages }` to `base_url` with Bearer auth. Returns the raw API response.
- **post** — Extracts `choices[0].message.content` from the response and pushes it as an assistant message to context.
