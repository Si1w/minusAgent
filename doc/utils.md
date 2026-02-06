# utils

## feature/utils

### `process_sse_stream(resp, on_chunk) -> Result<(String, bool)>`

Processes Server-Sent Events stream from LLM API.

| Parameter | Type | Description |
|-----------|------|-------------|
| `resp` | `Response` | HTTP response with SSE stream |
| `on_chunk` | `impl Fn(&str)` | Callback for each content chunk |

**Returns:** `(full_content, interrupted)` — interrupted is `true` if user pressed a key.

## interface/utils

### `start_thinking() -> (Arc<AtomicBool>, JoinHandle<()>)`

Spawns animated spinner (`⠋⠙⠹...`) on stdout.

### `stop_thinking(running, handle)`

Stops spinner and clears the line.
