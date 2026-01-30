# utils

Helper functions for common tasks.

## `parse_json(content: &str) -> Result<Value>`

Extracts and parses JSON from LLM responses.

| Parameter | Type | Description |
|-----------|------|-------------|
| `content` | `&str` | Raw string, possibly wrapped in markdown `` ```json `` fences |

**Returns:** `serde_json::Value` on success.

**Behavior:**
1. If `` ```json `` fence is found, extracts content between the fences.
2. Otherwise, parses the raw string directly.
3. Returns an error if no valid JSON is found.
