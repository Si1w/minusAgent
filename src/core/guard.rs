use crate::core::context::{Context, Message};
use crate::core::llm::{LLMClient, LLMResponse};
use crate::core::Outcome;

const COMPACT_THRESHOLD: f64 = 0.8;
const MAX_COMPACT_RETRIES: u32 = 3;
const SUMMARY_MAX_TOKENS: u32 = 2048;

/// Wraps `LLMClient` with context overflow protection.
///
/// Tracks token usage from API responses and applies a three-stage recovery
/// when the context approaches the window limit:
///   1. Truncate long observation content.
///   2. Compact older messages into an LLM-generated summary.
///   3. Fail if still overflowing after retries.
///
/// Also proactively compacts after a successful call when token usage exceeds
/// the threshold, preparing the context for the next iteration.
///
/// # Fields
/// - `client`: The underlying LLM client.
pub struct ContextGuard {
    client: LLMClient,
}

impl ContextGuard {
    /// Creates a new context guard wrapping the given LLM client.
    ///
    /// # Arguments
    /// - `client`: The LLM client to wrap.
    pub fn new(client: LLMClient) -> Self {
        Self { client }
    }

    /// Returns the context window size from the underlying LLM client.
    pub fn context_window(&self) -> usize {
        self.client.context_window()
    }

    /// Sends a conversation to the LLM with context overflow protection.
    ///
    /// On overflow error, retries up to 3 times: first truncating observations,
    /// then compacting history via LLM summarization. After a successful call,
    /// proactively compacts if token usage exceeds 80% of the context window.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context (may be mutated by truncation or compaction).
    ///
    /// # Returns
    /// A tuple of the parsed `LLMResponse` and the total token count.
    pub async fn chat(&self, ctx: &mut Context) -> Result<(LLMResponse, usize), String> {
        for attempt in 0..=MAX_COMPACT_RETRIES {
            let messages = ctx.to_messages();
            match self.client.chat(&messages).await {
                Ok((response, total_tokens)) => {
                    let window = self.context_window();
                    if total_tokens as f64 > window as f64 * COMPACT_THRESHOLD {
                        self.compact(ctx).await?;
                    }
                    return Ok((response, total_tokens));
                }
                Err(e) => {
                    if attempt >= MAX_COMPACT_RETRIES || !is_overflow_error(&e) {
                        return Err(e);
                    }
                    match attempt {
                        0 => self.truncate_observations(ctx),
                        _ => self.compact(ctx).await?,
                    }
                }
            }
        }

        Err("context guard: exhausted retries".to_string())
    }

    /// Truncates long observation content in the context.
    ///
    /// Walks all observation messages and cuts content exceeding 30% of the
    /// context window (in estimated characters), preserving line boundaries.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context to mutate.
    fn truncate_observations(&self, ctx: &mut Context) {
        let max_chars = (self.context_window() as f64 * 0.3 * 4.0) as usize;
        for msg in ctx.messages_mut() {
            if let Message::Observation { content, .. } = msg {
                if content.len() > max_chars {
                    let total_len = content.len();
                    let cut = content[..max_chars].rfind('\n').unwrap_or(max_chars);
                    content.truncate(cut);
                    content.push_str(&format!(
                        "\n\n[... truncated ({} chars total, showing first {}) ...]",
                        total_len, cut
                    ));
                }
            }
        }
    }

    /// Compacts older messages into an LLM-generated summary.
    ///
    /// Keeps the most recent 20% of messages (at least 4) intact and asks the
    /// LLM to summarize the older portion. The old messages are replaced with a
    /// user message containing the summary and an assistant acknowledgment.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context to compact.
    async fn compact(&self, ctx: &mut Context) -> Result<(), String> {
        let total = ctx.len();
        if total <= 4 {
            return Ok(());
        }

        let keep_count = (total / 5).max(4);
        let compress_count = (total / 2).max(2).min(total - keep_count);
        if compress_count < 2 {
            return Ok(());
        }

        let old_text = serialize_messages_for_summary(&ctx.messages()[..compress_count]);

        let prompt = format!(
            "Summarize the following conversation concisely, \
             preserving key facts and decisions. \
             Output only the summary, no preamble.\n\n{}",
            old_text
        );

        let summary = self
            .client
            .summarize(
                "You are a conversation summarizer. Be concise and factual.",
                &prompt,
                SUMMARY_MAX_TOKENS,
            )
            .await?;

        let messages = ctx.messages_mut();
        let recent: Vec<Message> = messages.drain(compress_count..).collect();
        messages.clear();
        messages.push(Message::User {
            content: format!("[Previous conversation summary]\n{}", summary),
        });
        messages.push(Message::Observation {
            skill: "compact".to_string(),
            outcome: Outcome::Success {
                output: "Context compacted successfully.".to_string(),
            },
            content: "Understood, I have the context from our previous conversation.".to_string(),
        });
        messages.extend(recent);

        Ok(())
    }
}

/// Checks if an LLM error string indicates a context overflow.
///
/// # Arguments
/// - `error`: The error message to check.
///
/// # Returns
/// `true` if the error likely indicates a token/context limit was exceeded.
fn is_overflow_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("context") || lower.contains("token")
}

/// Serializes a slice of messages into plain text for LLM summarization.
///
/// # Arguments
/// - `messages`: The messages to serialize.
///
/// # Returns
/// A newline-joined string of `[role]: content` lines.
fn serialize_messages_for_summary(messages: &[Message]) -> String {
    let mut parts = Vec::new();
    for msg in messages {
        match msg {
            Message::User { content } => {
                parts.push(format!("[user]: {}", content));
            }
            Message::Assistant { raw, .. } => {
                parts.push(format!("[assistant/thought]: {}", raw.thought.content));
                if let crate::core::llm::Action::UseSkill { skills } = &raw.action {
                    for s in skills {
                        parts.push(format!("[assistant/skill]: {}", s.skill));
                    }
                }
                if let crate::core::llm::Action::Execute { command } = &raw.action {
                    parts.push(format!("[assistant/execute]: {}", command));
                }
                if let crate::core::llm::Action::Completed { answer } = &raw.action {
                    parts.push(format!("[assistant/answer]: {}", answer));
                }
            }
            Message::Observation { skill, outcome, content } => {
                let outcome_str = match outcome {
                    Outcome::Success { .. } => "success",
                    Outcome::Failure { .. } => "failure",
                };
                let preview = if content.len() > 500 {
                    &content[..500]
                } else {
                    content.as_str()
                };
                parts.push(format!("[observation/{}/{}]: {}", skill, outcome_str, preview));
            }
        }
    }
    parts.join("\n")
}