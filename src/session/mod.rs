use crate::config::Config;
use crate::core::agent::{Agent, AgentResult};
use crate::core::context::Context;
use crate::core::llm::LLMClient;
use crate::skill::registry::SkillRegistry;

/// Top-level orchestrator for a single agent interaction.
///
/// Session coordinates Context, Agent, and SkillRegistry.
/// It receives user input from the transport layer, drives the
/// agent loop, and returns the final result.
///
/// # Fields
/// - `context`: The conversation message history.
/// - `agent`: The ReAct loop agent.
pub struct Session {
    context: Context,
    agent: Agent,
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

        let llm = LLMClient::new(llm_config)?;

        let mut registry = SkillRegistry::new();
        let mut search_paths = SkillRegistry::default_search_paths();
        for p in &config.skills.paths {
            search_paths.push(p.into());
        }
        registry.discover(&search_paths)?;

        let agent = Agent::new(llm, registry, config.agent.max_steps);

        Ok(Self {
            context: Context::new(),
            agent,
        })
    }

    /// Sends a user message and runs the agent loop to produce a response.
    ///
    /// # Arguments
    /// - `input`: The user's input text.
    ///
    /// # Returns
    /// The agent's final answer, an error, or a max-steps notification.
    pub async fn send(&mut self, input: String) -> AgentResult {
        self.context.add_user_message(input);
        self.agent.run(&mut self.context).await
    }
}