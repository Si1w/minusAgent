use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::LLMConfig;
use crate::core::{Action, Node};
use crate::core::context::Context;
use crate::core::prompt::PromptEngine;

/// Categories of reasoning in the agent's chain-of-thought.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThoughtType {
    Planning,
    Analysis,
    DecisionMaking,
    ProblemSolving,
    MemoryIntegration,
    SelfReflection,
    GoalSetting,
    Prioritization,
}

/// The thought component of an LLM response, capturing chain-of-thought reasoning.
///
/// # Fields
/// - `thought_type`: Category of reasoning.
/// - `content`: The actual reasoning text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub thought_type: ThoughtType,
    pub content: String,
}

/// Structured response from the LLM.
///
/// # Fields
/// - `thought`: Chain-of-thought reasoning.
/// - `action`: Control flow decision (use skill, continue thinking, or complete).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub thought: Thought,
    pub action: Action,
}

/// Hand-written JSON Schema for `LLMResponse`, used in structured output requests.
///
/// The `action` field uses a discriminated union via the `action` tag:
/// - `{"action": "use_skill", "skills": ["skill_name", ...]}`
/// - `{"action": "continue"}`
/// - `{"action": "completed", "answer": "..."}`
fn response_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "thought": {
                "type": "object",
                "properties": {
                    "thought_type": {
                        "type": "string",
                        "enum": [
                            "planning",
                            "analysis",
                            "decision_making",
                            "problem_solving",
                            "memory_integration",
                            "self_reflection",
                            "goal_setting",
                            "prioritization"
                        ]
                    },
                    "content": { "type": "string" }
                },
                "required": ["thought_type", "content"],
                "additionalProperties": false
            },
            "action": {
                "oneOf": [
                    {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string", "const": "use_skill" },
                            "skills": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["action", "skills"],
                        "additionalProperties": false
                    },
                    {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string", "const": "execute" },
                            "command": { "type": "string" }
                        },
                        "required": ["action", "command"],
                        "additionalProperties": false
                    },
                    {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string", "const": "continue" }
                        },
                        "required": ["action"],
                        "additionalProperties": false
                    },
                    {
                        "type": "object",
                        "properties": {
                            "action": { "type": "string", "const": "completed" },
                            "answer": { "type": "string" }
                        },
                        "required": ["action", "answer"],
                        "additionalProperties": false
                    }
                ]
            }
        },
        "required": ["thought", "action"],
        "additionalProperties": false
    })
}

/// HTTP client for OpenAI-compatible LLM APIs.
///
/// Sends conversation messages and parses structured JSON responses.
/// Implements `Node` so it can be driven through the prep→exec→post pipeline:
///   - **prep**: validates that the context has messages.
///   - **exec**: calls the LLM API, passes the response as `Value` in Outcome.
///   - **post**: deserializes the response and writes it into context.
///
/// # Fields
/// - `client`: Reusable HTTP client.
/// - `config`: LLM provider configuration.
/// - `api_key`: Resolved API key from environment.
/// - `prompt_engine`: Builds messages from context for API calls.
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
    api_key: String,
    prompt_engine: PromptEngine,
}

impl LLMClient {
    /// Creates a new LLM client from the given configuration.
    ///
    /// # Arguments
    /// - `config`: LLM provider configuration including model, base_url, and API key env var.
    /// - `prompt_engine`: The prompt engine for building messages.
    ///
    /// # Returns
    /// A configured `LLMClient` or an error if the API key is not set.
    pub fn new(config: LLMConfig, prompt_engine: PromptEngine) -> Result<Self, String> {
        let api_key = config.api_key()?;
        Ok(Self {
            client: Client::new(),
            config,
            api_key,
            prompt_engine,
        })
    }

    /// Returns the context window size for this LLM.
    pub fn context_window(&self) -> usize {
        self.config.context_window
    }

    /// Returns a reference to the prompt engine.
    pub fn prompt_engine(&self) -> &PromptEngine {
        &self.prompt_engine
    }

    /// Sends a conversation to the LLM and returns the structured response with token usage.
    ///
    /// # Arguments
    /// - `messages`: The conversation history as JSON values following OpenAI message format.
    ///
    /// # Returns
    /// A tuple of the parsed `LLMResponse` and the total token count from the API.
    pub async fn chat(&self, messages: &[Value]) -> Result<(LLMResponse, usize), String> {
        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": messages,
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "agent_response",
                    "strict": true,
                    "schema": response_schema(),
                }
            },
        });

        let (data, total_tokens) = self.send_request(&body).await?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("LLM response missing choices[0].message.content")?;

        let response: LLMResponse = serde_json::from_str(content)
            .map_err(|e| format!("failed to parse LLM JSON output: {}\nraw: {}", e, content))?;

        Ok((response, total_tokens))
    }

    /// Sends a request body to the LLM API and returns the parsed JSON response with token count.
    ///
    /// # Arguments
    /// - `body`: The JSON request body.
    ///
    /// # Returns
    /// A tuple of the response JSON value and the total token count.
    async fn send_request(&self, body: &Value) -> Result<(Value, usize), String> {
        let resp = self
            .client
            .post(&self.config.base_url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LLM returned {}: {}", status, text));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse LLM response: {}", e))?;

        let total_tokens = data["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;

        Ok((data, total_tokens))
    }
}

#[async_trait]
impl Node for LLMClient {
    /// Builds the full message list from context via the prompt engine.
    ///
    /// # Arguments
    /// - `shared`: The conversation context.
    ///
    /// # Returns
    /// A JSON array of messages ready for the LLM API.
    async fn prep(&mut self, shared: &Context) -> Result<Value, String> {
        let messages = self.prompt_engine.build(shared);
        if messages.len() <= 1 {
            return Err("context has no messages".to_string());
        }
        Ok(Value::Array(messages))
    }

    /// Calls the LLM chat API with the prepared messages.
    ///
    /// # Arguments
    /// - `prep_res`: JSON array of messages from `prep`.
    ///
    /// # Returns
    /// A JSON object with `response` and `total_tokens`.
    async fn exec(&mut self, prep_res: Value) -> Result<Value, String> {
        let messages = prep_res.as_array().ok_or("prep_res is not an array")?;
        let (response, total_tokens) = self.chat(messages).await?;
        let response_value = serde_json::to_value(&response)
            .map_err(|e| format!("failed to serialize LLMResponse: {}", e))?;
        Ok(serde_json::json!({
            "response": response_value,
            "total_tokens": total_tokens,
        }))
    }

    /// Writes the LLM response into context and returns the action for flow control.
    ///
    /// # Arguments
    /// - `shared`: The conversation context to update.
    /// - `_prep_res`: Unused.
    /// - `exec_res`: JSON object containing `response` and `total_tokens`.
    ///
    /// # Returns
    /// The `Action` from the LLM response.
    async fn post(&mut self, shared: &mut Context, _prep_res: Value, exec_res: Value) -> Action {
        let response: LLMResponse = match serde_json::from_value(exec_res["response"].clone()) {
            Ok(r) => r,
            Err(e) => return Action::Completed {
                answer: format!("failed to parse exec result: {}", e),
            },
        };
        let total_tokens = exec_res["total_tokens"].as_u64().unwrap_or(0) as usize;

        shared.add_total_tokens(total_tokens);
        shared.add_assistant_message(response.clone());

        response.action
    }
}