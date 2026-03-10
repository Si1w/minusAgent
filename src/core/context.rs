use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::llm::{LLMResponse, Thought};
use crate::skill::SkillMeta;

/// Outcome of a skill or command execution, stored in conversation history.
///
/// Used by `Message::Observation` to record whether an execution succeeded or failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success { output: String },
    Failure { error: String },
}

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

/// Manages the conversation state: skill catalog, message history, and token usage.
///
/// # Fields
/// - `skills`: Available skills loaded at initialization.
/// - `messages`: Ordered conversation messages.
/// - `total_tokens`: Cumulative token count from LLM API responses.
pub struct Context {
    skills: Vec<SkillMeta>,
    messages: Vec<Message>,
    total_tokens: usize,
}

impl Context {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            messages: Vec::new(),
            total_tokens: 0,
        }
    }

    /// Sets the available skills.
    ///
    /// # Arguments
    /// - `skills`: Skill metadata discovered by the registry.
    pub fn set_skills(&mut self, skills: Vec<SkillMeta>) {
        self.skills = skills;
    }

    /// Returns the available skills.
    pub fn skills(&self) -> &[SkillMeta] {
        &self.skills
    }

    /// Looks up a skill by name.
    ///
    /// # Arguments
    /// - `name`: The skill name to look up.
    ///
    /// # Returns
    /// The skill metadata if found.
    pub fn get_skill(&self, name: &str) -> Option<&SkillMeta> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// Updates the token count from the latest LLM API response.
    ///
    /// # Arguments
    /// - `tokens`: Total token count reported by the API.
    pub fn set_total_tokens(&mut self, tokens: usize) {
        self.total_tokens = tokens;
    }

    /// Adds to the cumulative token count.
    ///
    /// # Arguments
    /// - `tokens`: Token count to add.
    pub fn add_total_tokens(&mut self, tokens: usize) {
        self.total_tokens += tokens;
    }

    /// Returns the last known total token count.
    pub fn total_tokens(&self) -> usize {
        self.total_tokens
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
        self.messages.push(Message::Observation { skill, outcome, content });
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