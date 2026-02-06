use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use super::utils::process_sse_stream;
use crate::core::{Action, Context, Message, Node};

pub type StreamCallback = Box<dyn Fn(&str) + Send + Sync>;

#[derive(Clone)]
pub struct Llm {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl Llm {
    pub fn new(base_url: &str, model: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn exec_stream(
        &self,
        prep_res: Option<Value>,
        on_chunk: StreamCallback,
    ) -> Result<Option<Value>> {
        let messages = prep_res.ok_or_else(|| anyhow::anyhow!("No messages provided"))?;

        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true
        });

        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        let (full_content, interrupted) = process_sse_stream(resp, |c| on_chunk(c)).await?;

        Ok(Some(json!({
            "choices": [{
                "message": {
                    "content": full_content
                }
            }],
            "interrupted": interrupted
        })))
    }
}

#[async_trait]
impl Node for Llm {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>> {
        Ok(Some(ctx.to_prompt()))
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
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        let json = resp.json::<Value>().await?;
        Ok(Some(json))
    }

    async fn post(&mut self, _prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action> {
        if let Some(resp) = exec_res {
            let content = &resp["choices"][0]["message"]["content"];
            if !content.is_null() {
                ctx.push_history(Message::assistant(content.clone()));
            }
        }
        Ok(Action::Continue)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Arc, Mutex};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_llm_node() -> Result<()> {
        dotenvy::dotenv().ok();

        let base_url = env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-latest".to_string());
        let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY required");

        let mut node = Llm::new(&base_url, &model, &api_key);
        let mut ctx = Context::new();
        ctx.push_history(Message::user("Say hello"));

        node.run(&mut ctx).await?;

        let content = ctx.last_content()
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(content.to_lowercase().contains("hello"), "Response should contain 'hello'");

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_llm_stream() -> Result<()> {
        dotenvy::dotenv().ok();

        let base_url = env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-latest".to_string());
        let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY required");

        let node = Llm::new(&base_url, &model, &api_key);
        let chunks = Arc::new(Mutex::new(Vec::new()));
        let chunks_clone = chunks.clone();

        let callback: StreamCallback = Box::new(move |chunk| {
            print!("{}", chunk);
            chunks_clone.lock().unwrap().push(chunk.to_string());
        });

        let messages = json!([{"role": "user", "content": "Say hello in 3 words"}]);
        let result = node.exec_stream(Some(messages), callback).await?;

        assert!(!chunks.lock().unwrap().is_empty());

        let resp = result.unwrap();
        let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("");
        assert!(content.to_lowercase().contains("hello"), "Response should contain 'hello'");
        assert!(!resp["interrupted"].as_bool().unwrap_or(true), "Should not be interrupted");

        Ok(())
    }
}
