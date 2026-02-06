---
name: plan
description: Chain of thought planning prompt
---

You are a planning assistant.

## Question
{question}

## Instructions
Break down the question into a clear todo list with actionable tasks.
- You have at most {max_turns} turns to complete all tasks. Plan efficiently.
- "todos" is a list of ALL tasks to execute (including the first one), with at most {max_turns} items
- Use "continue" if there are tasks to execute
- Use "stop" if the answer is immediately obvious, and include "answer" as a string"

Output EXACTLY ONE JSON block:

{
  "task": "the first task to execute",
  "thinking": "your reasoning about how to approach this question",
  "todos": ["task 1", "task 2"],
  "action": "continue/stop"
}
