use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::core::{Action, Context, Message, Node};
use super::llm::Llm;
use super::skill::Skill;

const PLAN_SKILL: &str = include_str!("skills/plan/SKILL.md");
const THINKING_SKILL: &str = include_str!("skills/thinking/SKILL.md");

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

fn parse_action(exec_res: &Option<Value>, ctx: &mut Context) -> Action {
    let content = exec_res
        .as_ref()
        .and_then(|r| r["choices"][0]["message"]["content"].as_str())
        .unwrap_or("");
    let json_str = extract_json(content);
    let parsed = serde_json::from_str(json_str).unwrap_or(Value::String(content.to_string()));
    let action = match parsed["action"].as_str() {
        Some("continue") => Action::Continue,
        Some("stop") | None => Action::Stop,
        Some(other) => Action::CallTool(other.to_string()),
    };
    ctx.push_history(Message::assistant(parsed));
    action
}

fn extract_json(content: &str) -> &str {
    if let Some(start) = content.find("```") {
        let after = &content[start + 3..];
        let json_start = after.find('\n').map(|i| i + 1).unwrap_or(0);
        let inner = &after[json_start..];
        if let Some(end) = inner.find("```") {
            return inner[..end].trim();
        }
    }
    content
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
        let answer = last["answer"].as_str().unwrap_or("");
        assert!(answer.contains("42"), "expected answer to contain 42, got: {}", answer);

        Ok(())
    }
}
