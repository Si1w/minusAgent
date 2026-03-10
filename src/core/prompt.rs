use serde_json::Value;

use crate::core::context::{Context, Outcome};
use crate::skill::{load_body, SkillMeta};

const BASE_PROMPT: &str = "\
You are a ReAct agent. Think step-by-step, then choose an action.

## Response Format
Respond in JSON with `thought` and `action` fields.

## Actions
- `use_skill`: Load skills by name for instructions. `{\"action\": \"use_skill\", \"skills\": [\"name\"]}`
- `execute`: Run a shell command. `{\"action\": \"execute\", \"command\": \"...\"}`
- `continue`: Keep thinking without acting.
- `completed`: Return the final answer. `{\"action\": \"completed\", \"answer\": \"...\"}`";

/// Builds LLM API messages from a `Context` and loads skill instructions.
///
/// Owns the system prompt and combines it with the skill catalog and
/// conversation history from the context to produce the full message list.
/// Also handles loading skill body content into context as observations.
///
/// # Fields
/// - `system_prompt`: User-provided system instructions appended to the base prompt.
pub struct PromptEngine {
    system_prompt: String,
}

impl PromptEngine {
    /// Creates a new prompt engine with the given system instructions.
    ///
    /// # Arguments
    /// - `system_prompt`: Additional system instructions beyond the base prompt.
    pub fn new(system_prompt: String) -> Self {
        Self { system_prompt }
    }

    /// Builds the complete message list for the LLM API from context.
    ///
    /// Assembles the system message (base prompt + user instructions + skill
    /// catalog) followed by the conversation messages.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context to read from.
    ///
    /// # Returns
    /// A vector of JSON values ready for the LLM chat API.
    pub fn build(&self, ctx: &Context) -> Vec<Value> {
        let system = self.build_system_prompt(ctx.skills());

        let mut out = Vec::with_capacity(ctx.len() + 1);
        out.push(serde_json::json!({
            "role": "system",
            "content": system,
        }));
        out.extend(ctx.messages().iter().map(|m| m.to_json()));
        out
    }

    /// Loads a skill's instruction body into the context.
    ///
    /// Looks up the skill in the context, reads its SKILL.md body,
    /// and injects it as an observation message so the LLM can use
    /// the instructions in the next iteration.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context to look up the skill and inject the instruction into.
    /// - `skill_name`: The name of the skill to load.
    pub fn load_skill(&self, ctx: &mut Context, skill_name: &str) {
        let outcome = match ctx.get_skill(skill_name) {
            Some(meta) => {
                let skill_md = meta.path.join("SKILL.md");
                match load_body(&skill_md) {
                    Ok(body) => Outcome::Success { output: body },
                    Err(e) => Outcome::Failure { error: e },
                }
            }
            None => Outcome::Failure {
                error: format!("skill '{}' not found", skill_name),
            },
        };
        ctx.add_observation(skill_name.to_string(), outcome);
    }

    /// Assembles the full system prompt string.
    ///
    /// # Arguments
    /// - `skills`: Available skill metadata from context.
    fn build_system_prompt(&self, skills: &[SkillMeta]) -> String {
        let mut prompt = BASE_PROMPT.to_string();

        if !self.system_prompt.is_empty() {
            prompt.push_str("\n\n## User Instructions\n");
            prompt.push_str(&self.system_prompt);
        }

        if !skills.is_empty() {
            prompt.push_str("\n\n## Available Skills\n");
            for meta in skills {
                prompt.push_str(&format!("- `{}`: {}\n", meta.name, meta.description));
            }
        }

        prompt
    }
}
