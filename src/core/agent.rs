use crate::core::context::Context;
use crate::core::llm::LLMClient;
use crate::core::{Action, Node};

/// ReAct agent that owns the reasoning loop.
///
/// Drives the LLM one step at a time, handling `UseSkill` and `Continue`
/// internally. Returns `Execute` or `Completed` to Session for dispatch.
///
/// # Fields
/// - `llm`: The LLM client for chat completions.
/// - `max_steps`: Maximum total steps (LLM calls) per run.
pub struct Agent {
    llm: LLMClient,
    max_steps: u32,
}

impl Agent {
    /// Creates a new agent.
    ///
    /// # Arguments
    /// - `llm`: The LLM client.
    /// - `max_steps`: Maximum steps per run.
    pub fn new(llm: LLMClient, max_steps: u32) -> Self {
        Self { llm, max_steps }
    }

    /// Runs the agent loop until it needs external dispatch or completes.
    ///
    /// Handles `UseSkill` (loads instructions via prompt engine) and `Continue`
    /// internally. Returns `Execute` to Session for harness dispatch, or
    /// `Completed` when done. Also returns `Completed` on max steps.
    ///
    /// # Arguments
    /// - `ctx`: The conversation context with message history.
    ///
    /// # Returns
    /// `Execute` for Session to dispatch, or `Completed` when finished.
    pub async fn run(&mut self, ctx: &mut Context) -> Action {
        for _ in 0..self.max_steps {
            let action = self.llm.run(ctx).await;

            match action {
                Action::UseSkill { skills } => {
                    for skill in &skills {
                        self.llm.prompt_engine().load_skill(ctx, skill);
                    }
                }
                Action::Continue => {
                    ctx.add_user_message("continue".to_string());
                }
                action => return action,
            }
        }

        Action::Completed {
            answer: "max steps reached".to_string(),
        }
    }
}
