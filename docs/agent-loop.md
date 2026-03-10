# Agent Loop (ReAct)

## Flow

```
User Input → Session
  → Agent Loop: [LLM Think → Action] × N
    UseSkill  → load SKILL.md instructions into context → continue loop
    Continue  → pure thinking → continue loop
    Execute   → return to Session → Harness runs command → observe → re-enter loop
    Completed → return answer to Session
```

## LLM Structured Output

Each LLM step produces:

```json
{
  "thought": {
    "thought_type": "planning | analysis | decision_making | problem_solving | memory_integration | self_reflection | goal_setting | prioritization",
    "content": "CoT reasoning..."
  },
  "action": { ... }
}
```

### Action Variants

| Action | JSON | Agent Behavior |
|---|---|---|
| **UseSkill** | `{"action": "use_skill", "skills": [{"skill": "name", "input": {...}}]}` | Load skill instructions into context, continue loop |
| **Execute** | `{"action": "execute", "command": "shell command"}` | Return to Session for harness execution |
| **Continue** | `{"action": "continue"}` | Pure thinking, loop again without side effects |
| **Completed** | `{"action": "completed", "answer": "final response"}` | Return answer to Session |

- `skills[].input` is optional — omit when the skill needs no arguments.
- `UseSkill` supports multiple skills in one action.

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

## Agent ↔ Session Boundary

The Agent does NOT own the Harness. Responsibility split:

- **Agent handles internally**: `UseSkill` (load instructions), `Continue` (loop)
- **Agent returns to Session**: `Execute` (command), `Completed` (answer), `MaxSteps`, `Error`
- **Session dispatches**: runs Harness for `Execute`, adds observation to Context, re-enters Agent

```rust
enum AgentResult {
    Answer(String),
    Execute { command: String },
    MaxSteps,
    Error(String),
}
```

## Termination Conditions

- `Completed` action — agent returns `Answer`
- `max_steps` reached — agent returns `MaxSteps`
- User interrupts — agent stops immediately (session-level signal)

## Outcome

Result of a single command execution (from Harness):

```rust
enum Outcome {
    Success { output: String },
    Failure { error: String },
}
```

## Observation

What gets fed back to the LLM after skill loading or command execution:

```json
{
  "role": "observation",
  "skill": "skill-name",
  "outcome": "success | failure",
  "content": "skill instructions, command output, or error message"
}
```

All `thought` entries are recorded in the session message history for full CoT traceability.

## Error Handling

| Type | Trigger | Behavior |
|---|---|---|
| **User Interrupt** | User pauses or cancels | Session-level signal, agent stops immediately, session is preserved |
| **Environment Failure** | Command execution fails, network error, etc. | Observation with `Failure` outcome fed back to LLM for re-decision |

The agent NEVER retries silently. On environment failure, the LLM sees the error and decides the next action.