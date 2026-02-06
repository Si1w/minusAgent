# core

Defines the `Node` trait — the fundamental abstraction of the framework.

## Node (trait)

Every processing unit implements three async phases:

### `prep(ctx: &Context) -> Result<Option<Value>>`

Gather input from context. Returns optional JSON data for the next phase.

| Parameter | Type | Description |
|-----------|------|-------------|
| `ctx` | `&Context` | Read-only access to current conversation state |

### `exec(prep_res: Option<Value>) -> Result<Option<Value>>`

Perform the actual work using data from `prep()`.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prep_res` | `Option<Value>` | Output from `prep()`, `None` if prep produced nothing |

### `post(prep_res, exec_res, ctx) -> Result<Action>`

Write results back to context and determine next action.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prep_res` | `Option<Value>` | Output from `prep()` |
| `exec_res` | `Option<Value>` | Output from `exec()` |
| `ctx` | `&mut Context` | Mutable access to conversation state |

### `run(ctx: &mut Context) -> Result<Action>`

Default orchestrator. Calls prep → exec → post in sequence.
