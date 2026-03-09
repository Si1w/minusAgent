# Harness

## Definition

A harness is the execution environment that runs skills. It provides:

- Sandboxing and permission control
- I/O routing (stdin/stdout, file system, network)
- Resource limits (timeout, memory)
- Observation capture (skill output → agent context)

## Separation from Skills

The harness is separate from skills — the same skill can run in different harnesses (local shell, Docker, remote server).