use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::LLMConfig;

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

/// A single skill invocation requested by the LLM.
///
/// # Fields
/// - `skill`: The name of the skill to invoke.
/// - `input`: Optional JSON input passed to the skill. `None` when the skill needs no arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCall {
    pub skill: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
}

/// Control flow action from the LLM.
///
/// # Variants
/// - `UseSkill`: Execute one or more skills, observe results, and loop.
/// - `Execute`: Run a shell command directly via the harness.
/// - `Continue`: Pure thinking step, no skill invocation, loop again.
/// - `Completed`: Task is done, return the answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    UseSkill {
        skills: Vec<SkillCall>,
    },
    Execute {
        command: String,
    },
    Continue,
    Completed {
        answer: String,
    },
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
/// - `{"action": "use_skill", "skills": [...]}`
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
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "skill": { "type": "string" },
                                        "input": { "type": "object" }
                                    },
                                    "required": ["skill"],
                                    "additionalProperties": false
                                }
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
///
/// # Fields
/// - `client`: Reusable HTTP client.
/// - `config`: LLM provider configuration.
/// - `api_key`: Resolved API key from environment.
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
    api_key: String,
}

impl LLMClient {
    /// Creates a new LLM client from the given configuration.
    ///
    /// # Arguments
    /// - `config`: LLM provider configuration including model, base_url, and API key env var.
    ///
    /// # Returns
    /// A configured `LLMClient` or an error if the API key is not set.
    pub fn new(config: LLMConfig) -> Result<Self, String> {
        let api_key = config.api_key()?;
        Ok(Self {
            client: Client::new(),
            config,
            api_key,
        })
    }

    /// Returns the context window size for this LLM.
    pub fn context_window(&self) -> usize {
        self.config.context_window
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

    /// Sends a plain text prompt to the LLM and returns the text response.
    ///
    /// Used for auxiliary tasks like context compaction where structured output
    /// is not needed. No JSON Schema constraint is applied so the LLM can
    /// produce free-form natural language.
    ///
    /// # Arguments
    /// - `system`: System prompt for the task.
    /// - `prompt`: The user prompt text.
    /// - `max_tokens`: Maximum tokens for the response.
    ///
    /// # Returns
    /// The plain text response from the LLM.
    pub async fn summarize(
        &self,
        system: &str,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": prompt },
            ],
        });

        let (data, _) = self.send_request(&body).await?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("LLM response missing choices[0].message.content")?;

        Ok(content.to_string())
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