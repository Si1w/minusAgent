use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::Outcome;
use crate::core::llm::{LLMResponse, Thought};

/// A single message in the conversation history.
///
/// # Variants
/// - `User`: Input from the user.
/// - `Assistant`: LLM response with thought and optional actions/answer.
/// - `Observation`: Result of a skill execution fed back to the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    User { content: String },
    Assistant { thought: Thought, raw: LLMResponse },
    Observation { skill: String, outcome: Outcome, content: String },
}

impl Message {
    /// Converts this message into JSON format for the LLM API.
    ///
    /// # Returns
    /// A JSON value representing the message.
    pub fn to_json(&self) -> Value {
        match self {
            Message::User { content } => serde_json::json!({
                "role": "user",
                "content": content,
            }),
            Message::Assistant { raw, .. } => serde_json::json!({
                "role": "assistant",
                "content": serde_json::to_string(raw).unwrap_or_default(),
            }),
            Message::Observation { skill, outcome, content } => {
                let outcome_str = match outcome {
                    Outcome::Success { .. } => "success",
                    Outcome::Failure { .. } => "failure",
                };
                serde_json::json!({
                    "role": "user",
                    "content": serde_json::json!({
                        "role": "observation",
                        "skill": skill,
                        "outcome": outcome_str,
                        "content": content,
                    }).to_string(),
                })
            }
        }
    }
}

/// Manages the ordered conversation message history.
///
/// Provides methods to append messages and export them
/// in JSON format for LLM consumption.
///
/// # Fields
/// - `messages`: Ordered list of conversation messages.
pub struct Context {
    messages: Vec<Message>,
}

impl Context {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Appends a user message to the conversation history.
    ///
    /// # Arguments
    /// - `content`: The user's input text.
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message::User { content });
    }

    /// Appends an assistant message from an LLM response.
    ///
    /// # Arguments
    /// - `response`: The structured LLM response.
    pub fn add_assistant_message(&mut self, response: LLMResponse) {
        self.messages.push(Message::Assistant {
            thought: response.thought.clone(),
            raw: response,
        });
    }

    /// Appends an observation message from a skill execution result.
    ///
    /// # Arguments
    /// - `skill`: The name of the executed skill.
    /// - `outcome`: The execution outcome (Success or Failure).
    pub fn add_observation(&mut self, skill: String, outcome: Outcome) {
        let content = match &outcome {
            Outcome::Success { output } => output.clone(),
            Outcome::Failure { error } => error.clone(),
        };
        self.messages.push(Message::Observation {
            skill,
            outcome,
            content,
        });
    }

    /// Exports all messages as JSON for the LLM API.
    ///
    /// # Returns
    /// A vector of JSON values representing the conversation history.
    pub fn to_messages(&self) -> Vec<Value> {
        self.messages.iter().map(|m| m.to_json()).collect()
    }

    /// Returns the number of messages in the conversation.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Returns a reference to the message list.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Returns a mutable reference to the message list.
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }
}