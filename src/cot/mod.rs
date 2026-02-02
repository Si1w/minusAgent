use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::context::{Action, Context};
use crate::core::Node;
use crate::llm::Llm;
use crate::utils::parse_action;

const PLAN_PROMPT: &str = r#"You are a planning assistant.

## Question
{question}

## Instructions
Break down the question into a clear todo list with actionable tasks.
- You have at most {max_turns} turns to complete all tasks. Plan efficiently.

Output EXACTLY ONE JSON block:

{
  "task": "the first task to execute",
  "thinking": "your reasoning about how to approach this question",
  "todos": ["task 1", "task 2"],
  "action": "continue/stop"
}

- "todos" is a list of ALL tasks to execute (including the first one), with at most {max_turns} items
- Use "continue" if there are tasks to execute
- Use "stop" if the answer is immediately obvious, and include "answer" as a string"#;

const THINKING_PROMPT: &str = r#"You are a thinking assistant.

## Question
{question}

## Current Task
{task}

## Remaining Tasks
{todos}

## Previous Thinking
{thinking}

## Instructions
Execute the current task. Update the remaining tasks and decide the next step.

Output EXACTLY ONE JSON block:

{
  "thinking": "your detailed reasoning and result for the current task",
  "todos": ["remaining task 1", "remaining task 2"],
  "answer": "the final answer string, only present when action is stop",
  "action": "continue/stop"
}

- "todos" contains ONLY the remaining unfinished tasks (remove the current task once done)
- Use "continue" if there are remaining tasks
- Use "stop" when all tasks are done, and include "answer" as a string"#;

pub struct Thought {
    llm: Llm,
}

impl Thought {
    pub fn new(llm: Llm) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Node for Thought {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        Ok(Some(ctx.to_prompt()))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        ctx.action = parse_action(&exec_res, ctx);
        Ok(ctx.action.clone())
    }
}

const DEFAULT_MAX_TURNS: usize = 10;

pub struct ChainOfThought {
    thought: Thought,
    plan_prompt: String,
    thinking_prompt: String,
    max_turns: usize,
}

impl ChainOfThought {
    pub fn new(llm: Llm) -> Self {
        Self {
            thought: Thought::new(llm),
            plan_prompt: PLAN_PROMPT.to_string(),
            thinking_prompt: THINKING_PROMPT.to_string(),
            max_turns: DEFAULT_MAX_TURNS,
        }
    }

    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<Action> {
        let question = ctx.user_message.as_deref().unwrap_or("").to_string();
        let max_turns_str = self.max_turns.to_string();

        ctx.set_system_prompt(
            self.plan_prompt
                .replace("{question}", &question)
                .replace("{max_turns}", &max_turns_str),
        );
        self.thought.run(ctx).await?;

        let mut turn = 0;
        loop {
            if ctx.action != Action::Continue || turn >= self.max_turns {
                return Ok(ctx.action.clone());
            }

            let last = ctx.last_content().expect("node should produce output").clone();

            // Remove last CoT response, keep prior conversation history
            ctx.history.pop();

            let thinking = last["thinking"].as_str().unwrap_or("").to_string();
            let next_task = last["todos"][0].as_str().unwrap_or("");
            let todos_str = last["todos"].to_string();

            ctx.set_system_prompt(
                self.thinking_prompt
                    .replace("{question}", &question)
                    .replace("{task}", next_task)
                    .replace("{todos}", &todos_str)
                    .replace("{thinking}", &thinking),
            );
            self.thought.run(ctx).await?;
            turn += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_cot_run() -> Result<()> {
        dotenvy::dotenv().ok();

        let base_url = env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-2508".to_string());
        let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY required");

        let llm = Llm::new(&base_url, &model, &api_key);
        let mut cot = ChainOfThought::new(llm);

        let mut ctx = Context::new();
        ctx.set_user_message("What is 15 + 27?");
        let action = cot.run(&mut ctx).await?;
        assert_eq!(action, Action::Stop);
        println!("# final context: {:?}", ctx);
        
        let last = ctx.last_content().expect("should have final answer");
        let answer = last["answer"].as_str().unwrap_or("");
        assert!(answer.contains("42"), "expected answer to contain 42, got: {}", answer);

        Ok(())
    }
}
