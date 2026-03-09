# Agent Loop (ReAct)

## Flow

```
User Input → [LLM Think → Skill Execute → Observe] × N → Answer
```

## LLM Structured Output

Each LLM step produces:

```json
{
  "thought": {
    "thought_type": "planning | analysis | decision_making | problem_solving | memory_integration | self_reflection | goal_setting | prioritization",
    "content": "CoT reasoning..."
  },
  "actions": [
    { "skill": "skill-name", "input": { ... } },
    { "skill": "another-skill", "input": { ... } }
  ],
  "answer": "only present when actions is empty"
}
```

- `actions` non-empty → execute skill(s), feed observations back, loop
- `actions` empty → task complete, return `answer`

### Thought Types

| Type | Example |
|---|---|
| planning | "I need to break this task into three steps..." |
| analysis | "Based on the error message, the issue appears to be..." |
| decision_making | "Given the constraints, I should recommend..." |
| problem_solving | "To optimize this, I should first profile..." |
| memory_integration | "The user mentioned their preference earlier..." |
| self_reflection | "My last approach didn't work, I should try..." |
| goal_setting | "To complete this, I need to first establish..." |
| prioritization | "The security issue should be addressed before..." |

## Termination Conditions

- `actions` is empty — agent returns `answer`
- `max_steps` reached — agent returns error
- User interrupts — agent stops immediately (session-level signal)

## Outcome

Result of a single skill execution:

```rust
enum Outcome {
    Success { output: String },
    Failure { error: String },
}
```

No process states (Pending/Running) here — those belong to Session task tracking.

## Observation

What gets fed back to the LLM after skill execution:

```json
{
  "role": "observation",
  "skill": "skill-name",
  "outcome": "success | failure",
  "content": "skill output or error message"
}
```

All `thought` entries are recorded in the session message history for full CoT traceability.

## Error Handling

| Type | Trigger | Behavior |
|---|---|---|
| **User Interrupt** | User pauses or cancels | Session-level signal, agent stops immediately, session is preserved |
| **Environment Failure** | Skill execution fails, network error, etc. | Observation with `Failure` outcome fed back to LLM for re-decision |

The agent NEVER retries silently. On environment failure, the LLM sees the error and decides the next action.
