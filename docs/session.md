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
3. Session calls Agent (via ContextGuard for overflow protection)
4. Agent returns `AgentResult`:
   - `Answer` → Session returns result to transport
   - `Execute` → Session runs Harness, adds observation to Context, goto 3
   - `MaxSteps` / `Error` → Session handles error
5. Session returns the result to transport

## Context Guard

Session uses `ContextGuard` (wrapping `LLMClient`) for overflow protection:

- **Proactive**: after a successful call, if token usage > 80% of context window, compact
- **Reactive**: on overflow error, three-stage recovery:
  1. Truncate long observation content
  2. Compact older messages via LLM summarization
  3. Fail if still overflowing after 3 retries

Compact replaces older messages with a summary + acknowledgment pair, preserving recent context.

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