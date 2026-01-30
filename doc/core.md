# core

Defines the `Node` trait — the fundamental abstraction of the framework.

## Node (trait)

Every processing unit implements three async phases:

### `prep(ctx: &dyn Context) -> Result<Option<Value>>`

Gather input from context. Returns optional JSON data for the next phase.

| Parameter | Type | Description |
|-----------|------|-------------|
| `ctx` | `&dyn Context` | Read-only access to current conversation state |

### `exec(prep_res: Option<Value>) -> Result<Option<Value>>`

Perform the actual work using data from `prep()`.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prep_res` | `Option<Value>` | Output from `prep()`, `None` if prep produced nothing |

### `post(prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut dyn Context) -> Result<()>`

Write results back to context.

| Parameter | Type | Description |
|-----------|------|-------------|
| `prep_res` | `Option<Value>` | Output from `prep()` |
| `exec_res` | `Option<Value>` | Output from `exec()` |
| `ctx` | `&mut dyn Context` | Mutable access to conversation state |

### `run(ctx: &mut dyn Context) -> Result<()>`

Default orchestrator. Calls prep → exec → post in sequence.
