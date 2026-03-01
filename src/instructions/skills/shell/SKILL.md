---
name: shell
description: Execute whitelisted programs with structured parameters. Use when you need to run system commands like git, cargo, file operations, or build tools.
---
After activating this skill, use the `Execute` action with a shell command string to run commands.

```json
{
  "action": "Execute",
  "command": "git add . && git commit -m 'fix'"
}
```

Allowed programs: git, cargo, cat, ls, mkdir, cp, mv, head, tail, find, grep, echo, touch, rustc

Commands can be chained with `&&` to run sequentially (stops on failure).
