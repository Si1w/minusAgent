use crate::core::context::Context;
use crate::core::llm::{Action, LLMClient};
use crate::core::Outcome;
use crate::skill::loader::load_body;
use crate::skill::registry::SkillRegistry;

/// Result of an agent run, returned to Session for dispatch.
///
/// # Variants
/// - `Answer`: The agent produced a final answer.
/// - `Execute`: The agent requests a shell command execution (handled by Session via harness).
/// - `MaxSteps`: The agent hit the step limit without completing.
/// - `Error`: An unrecoverable error occurred.
#[derive(Debug)]
pub enum AgentResult {
    Answer(String),
    Execute { command: String },
    MaxSteps,
    Error(String),
}

/// ReAct loop agent that coordinates LLM calls and skill loading.
///
/// The agent handles `UseSkill` (load instructions into context) and `Continue`
/// (pure thinking) internally. It returns to Session on `Execute` (for harness
/// dispatch) or `Completed` (final answer).
///
/// # Fields
/// - `llm`: The LLM client for chat completions.
/// - `registry`: The skill registry for resolving skill instructions.
/// - `max_steps`: Maximum number of ReAct iterations.
pub struct Agent {
    llm: LLMClient,
    registry: SkillRegistry,
    max_steps: u32,
}

impl Agent {
    /// Creates a new agent.
    ///
    /// # Arguments
    /// - `llm`: The LLM client.
    /// - `registry`: The skill registry.
    /// - `max_steps`: Maximum number of ReAct iterations.
    pub fn new(llm: LLMClient, registry: SkillRegistry, max_steps: u32) -> Self {
        Self {
            llm,
            registry,
            max_steps,
        }
    }

    /// Runs the ReAct loop on the given context.
    ///
    /// Handles `UseSkill` and `Continue` internally. Returns to Session
    /// on `Execute` or `Completed` for external dispatch.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context with message history.
    ///
    /// # Returns
    /// An `AgentResult` indicating the outcome or next action for Session.
    pub async fn run(&mut self, ctx: &mut Context) -> AgentResult {
        for _ in 0..self.max_steps {
            let messages = ctx.to_messages();
            let (response, _total_tokens) = match self.llm.chat(&messages).await {
                Ok(r) => r,
                Err(e) => return AgentResult::Error(e),
            };

            ctx.add_assistant_message(response.clone());

            match &response.action {
                Action::Completed { answer } => {
                    return AgentResult::Answer(answer.clone());
                }
                Action::Execute { command } => {
                    return AgentResult::Execute {
                        command: command.clone(),
                    };
                }
                Action::Continue => {
                    continue;
                }
                Action::UseSkill { skills } => {
                    for skill_call in skills {
                        self.load_skill_instruction(ctx, &skill_call.skill);
                    }
                }
            }
        }

        AgentResult::MaxSteps
    }

    /// Loads a skill's instruction body into the context.
    ///
    /// Looks up the skill in the registry, reads its SKILL.md body,
    /// and injects it as an observation message so the LLM can use
    /// the instructions in the next iteration.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context to inject the instruction into.
    /// - `skill_name`: The name of the skill to load.
    fn load_skill_instruction(&self, ctx: &mut Context, skill_name: &str) {
        let outcome = match self.registry.get(skill_name) {
            Some(meta) => {
                let skill_md = meta.path.join("SKILL.md");
                match load_body(&skill_md) {
                    Ok(body) => Outcome::Success { output: body },
                    Err(e) => Outcome::Failure { error: e },
                }
            }
            None => Outcome::Failure {
                error: format!("skill '{}' not found in registry", skill_name),
            },
        };
        ctx.add_observation(skill_name.to_string(), outcome);
    }
}