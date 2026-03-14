# Configuration

## Location

Stored at `~/.minusagent/config.json`. On first run, a default config is auto-created. Editable via REPL commands or direct file editing.

## Schema

```json
{
  "agent": {
    "max_steps": 20
  },
  "llm": [
    {
      "name": "codestral",
      "model": "codestral-latest",
      "base_url": "https://codestral.mistral.ai/v1/chat/completions",
      "api_key_env": "LLM_API_KEY",
      "max_tokens": 4096,
      "context_window": 25600
    }
  ],
  "skills": {
    "paths": []
  }
}
```

## Fields

### `agent`

| Field | Type | Description |
|---|---|---|
| `max_steps` | `u32` | Maximum LLM calls per agent run (default: 20) |

### `llm[]`

The first entry in the array is the active LLM. Use `/switch <name>` to promote a different one.

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `string` | required | Unique identifier for this LLM |
| `model` | `string` | required | Model name sent to the API |
| `base_url` | `string` | required | OpenAI-compatible chat completions endpoint |
| `api_key_env` | `string` | required | Environment variable name holding the API key |
| `max_tokens` | `u32` | `4096` | Max tokens per LLM response |
| `context_window` | `usize` | `128000` | Context window size in tokens |

### `skills`

| Field | Type | Description |
|---|---|---|
| `paths` | `string[]` | Additional directories to scan for skills |

## API Key

`api_key_env` stores the environment variable **name** (e.g. `"LLM_API_KEY"`), not the key itself. Set the actual key via:

- Export in shell: `export LLM_API_KEY="your-key"`
- Or create a `.env` file in the project directory (auto-loaded via `dotenvy`)

## REPL Commands

| Command | Description |
|---|---|
| `/config` | View current configuration as JSON |
| `/config set <key> <value>` | Set a field by dotted path (e.g. `agent.max_steps 30`, `llm.0.model gpt-4o`) |
| `/config add llm` | Add a new LLM provider (interactive prompts for name, model, URL, env var) |
| `/config remove llm <name>` | Remove an LLM provider by name |
| `/switch <name>` | Promote an LLM to the top, rebuild session (context preserved) |

All changes are auto-saved to `~/.minusagent/config.json`.
