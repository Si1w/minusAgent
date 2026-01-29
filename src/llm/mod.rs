use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::{Context, Node};

pub struct LlmNode {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl LlmNode {
    pub fn new(base_url: &str, model: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl Node for LlmNode {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        Ok(Some(json!(ctx)))
    }

    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>> {
        let messages = prep_res.ok_or_else(|| anyhow::anyhow!("No messages provided"))?;

        let body = json!({
            "model": self.model,
            "messages": messages
        });

        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(Some(resp))
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<()> {
        if let Some(resp) = exec_res {
            let content = resp["choices"][0]["message"].clone();
            ctx.push(content);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_node() {
        dotenvy::dotenv().ok();

        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-2508".to_string());
        let api_key = std::env::var("LLM_API_KEY").expect("LLM_API_KEY required");

        let mut node = LlmNode::new(&base_url, &model, &api_key);
        let mut ctx: Context = vec![json!({"role": "user", "content": "Say hello"})];

        node.run(&mut ctx).await.unwrap();
        assert!(ctx.len() > 1);
    }
}