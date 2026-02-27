use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::config::LLMConfig;
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

    pub fn from_config(config: &LLMConfig) -> Self {
        Self::new(
            config.model.clone(),
            config.base_url.clone(),
            config.api_key.clone(),
            config.max_tokens(),
        )
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
        let raw = exec["choices"][0]["message"]["content"].as_str().unwrap_or_default();
        let content = raw.trim()
            .strip_prefix("```json").or_else(|| raw.trim().strip_prefix("```"))
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(raw.trim());
        let parsed: Value = serde_json::from_str(content)?;

        let thought_type = match parsed["thought"]["thought_type"].as_str().unwrap_or_default() {
            "Planning" => ThoughtType::Planning,
            "Solving" => ThoughtType::Solving,
            "GoalSetting" => ThoughtType::GoalSetting,
            _ => ThoughtType::None,
        };
        let response = parsed["thought"]["content"].as_str().unwrap_or_default().to_string();
        let thought = Thought {
            thought_type,
            content: Some(response),
        };
        let action = match parsed["action"].as_str().unwrap_or_default() {
            "Running" => Action::Running,
            "Completed" => Action::Completed,
            "Execute" => {
                let cmd = parsed["command"].as_str().map(|s| s.to_string());
                Action::Execute(cmd)
            }
            "UseSkill" => {
                let names = parsed["skills"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                Action::UseSkill(names)
            }
            _ => Action::Pending,
        };

        let answer = parsed["answer"].as_str().map(|s| s.to_string());
        ctx.log_trajectory(thought, action.clone(), None, answer);
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::LLMConfig;

    const SYSTEM_PROMPT: &str = include_str!("../instructions/system_prompt.md");

    #[tokio::test]
    async fn test_message() {
        let config = LLMConfig::load(None).unwrap();
        let llm = LLM::from_config(&config);
        let result = llm.message(SYSTEM_PROMPT.to_string(), "Say hello.".to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_node_run() {
        let config = LLMConfig::load(None).unwrap();
        let mut llm = LLM::from_config(&config);
        let mut ctx = Context::new(SYSTEM_PROMPT.to_string());
        ctx.init_trajectory("Say hello.".to_string());
        let action = llm.run(&mut ctx).await;
        assert!(action.is_ok());
        assert!(ctx.trajectories.len() >= 2);
    }
}

