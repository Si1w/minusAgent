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
| **Continue** | `{"action": "continue"}` | Insert synthetic user "continue" message, loop again |
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

`Action` is the unified signal throughout the pipeline. The Agent does NOT own the Harness. Responsibility split:

- **Agent handles internally**: `UseSkill` (load instructions via PromptEngine), `Continue` (insert synthetic user message, loop)
- **Agent returns to Session**: `Execute`, `Completed` (including LLM errors and max steps)
- **Session dispatches**: runs Harness (via `Node::run()`) for `Execute`, re-enters Agent loop

`Agent::run()` drives the inner CoT loop. `Session::turn()` drives one orchestrator turn (Agent ↔ Harness dispatch). The REPL loop lives in the transport.

## Continue & Message Ordering

Many LLM APIs (Mistral, etc.) require the last message to be `user` or `tool` role. When the agent loops on `Continue`, the last context message is `assistant`. The agent inserts a synthetic `{"role": "user", "content": "continue"}` message to satisfy this constraint before the next LLM call.

## Termination Conditions

- `Completed` action — Agent returns to Session, Session returns the answer
- `max_steps` reached — Agent returns `Completed { answer: "max steps reached" }`
- User interrupts — agent stops immediately (session-level signal)

## Node Pipeline

The `Node` trait drives a prep → exec → post pipeline:

```rust
trait Node {
    fn prep(&mut self, shared: &Context) -> Result<Value, String>;
    fn exec(&mut self, prep_res: Value) -> Result<Value, String>;
    fn post(&mut self, shared: &mut Context, prep_res: Value, exec_res: Value) -> Action;
}
```

- `prep`: Read and preprocess data from shared store.
- `exec`: Pure compute (LLM calls, shell commands). No access to shared.
- `post`: Write results back to shared, return `Action` for flow control.
- `run` (default): Drives prep → exec → post. Short-circuits to `Action::Completed` on error.

Harness implements `Node`: prep validates the command, exec runs `sh -c`, post writes the observation to context and returns `Action::Continue`.

## Outcome

Result stored in conversation history for observations:

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