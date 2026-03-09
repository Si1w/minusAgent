use crate::core::context::Context;
use crate::core::harness::Harness;
use crate::core::Node;
use crate::core::llm::{Action, LLMClient};
use crate::skill::registry::SkillRegistry;

/// Result of an agent run.
///
/// # Variants
/// - `Answer`: The agent produced a final answer.
/// - `MaxSteps`: The agent hit the step limit without completing.
/// - `Error`: An unrecoverable error occurred.
#[derive(Debug)]
pub enum AgentResult {
    Answer(String),
    MaxSteps,
    Error(String),
}

/// ReAct loop agent that coordinates LLM calls and skill execution.
///
/// The agent iterates: LLM → parse actions → execute via harness → observe → loop,
/// until the LLM returns an answer or max_steps is reached.
///
/// # Fields
/// - `llm`: The LLM client for chat completions.
/// - `harness`: The execution environment for running commands.
/// - `registry`: The skill registry for resolving skill actions.
/// - `max_steps`: Maximum number of ReAct iterations.
pub struct Agent {
    llm: LLMClient,
    harness: Harness,
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
            harness: Harness::new(),
            registry,
            max_steps,
        }
    }

    /// Runs the ReAct loop on the given context.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context with message history.
    ///
    /// # Returns
    /// An `AgentResult` indicating the outcome of the run.
    pub async fn run(&mut self, ctx: &mut Context) -> AgentResult {
        for _ in 0..self.max_steps {
            let messages = ctx.to_messages();
            let response = match self.llm.chat(&messages).await {
                Ok(r) => r,
                Err(e) => return AgentResult::Error(e),
            };

            ctx.add_assistant_message(response.clone());

            if response.actions.is_empty() {
                return AgentResult::Answer(
                    response.answer.unwrap_or_default(),
                );
            }

            for action in &response.actions {
                let command = self.resolve_command(action);
                self.harness.set_command(command);
                let outcome = self.harness.run(ctx).await;
                ctx.add_observation(action.skill.clone(), outcome);
            }
        }

        AgentResult::MaxSteps
    }

    /// Resolves an LLM action into a shell command string.
    ///
    /// If the action refers to a registered skill, constructs the command
    /// from the skill's scripts/run.sh. Otherwise treats the input as
    /// a direct shell command.
    ///
    /// # Arguments
    /// - `action`: The action from the LLM response.
    ///
    /// # Returns
    /// A shell command string ready for harness execution.
    fn resolve_command(&self, action: &Action) -> String {
        if let Some(meta) = self.registry.get(&action.skill) {
            let script = meta.path.join("scripts").join("run.sh");
            if script.exists() {
                let input_str = serde_json::to_string(&action.input).unwrap_or_default();
                return format!(
                    "SKILL_INPUT='{}' sh {}",
                    input_str.replace('\'', "'\\''"),
                    script.display()
                );
            }
        }

        action.input.as_str().unwrap_or("echo 'no command'").to_string()
    }
}
