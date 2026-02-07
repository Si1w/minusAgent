# utils

## feature/utils

### `process_sse_stream(resp, on_chunk) -> Result<(String, bool)>`

Processes Server-Sent Events stream from LLM API.

| Parameter | Type | Description |
|-----------|------|-------------|
| `resp` | `Response` | HTTP response with SSE stream |
| `on_chunk` | `impl Fn(&str)` | Callback for each content chunk |

**Returns:** `(full_content, interrupted)` — interrupted is `true` if user pressed a key.

### `parse_action(content: &str) -> (Action, &str)`

Parses `<action>body</action>` tags from LLM response.

| Parameter | Type | Description |
|-----------|------|-------------|
| `content` | `&str` | Raw LLM response text |

**Returns:** `(action, body)` — action is `Continue` or `Stop`, body is the content inside the tags. Falls back to `Stop` with full content if no tag is found.

## interface/utils

### `start_thinking() -> (Arc<AtomicBool>, JoinHandle<()>)`

Spawns animated spinner (`⠋⠙⠹...`) on stdout.

### `stop_thinking(running, handle)`

Stops spinner and clears the line.
