# Harness

## Definition

A harness is the execution layer between the agent and the environment. Skills are executed through the harness, which follows the Node trait pipeline (prep → exec → post).

The harness is responsible for:

- Looking up the requested skill in the registry
- Loading the skill body (progressive disclosure)
- Executing the skill (scripts, commands, etc.)
- Capturing the result as an Outcome
- Sandboxing and permission control
- I/O routing (stdin/stdout, file system, network)
- Resource limits (timeout, memory)

## Node Pipeline

- **prep**: Resolve the skill from the registry, load full SKILL.md body
- **exec**: Run the skill's scripts or commands against the environment
- **post**: Package the execution result into an Outcome (Success or Failure)

## Separation from Skills

The harness is separate from skills — the same skill can run in different harnesses (local shell, Docker, remote server).
