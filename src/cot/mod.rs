use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::context::{Action, Context, Message};
use crate::core::Node;
use crate::llm::Llm;
use crate::utils::parse_content;

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
  "todos": "- [ ] task 1\n- [ ] task 2",
  "action": "continue"
}

- Use "continue" if there are tasks to execute
- Use "stop" if the answer is immediately obvious (include "answer" field)"#;

const THINKING_PROMPT: &str = r#"You are a thinking assistant.

## Question
{question}

## Current Task
{task}

## Todo List
{todos}

## Previous Thinking
{thinking}

## Instructions
Execute the current task. Update the todo list and decide the next step.

Output EXACTLY ONE JSON block:

{
  "task": "next task to execute",
  "thinking": "your detailed reasoning and result for the current task",
  "todos": "- [x] done task\n- [ ] remaining task",
  "action": "continue"
}

- Use "continue" if there are remaining tasks
- Use "stop" when all tasks are done (include "answer" field with the final answer)"#;

pub struct PlanNode {
    llm: Llm,
}

impl PlanNode {
    pub fn new(llm: Llm) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Node for PlanNode {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        Ok(Some(json!(ctx.to_messages())))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        let content = exec_res
            .as_ref()
            .and_then(|r| r["choices"][0]["message"]["content"].as_str())
            .unwrap_or("");

        ctx.action = match parse_content(content) {
            Ok(parsed) if parsed.is_object() && parsed.get("action").is_some() => {
                ctx.push_history(Message::assistant(parsed.to_string()));
                match parsed["action"].as_str().unwrap_or("stop") {
                    "continue" => Action::Continue,
                    "stop" => Action::Stop,
                    other => Action::CallTool(other.to_string()),
                }
            }
            _ => {
                ctx.push_history(Message::assistant(content.to_string()));
                Action::Stop
            }
        };
        Ok(ctx.action.clone())
    }
}

pub struct ThinkingNode {
    llm: Llm,
}

impl ThinkingNode {
    pub fn new(llm: Llm) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Node for ThinkingNode {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        Ok(Some(json!(ctx.to_messages())))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        let content = exec_res
            .as_ref()
            .and_then(|r| r["choices"][0]["message"]["content"].as_str())
            .unwrap_or("");

        ctx.action = match parse_content(content) {
            Ok(parsed) if parsed.is_object() && parsed.get("action").is_some() => {
                ctx.push_history(Message::assistant(parsed.to_string()));
                match parsed["action"].as_str().unwrap_or("stop") {
                    "continue" => Action::Continue,
                    "stop" => Action::Stop,
                    other => Action::CallTool(other.to_string()),
                }
            }
            _ => {
                ctx.push_history(Message::assistant(content.to_string()));
                Action::Stop
            }
        };
        Ok(ctx.action.clone())
    }
}

const DEFAULT_MAX_TURNS: usize = 10;

pub struct ChainOfThought {
    plan_node: PlanNode,
    thinking_node: ThinkingNode,
    plan_prompt: String,
    thinking_prompt: String,
    max_turns: usize,
}

impl ChainOfThought {
    pub fn new(plan_node: PlanNode, thinking_node: ThinkingNode) -> Self {
        Self {
            plan_node,
            thinking_node,
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

        // First plan: standalone call with user query
        ctx.set_system_prompt(
            self.plan_prompt
                .replace("{question}", &question)
                .replace("{max_turns}", &max_turns_str),
        );
        self.plan_node.run(ctx).await?;

        // Thinking loop
        let mut turn = 0;
        loop {
            if ctx.action != Action::Continue || turn >= self.max_turns {
                return Ok(ctx.action.clone());
            }

            let last = ctx.last_content()
                .and_then(|c| serde_json::from_str::<Value>(c).ok())
                .unwrap_or(json!({}));

            let task = last["task"].as_str().unwrap_or("");
            let todos = last["todos"].as_str().unwrap_or("");
            let thinking = last["thinking"].as_str().unwrap_or("");

            ctx.set_system_prompt(
                self.thinking_prompt
                    .replace("{question}", &question)
                    .replace("{task}", task)
                    .replace("{todos}", todos)
                    .replace("{thinking}", thinking),
            );
            self.thinking_node.run(ctx).await?;
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
        let plan_node = PlanNode::new(llm.clone());
        let thinking_node = ThinkingNode::new(llm);
        let mut cot = ChainOfThought::new(plan_node, thinking_node);

        let mut ctx = Context::new();
        ctx.set_user_message("What is 15 + 27?");
        let action = cot.run(&mut ctx).await?;
        assert_eq!(action, Action::Stop);

        let last = ctx.last_content().expect("should have final answer");
        let parsed: Value = serde_json::from_str(last).expect("should be valid JSON");
        let answer = parsed["answer"].as_str().unwrap_or(last);
        assert!(answer.contains("42"), "expected answer to contain 42, got: {}", answer);

        Ok(())
    }
}
