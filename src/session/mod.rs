use crate::config::Config;
use crate::core::agent::Agent;
use crate::core::context::{Context, Message, Outcome};
use crate::core::harness::Harness;
use crate::core::llm::LLMClient;
use crate::core::prompt::PromptEngine;
use crate::core::{Action, Node};
use crate::skill::{SkillMeta, SkillRegistry};

/// Progress events emitted during a session turn.
///
/// Used by transports to display intermediate agent activity.
///
/// # Variants
/// - `Thinking`: Agent produced a chain-of-thought reasoning step.
/// - `Executing`: Agent chose to run a shell command.
/// - `Output`: Command produced output (content, success flag).
pub enum Event {
    Thinking(String),
    Executing(String),
    Output(String, bool),
}

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

    /// Returns the available skills for this session.
    pub fn skills(&self) -> &[SkillMeta] {
        self.context.skills()
    }

    /// Rebuilds the agent using the first LLM in config, preserving context.
    ///
    /// Call after `/switch` has moved the desired LLM to the top of the list.
    /// Context is retained so conversation history carries over. This design
    /// also allows context to be forked for multi-agent scenarios in the future.
    ///
    /// # Arguments
    /// - `config`: The configuration with the desired LLM at index 0.
    ///
    /// # Returns
    /// `Ok(())` on success, or an error string.
    pub fn extend(&mut self, config: &Config) -> Result<(), String> {
        let llm_config = config
            .llm
            .first()
            .ok_or("no LLM configured")?
            .clone();

        let prompt_engine = PromptEngine::new(String::new());
        let llm = LLMClient::new(llm_config, prompt_engine)?;
        self.agent = Agent::new(llm, config.agent.max_steps);
        Ok(())
    }

    /// Processes one user turn: drives the agent loop and dispatches
    /// `Execute` actions to the harness until the agent completes.
    ///
    /// Emits `Event`s via the callback so transports can display progress.
    ///
    /// # Arguments
    /// - `input`: The user's input text.
    /// - `on_event`: Callback invoked for each progress event.
    ///
    /// # Returns
    /// The agent's final answer string.
    pub async fn turn(&mut self, input: String, on_event: impl Fn(&Event)) -> String {
        self.context.add_user_message(input);

        loop {
            let action = self.agent.run(&mut self.context).await;

            if let Some(Message::Assistant { thought, .. }) = self.context.messages().last() {
                on_event(&Event::Thinking(thought.content.clone()));
            }

            match action {
                Action::Execute { command } => {
                    on_event(&Event::Executing(command.clone()));
                    self.harness.set_command(command.clone());
                    let result = self.harness.run(&mut self.context).await;
                    match result {
                        Action::Completed { answer } => {
                            on_event(&Event::Output(answer.clone(), false));
                            self.context.add_observation(
                                command,
                                Outcome::Failure { error: answer },
                            );
                        }
                        _ => {
                            if let Some(Message::Observation { content, .. }) =
                                self.context.messages().last()
                            {
                                on_event(&Event::Output(content.clone(), true));
                            }
                        }
                    }
                }
                Action::Completed { answer } => return answer,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, LLMConfig, SkillsConfig};

    fn test_config() -> Config {
        dotenvy::dotenv().ok();
        Config {
            agent: AgentConfig { max_steps: 5 },
            llm: vec![LLMConfig {
                name: "test".to_string(),
                model: "test-model".to_string(),
                base_url: "https://test.example.com".to_string(),
                api_key_env: "LLM_API_KEY".to_string(),
                max_tokens: 1024,
                context_window: 25_600,
            }],
            skills: SkillsConfig::default(),
        }
    }

    #[test]
    fn test_new_creates_session() {
        let config = test_config();
        let session = Session::new(&config);
        assert!(session.is_ok());
    }

    #[test]
    fn test_new_no_llm_configured() {
        dotenvy::dotenv().ok();
        let config = Config {
            agent: AgentConfig { max_steps: 5 },
            llm: vec![],
            skills: SkillsConfig::default(),
        };
        let result = Session::new(&config);
        match result {
            Err(e) => assert!(e.contains("no LLM configured")),
            Ok(_) => panic!("expected error for empty llm list"),
        }
    }

    #[test]
    fn test_new_missing_api_key_env() {
        let config = Config {
            agent: AgentConfig { max_steps: 5 },
            llm: vec![LLMConfig {
                name: "test".to_string(),
                model: "test-model".to_string(),
                base_url: "https://test.example.com".to_string(),
                api_key_env: "NONEXISTENT_KEY_99999".to_string(),
                max_tokens: 1024,
                context_window: 25_600,
            }],
            skills: SkillsConfig::default(),
        };
        let result = Session::new(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_skills_empty_by_default() {
        let config = test_config();
        let session = Session::new(&config).unwrap();
        assert!(session.skills().is_empty());
    }

    #[test]
    fn test_extend_preserves_context() {
        let config = test_config();
        let mut session = Session::new(&config).unwrap();

        // Add a user message to context
        session.context.add_user_message("hello".to_string());
        assert_eq!(session.context.len(), 1);

        // Extend with same config — context should survive
        session.extend(&config).unwrap();
        assert_eq!(session.context.len(), 1);
        if let Message::User { content } = &session.context.messages()[0] {
            assert_eq!(content, "hello");
        } else {
            panic!("expected User message");
        }
    }

    #[test]
    fn test_extend_no_llm_configured() {
        let config = test_config();
        let mut session = Session::new(&config).unwrap();

        let empty_config = Config {
            agent: AgentConfig { max_steps: 5 },
            llm: vec![],
            skills: SkillsConfig::default(),
        };
        let result = session.extend(&empty_config);
        assert!(result.is_err());
    }
}
