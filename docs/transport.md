# Transport Layer

## Definition

Thin adapter between external platforms and Session. Each transport only needs to:

- Receive user input → call `session.send(input)`
- Receive session output → deliver to platform

No heavy abstraction needed — a transport is just a loop that bridges I/O.

## Transports

- **CLI**: MVP, interactive REPL
- **Discord / HTTP**: Future, same session interface