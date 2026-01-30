# cli

Entry point and command dispatch. Uses `clap` for argument parsing.

## Commands (enum)

| Subcommand | Parameters | Description |
|------------|------------|-------------|
| `prompt` | `text: String` | Single LLM call. Sends the text and prints the response |
| `cot` | `text: String` | Chain-of-Thought reasoning. Plans then executes step by step |
| `interactive` | — | Multi-turn conversation loop. Type `exit` or `quit` to stop |

## Internal functions

### `create_llm() -> Result<Llm>`

Reads configuration from environment variables (supports `.env` file via `dotenvy`).

| Env Variable | Required | Default |
|-------------|----------|---------|
| `LLM_API_KEY` | Yes | — |
| `LLM_BASE_URL` | No | `https://codestral.mistral.ai/v1/chat/completions` |
| `LLM_MODEL` | No | `codestral-2508` |

### `start_thinking() -> (Arc<AtomicBool>, JoinHandle<()>)`

Spawns an animated spinner on stdout. Returns a flag and handle to stop it later.

### `stop_thinking(running: Arc<AtomicBool>, handle: JoinHandle<()>)`

Stops the spinner and clears the line.

### `parse_final(content: &str) -> String`

Extracts the `"final"` field from a JSON response. Falls back to returning the raw content.

| Parameter | Type | Description |
|-----------|------|-------------|
| `content` | `&str` | Raw LLM response, possibly wrapped in markdown fences |
