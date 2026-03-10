# Harness

## Definition

A harness is the execution layer between the agent and the environment. It implements the `Node` trait pipeline (prep → exec → post) to run shell commands.

The harness is responsible for:

- Validating commands against blocked patterns (dangerous command blocking)
- Executing shell commands via `sh -c`
- Writing execution results as observations to the shared context
- Sandboxing and permission control (future)
- Resource limits: timeout, memory (future)

## Node Pipeline

- **prep**: Validate that a command is set and not blocked. Returns the command string.
- **exec**: Spawn `sh -c` subprocess. Pure compute, no access to shared context.
- **post**: Write the stdout as an `Outcome::Success` observation to context. Return `Action::Continue`.

On prep/exec failure, `Node::run()` short-circuits and returns `Action::Completed` with the error message. Session then records this as an `Outcome::Failure` observation.

## Blocked Patterns

Commands matching these patterns are rejected at prep:

- `rm -rf /`, `mkfs`, `> /dev/sd`, `dd if=`

## Separation from Skills

The harness is separate from skills — the same skill can run in different harnesses (local shell, Docker, remote server).
