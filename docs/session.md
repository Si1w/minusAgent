# Session

## Definition

A session represents one continuous agent interaction. It holds:

- Conversation history (messages between user, agent, and environment)
- Active skill states
- Configuration snapshot

## Message Format

```json
[
  { "role": "user", "content": "..." },
  { "role": "assistant", "thought": { "thought_type": "...", "content": "..." }, "actions": [...] },
  { "role": "observation", "skill": "skill-name", "outcome": "success", "content": "..." }
]
```

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