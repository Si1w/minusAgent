# Configuration

## Location

Stored at `~/.minusagent/config.json`, editable by users through CLI commands or direct file editing.

## Schema

```json
{
  "agent": {
    "max_steps": 20
  },
  "llm": [
    {
      "name": "default",
      "model": "codestral-latest",
      "base_url": "https://codestral.mistral.ai/v1/chat/completions",
      "api_key_env": "LLM_API_KEY",
      "max_tokens": 4096
    }
  ],
  "skills": {
    "paths": [],
    "disabled": []
  }
}
```

## Notes

- `api_key_env` stores the environment variable **name** (e.g. `"LLM_API_KEY"`), not the key itself. The actual key is read from the environment at runtime (e.g. exported in `~/.zshrc` or `~/.bashrc`).
- `llm` is an array to support multiple LLM providers, each with its own `api_key_env`
- `skills.paths` allows adding custom skill directories beyond the default locations
- `skills.disabled` allows disabling specific skills by name
