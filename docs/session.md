# Session

## Definition

A session is the top-level orchestrator for a single agent interaction. It coordinates:

- **Context**: Conversation history (managed by `Context`)
- **Agent**: ReAct loop execution
- **Harness**: Skill execution environment
- **Config**: Configuration snapshot

Session receives user input from the transport layer, drives the agent loop, and returns the final answer.

## Context (Message History)

Context manages the ordered message history. Each message has one of three roles:

```json
[
  { "role": "user", "content": "..." },
  { "role": "assistant", "thought": { "thought_type": "...", "content": "..." }, "actions": [...] },
  { "role": "observation", "skill": "skill-name", "outcome": "success", "content": "..." }
]
```

Context is responsible for:

- Appending user, assistant, and observation messages
- Exporting messages in OpenAI-compatible format for LLM consumption

## Session Lifecycle

1. Transport receives user input
2. Session appends user message to Context
3. Session runs the agent loop (LLM → actions → harness → observations → loop)
4. Agent loop terminates (answer, max_steps, or interrupt)
5. Session returns the result to transport

## Persistence (User-Triggered)

Sessions live in memory by default. Users explicitly save sessions via commands (e.g. `/save`). Saved sessions are stored as JSON files:

```
~/.minusagent/sessions/
├── {session_id}.json
└── ...
```

```json
{
  "id": "uuid",
  "created_at": "ISO8601",
  "updated_at": "ISO8601",
  "messages": [ ... ],
  "config": {}
}
```

Without an explicit save, sessions are discarded when the process exits.