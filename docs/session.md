# Session

## Definition

A session is the top-level orchestrator for a multi-turn conversation. It coordinates:

- **Context**: Conversation history (managed by `Context`)
- **Agent**: ReAct loop execution
- **Harness**: Skill execution environment
- **Config**: Configuration snapshot

Session receives user input from the transport layer, drives the agent loop, and emits progress events via callbacks.

## Context (Message History)

Context manages the ordered message history. Each message has one of three roles:

```json
[
  { "role": "user", "content": "..." },
  { "role": "assistant", "thought": { "thought_type": "...", "content": "..." }, "action": { ... } },
  { "role": "observation", "skill": "skill-name", "outcome": "success", "content": "..." }
]
```

Context is responsible for:

- Appending user, assistant, and observation messages
- Exporting messages in OpenAI-compatible format for LLM consumption

## Session Lifecycle

1. Transport receives user input
2. Session appends user message to Context
3. Session calls `Agent::run()`, which drives the inner loop:
   - `UseSkill` → PromptEngine loads SKILL.md into Context → continue loop
   - `Continue` → continue loop
   - `Execute` → return to Session
   - `Completed` → return to Session
   - `max_steps` reached → return `Completed { answer: "max steps reached" }`
4. Session dispatches on the returned `Action`, emitting `Event` callbacks:
   - `Execute` → emit `Executing` → run Harness → emit `Output` → goto 3
   - `Completed` → return answer string to transport
5. Transport displays the answer

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