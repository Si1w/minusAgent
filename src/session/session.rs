use std::io::{self, Write};

use anyhow::Result;

use crate::agent::Agent;
use crate::agent::llm::LLM;
use crate::core::Context;
use crate::core::{Action, Node};
use crate::session::config::Config;
use crate::session::harness::Harness;

const SYSTEM_PROMPT: &str = include_str!("../prompt/system_prompt.md");

pub struct Session {
    pub agent: Agent,
    pub ctx: Context,
    harness: Harness,
}

impl Session {
    pub fn new(llm_name: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let llm_config = config.get_llm(llm_name)?;
        let llm = LLM::from_config(&llm_config);
        let agent = Agent::new(llm, config.agent.max_iterations());
        let ctx = Context::new(SYSTEM_PROMPT.to_string());
        Ok(Session { agent, ctx, harness: Harness })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() || input == "exit" {
                break;
            }

            self.ctx.init_trajectory(input.to_string());

            loop {
                self.agent.run(&mut self.ctx).await?;

                match self.ctx.trajectories.last().map(|t| &t.action) {
                    Some(Action::Completed) => {
                        if let Some(answer) = self.ctx.trajectories.last().and_then(|t| t.answer.as_ref()) {
                            println!("{}", answer);
                        } else {
                            println!("Task completed");
                        }
                        break;
                    }
                    Some(Action::Execute(_)) => {
                        self.harness.run(&mut self.ctx).await?;
                    }
                    _ => {
                        println!("Failed to complete task");
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
