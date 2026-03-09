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

/// A single action requested by the LLM.
///
/// # Fields
/// - `skill`: The name of the skill to invoke.
/// - `input`: Arbitrary JSON input passed to the skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub skill: String,
    #[serde(default)]
    pub input: Value,
}

/// Structured response from the LLM.
///
/// When `actions` is non-empty, the agent executes skills and loops.
/// When `actions` is empty, `answer` contains the final response.
///
/// # Fields
/// - `thought`: Chain-of-thought reasoning.
/// - `actions`: List of skill invocations (empty means task complete).
/// - `answer`: Final answer, only present when actions is empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub thought: Thought,
    #[serde(default)]
    pub actions: Vec<Action>,
    pub answer: Option<String>,
}

/// Hand-written JSON Schema for `LLMResponse`, used in structured output requests.
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
            "actions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "skill": { "type": "string" },
                        "input": { "type": "object" }
                    },
                    "required": ["skill", "input"],
                    "additionalProperties": false
                }
            },
            "answer": { "type": "string" }
        },
        "required": ["thought", "actions"],
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

    /// Sends a conversation to the LLM and returns the structured response.
    ///
    /// # Arguments
    /// - `messages`: The conversation history as JSON values following OpenAI message format.
    ///
    /// # Returns
    /// A parsed `LLMResponse` containing thought, actions, and optional answer.
    pub async fn chat(&self, messages: &[Value]) -> Result<LLMResponse, String> {
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

        let resp = self
            .client
            .post(&self.config.base_url)
            .bearer_auth(&self.api_key)
            .json(&body)
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

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("LLM response missing choices[0].message.content")?;

        serde_json::from_str(content)
            .map_err(|e| format!("failed to parse LLM JSON output: {}\nraw: {}", e, content))
    }
}