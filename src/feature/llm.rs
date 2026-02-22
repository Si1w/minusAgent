use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::context::{Context, Thought, ThoughtType};
use crate::core::prompt::PromptEngine;
use crate::core::{Action, Node};

pub struct LLM {
    client: Client,
    model: String,
    base_url: String,
    api_key: String,
    max_tokens: usize,
}

impl LLM {
    pub fn new(model: String, base_url: String, api_key: String, max_tokens: usize) -> Self {
        LLM {
            client: Client::new(),
            model,
            base_url,
            api_key,
            max_tokens,
        }
    }

    pub async fn message(&self, system_prompt: String, user_prompt: String) -> Result<Option<Value>> {
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ]
        });
        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(Some(response.json::<Value>().await?))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error ({}): {}", status, body)
        }
    }
}

#[async_trait]
impl Node for LLM {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        let system_prompt = ctx.system_prompt.clone();
        let user_prompt = PromptEngine::new(ctx.clone()).render();
        Ok(Some(json!({
            "system_prompt": system_prompt,
            "user_prompt": user_prompt
        })))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        let prep = prep_res.ok_or_else(|| anyhow::anyhow!("prep result is None"))?;
        let system_prompt = prep["system_prompt"].as_str().unwrap_or_default().to_string();
        let user_prompt = prep["user_prompt"].as_str().unwrap_or_default().to_string();
        self.message(system_prompt, user_prompt).await
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        let exec = exec_res.ok_or_else(|| anyhow::anyhow!("exec result is None"))?;
        let content = exec["choices"][0]["message"]["content"].as_str().unwrap_or_default();
        let parsed: Value = serde_json::from_str(content)?;

        let thought_type = match parsed["thought_type"].as_str().unwrap_or_default() {
            "Planning" => ThoughtType::Planning,
            "Solving" => ThoughtType::Solving,
            "GoalSetting" => ThoughtType::GoalSetting,
            _ => ThoughtType::None,
        };
        let thought = Thought {
            thought_type,
            content: parsed["thought"].as_str().map(String::from),
        };
        let action = match parsed["action"].as_str().unwrap_or_default() {
            "Running" => Action::Running,
            "Completed" => Action::Completed,
            _ => Action::Pending,
        };
        let observation = parsed["observation"].as_str().unwrap_or_default().to_string();

        ctx.log_trajectory(thought, action.clone(), observation);
        Ok(action)
    }
}