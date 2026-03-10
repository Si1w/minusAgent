use crate::config::Config;
use crate::core::agent::Agent;
use crate::core::context::{Context, Outcome};
use crate::core::harness::Harness;
use crate::core::llm::LLMClient;
use crate::core::prompt::PromptEngine;
use crate::core::{Action, Node};
use crate::skill::SkillRegistry;

/// Top-level orchestrator for a multi-turn conversation.
///
/// Session owns the context, agent, and harness. Each call to `turn()`
/// processes one user message: drives the agent loop and dispatches
/// `Execute` actions to the harness until the agent completes.
/// Context persists across turns for the lifetime of the session.
///
/// # Fields
/// - `context`: The conversation message history (persists across turns).
/// - `agent`: The ReAct agent that owns the reasoning loop.
/// - `harness`: The command execution environment.
pub struct Session {
    context: Context,
    agent: Agent,
    harness: Harness,
}

impl Session {
    /// Creates a new session from the given configuration.
    ///
    /// Initializes the LLM client, discovers skills, and builds the agent.
    ///
    /// # Arguments
    /// - `config`: The application configuration.
    ///
    /// # Returns
    /// A configured `Session` or an error string.
    pub fn new(config: &Config) -> Result<Self, String> {
        let llm_config = config
            .llm
            .first()
            .ok_or("no LLM configured")?
            .clone();

        let registry = SkillRegistry::new(&config.skills.paths)?;

        let prompt_engine = PromptEngine::new(String::new());
        let llm = LLMClient::new(llm_config, prompt_engine)?;
        let agent = Agent::new(llm, config.agent.max_steps);

        let mut context = Context::new();
        context.set_skills(registry.skills());

        Ok(Self {
            context,
            agent,
            harness: Harness::new(),
        })
    }

    /// Runs the session REPL: accepts user input, drives the agent loop,
    /// and dispatches `Execute` actions to the harness until completion.
    /// IO handling will be added later.
    ///
    /// # Arguments
    /// - `input`: The user's input text.
    ///
    /// # Returns
    /// The final `Action` (typically `Completed`) for this turn.
    pub async fn run(&mut self, input: String) -> Action {
        self.context.add_user_message(input);

        loop {
            match self.agent.run(&mut self.context).await {
                Action::Execute { command } => {
                    self.harness.set_command(command.clone());
                    let result = self.harness.run(&mut self.context).await;
                    if let Action::Completed { answer } = result {
                        self.context.add_observation(
                            command,
                            Outcome::Failure { error: answer },
                        );
                    }
                }
                action => return action,
            }
        }
    }
}
