use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::core::{Action, Context, Message, Node, Skill, prompt};
use super::llm::Llm;
use super::utils::parse_action;

const PLAN_SKILL: &str = include_str!("../skills/plan/SKILL.md");
const THINKING_SKILL: &str = include_str!("../skills/thinking/SKILL.md");

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
        Ok(Some(prompt::render(ctx)))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        let raw = exec_res
            .as_ref()
            .and_then(|r| r["choices"][0]["message"]["content"].as_str())
            .unwrap_or("");
        let (action, body) = parse_action(raw);
        ctx.push_history(Message::assistant(Value::String(body.to_string())));
        ctx.action = action;
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
        let plan_skill = Skill::parse(PLAN_SKILL, "plan").expect("invalid plan skill");
        let thinking_skill = Skill::parse(THINKING_SKILL, "thinking").expect("invalid thinking skill");

        Self {
            thought: Thought::new(llm),
            plan_prompt: plan_skill.script,
            thinking_prompt: thinking_skill.script,
            max_turns: DEFAULT_MAX_TURNS,
        }
    }

    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    pub async fn run(&mut self, ctx: &mut Context) -> Result<Action> {
        // Save original question to history for context
        if let Some(ref q) = ctx.user_message {
            ctx.push_history(Message::user(q.clone()));
        }

        // First turn: planning
        ctx.set_system_prompt(&self.plan_prompt);
        self.thought.run(ctx).await?;

        let mut turn = 0;
        loop {
            if ctx.action != Action::Continue || turn >= self.max_turns {
                return Ok(ctx.action.clone());
            }

            let last = ctx.last_content().expect("node should produce output").clone();
            let prev_content = last.as_str().unwrap_or("").to_string();

            // Pop last response, inject as context for next turn
            ctx.history.pop();
            ctx.set_system_prompt(&self.thinking_prompt);
            ctx.set_user_message(&prev_content);

            self.thought.run(ctx).await?;
            turn += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::core::Context;

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
        let answer = last.as_str().unwrap_or("");
        assert!(answer.contains("42"), "expected answer to contain 42, got: {}", answer);

        Ok(())
    }
}
