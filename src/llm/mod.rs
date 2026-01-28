use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::{Context, Node, Result, Status};

pub struct LlmNode {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    response: Option<Value>,
}

impl LlmNode {
    pub fn new(base_url: &str, model: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
            response: None,
        }
    }
}

#[async_trait]
impl Node for LlmNode {
    async fn prep(&mut self, _ctx: &Context) -> Result {
        Result {
            status: Status::Success,
            value: None,
            error: None,
        }
    }

    async fn exec(&mut self, ctx: &Context) -> Result {
        let body = json!({
            "model": self.model,
            "messages": ctx
        });

        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp {
            Ok(r) => match r.json::<Value>().await {
                Ok(json) => {
                    self.response = Some(json);
                    Result {
                        status: Status::Success,
                        value: None,
                        error: None,
                    }
                }
                Err(e) => Result {
                    status: Status::Failed,
                    value: None,
                    error: Some(e.to_string()),
                },
            },
            Err(e) => Result {
                status: Status::Failed,
                value: None,
                error: Some(e.to_string()),
            },
        }
    }

    async fn post(&mut self, ctx: &mut Context) -> Result {
        if let Some(resp) = self.response.take() {
            let content = resp["choices"][0]["message"].clone();
            ctx.push(content);
            Result {
                status: Status::Success,
                value: Some(resp),
                error: None,
            }
        } else {
            Result {
                status: Status::Failed,
                value: None,
                error: Some("No response".to_string()),
            }
        }
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

        let result = node.run(&mut ctx).await;

        assert_eq!(result.status, Status::Success);
        assert!(ctx.len() > 1);
        println!("Response: {:?}", ctx.last());
    }
}