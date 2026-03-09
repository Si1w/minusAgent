# Skill System

## Definition

A skill is the unit of capability. All tools, MCP servers, and custom instructions are wrapped as skills. A skill is NOT a harness — skills define *what* can be done, harnesses define *how* it runs.

Skills follow the [Agent Skills Specification](https://agentskills.io/specification).

## Structure

```
skill-name/
├── SKILL.md          # Required: frontmatter (name, description) + instructions
├── scripts/          # Optional: executable code
├── references/       # Optional: extra documentation
└── assets/           # Optional: static resources
```

SKILL.md format:

```yaml
---
name: skill-name
description: What this skill does and when to use it.
---

Instructions for the agent when this skill is activated.
```

## Progressive Disclosure

1. **Startup**: Load `name` + `description` from all registered skills (~100 tokens each)
2. **Activation**: When agent selects a skill, load full SKILL.md body
3. **Execution**: Load scripts/references/assets only as needed

## Registration

Skills are discovered from multiple sources (searched in order):

1. **Project-local**: `.minusagent/skills/` in the working directory
2. **User-global**: `~/.minusagent/skills/`
3. **Built-in**: Bundled with the binary

## MCP as Skill

MCP servers are wrapped as skills. The skill's SKILL.md describes the MCP server, and the skill runtime handles MCP protocol communication.

<!-- TODO: Design MCP protocol integration details during implementation -->