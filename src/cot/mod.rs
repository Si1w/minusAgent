use std::iter::once;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use serde_yaml::from_str;

use crate::context::{Action, Context, Message};
use crate::core::Node;
use crate::llm::Llm;

fn parse_yaml(content: &str) -> Result<Value> {
    let yaml_str = if let Some(start) = content.find("```yaml") {
        let start = start + 7;
        let end = content[start..].find("```").map(|i| start + i).unwrap_or(content.len());
        &content[start..end]
    } else {
        content
    };

    let value: Value = from_str(yaml_str.trim())?;
    Ok(value)
}

const PLAN_PROMPT: &str = r#"You are a planning assistant.

## Question
{question}

## Instructions
Break down the question into a todo list of atomic, actionable tasks.

Output EXACTLY ONE YAML block:

```yaml
todos:
  - task 1
  - task 2
action: continue
```"#;

const EXEC_PROMPT: &str = r#"You are an execution assistant.

## Question
{question}

## Current Progress
{context}

## Instructions
Complete the next task based on the todo list above. Update the remaining todos after completion.

Output EXACTLY ONE YAML block:

```yaml
current: the task you are working on
result: your result for this task
todos:
  - remaining task 1
  - remaining task 2
action: continue | stop
final: your final answer (if action is stop, this is required)
```

- Use "continue" if there are remaining tasks
- Use "stop" when all tasks are done, provide the final answer
- Quote strings containing colons with double quotes"#;

pub struct Plan {
    llm: Llm,
    system_prompt: String,
}

impl Plan {
    pub fn new(llm: Llm) -> Self {
        Self {
            llm,
            system_prompt: PLAN_PROMPT.to_string(),
        }
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }
}

#[async_trait]
impl Node for Plan {
    async fn prep(&mut self, ctx: &dyn Context) -> Result<Option<Value>> {
        let question = ctx.messages().first().map(|m| m.content.as_str()).unwrap_or("");
        let prompt = self.system_prompt.replace("{question}", question);
        let system = json!({"role": "system", "content": prompt});
        let context = ctx.to_vec();
        let messages: Vec<Value> = once(system).chain(context).collect();
        Ok(Some(json!(messages)))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut dyn Context) -> Result<()> {
        let content = exec_res
            .as_ref()
            .and_then(|r| r["choices"][0]["message"]["content"].as_str())
            .unwrap_or("");

        let parsed = parse_yaml(content).unwrap_or(json!({"action": "continue"}));
        let action_str = parsed["action"].as_str().unwrap_or("continue");
        match action_str {
            "continue" => ctx.set_action(Action::Continue),
            "stop" => ctx.set_action(Action::Stop),
            other => ctx.set_action(Action::CallTool(other.to_string())),
        }

        ctx.push(Message::assistant(content));
        Ok(())
    }
}

pub struct Execute {
    llm: Llm,
    system_prompt: String,
}

impl Execute {
    pub fn new(llm: Llm) -> Self {
        Self {
            llm,
            system_prompt: EXEC_PROMPT.to_string(),
        }
    }

    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }
}

#[async_trait]
impl Node for Execute {
    async fn prep(&mut self, ctx: &dyn Context) -> Result<Option<Value>> {
        let question = ctx.messages().first().map(|m| m.content.as_str()).unwrap_or("");
        let context_str = ctx.last().map(|m| m.content.as_str()).unwrap_or("");
        let prompt = self.system_prompt
            .replace("{question}", question)
            .replace("{context}", context_str);
        let system = json!({"role": "system", "content": prompt});
        let context = ctx.to_vec();
        // API requires last message to be user role, add a trigger message
        let trigger = json!({"role": "user", "content": "Continue"});
        let messages: Vec<Value> = once(system).chain(context).chain(once(trigger)).collect();
        Ok(Some(json!(messages)))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        self.llm.exec(prep_res).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut dyn Context) -> Result<()> {
        let content = exec_res
            .as_ref()
            .and_then(|r| r["choices"][0]["message"]["content"].as_str())
            .unwrap_or("");

        let parsed = parse_yaml(content).unwrap_or(json!({"action": "stop"}));
        let action_str = parsed["action"].as_str().unwrap_or("stop");

        match action_str {
            "continue" => ctx.set_action(Action::Continue),
            "stop" => ctx.set_action(Action::Stop),
            other => ctx.set_action(Action::CallTool(other.to_string())),
        }

        ctx.push(Message::assistant(content));
        Ok(())
    }
}

pub struct ChainOfThought {
    plan: Plan,
    execute: Execute,
}

impl ChainOfThought {
    pub fn new(plan: Plan, execute: Execute) -> Self {
        Self { plan, execute }
    }

    pub async fn run(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.plan.run(ctx).await?;

        loop {
            self.execute.run(ctx).await?;

            if *ctx.action() == Action::Stop {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::context::ChatContext;

    #[tokio::test]
    #[ignore]
    async fn test_cot_run() -> Result<()> {
        dotenvy::dotenv().ok();

        let base_url = env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-2508".to_string());
        let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY required");

        let llm1 = Llm::new(&base_url, &model, &api_key);
        let llm2 = Llm::new(&base_url, &model, &api_key);
        let plan = Plan::new(llm1);
        let execute = Execute::new(llm2);
        let mut cot = ChainOfThought::new(plan, execute);

        let mut ctx = ChatContext::new();
        ctx.push(Message::user("What is 15 + 27?"));
        cot.run(&mut ctx).await?;

        println!("Action: {:?}", ctx.action());
        for (i, msg) in ctx.messages().iter().enumerate() {
            println!("--- Message {} ---\n{}", i, msg.content);
        }
        assert_eq!(ctx.action(), &Action::Stop);
        Ok(())
    }
}
