use std::io::{self, Write};

use anyhow::Result;

use crate::core::config::LLMConfig;
use crate::core::context::Context;
use crate::core::Action;
use crate::feature::agent::Agent;
use crate::feature::llm::LLM;

const SYSTEM_PROMPT: &str = include_str!("../instructions/system_prompt.md");

pub struct Session {
    pub agent: Agent,
    pub ctx: Context,
}

impl Session {
    pub fn new(llm_name: Option<&str>) -> Result<Self> {
        let llm_config = LLMConfig::load(llm_name)?;
        let llm = LLM::from_config(&llm_config);
        let agent = Agent::new(llm);
        let ctx = Context::new(SYSTEM_PROMPT.to_string());
        Ok(Session { agent, ctx })
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
            self.agent.run(&mut self.ctx).await?;

            if let Some(last_traj) = self.ctx.trajectories.last() {
                match &last_traj.action {
                    Action::Completed => {
                        if let Some(answer) = &last_traj.answer {
                            println!("{}", answer);
                        }
                        else {
                            println!("Task completed");
                        }
                    }
                    _ => {
                        println!("Failed to complete task");
                    }
                }
            }
        }
        Ok(())
    }
}