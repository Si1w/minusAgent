use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::config::LlmConfig;
use crate::core::{Context, Node, Outcome};

pub struct LlmNode {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    response: Option<Value>,
}

impl LlmNode {
    pub fn from_config(config: &LlmConfig) -> Result<Self, String> {
        let api_key = config.api_key()?;
        Ok(Self {
            client: Client::new(),
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            api_key,
            response: None,
        })
    }
}

#[async_trait]
impl Node for LlmNode {
    async fn prep(&mut self, _ctx: &Context) -> Outcome {
        Outcome::Success {
            output: String::new(),
        }
    }

    async fn exec(&mut self, ctx: &Context) -> Outcome {
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
                    Outcome::Success {
                        output: String::new(),
                    }
                }
                Err(e) => Outcome::Failure {
                    error: e.to_string(),
                },
            },
            Err(e) => Outcome::Failure {
                error: e.to_string(),
            },
        }
    }

    async fn post(&mut self, ctx: &mut Context) -> Outcome {
        if let Some(resp) = self.response.take() {
            let content = resp["choices"][0]["message"].clone();
            ctx.push(content);
            Outcome::Success {
                output: resp.to_string(),
            }
        } else {
            Outcome::Failure {
                error: "no response".to_string(),
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

        let config = LlmConfig {
            name: "test".to_string(),
            model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "codestral-2508".to_string()),
            base_url: std::env::var("LLM_BASE_URL").unwrap_or_else(|_| {
                "https://codestral.mistral.ai/v1/chat/completions".to_string()
            }),
            api_key_env: "LLM_API_KEY".to_string(),
            max_tokens: 4096,
        };

        let mut node = LlmNode::from_config(&config).expect("failed to create LlmNode");
        let mut ctx: Context = vec![json!({"role": "user", "content": "Say hello"})];

        let outcome = node.run(&mut ctx).await;

        assert!(outcome.is_success());
        assert!(ctx.len() > 1);
        println!("Response: {:?}", ctx.last());
    }
}