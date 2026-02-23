use std::fs;
use std::io::{self, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::core::config::{self, LLMConfig};
use crate::core::context::Context;
use crate::core::{Action, Node};
use crate::feature::llm::LLM;

const SYSTEM_PROMPT: &str = include_str!("../instructions/system_prompt.md");

#[derive(Parser)]
#[command(name = "minusagent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long)]
    pub llm: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
}

pub struct Session {
    pub llm: LLM,
    pub ctx: Context,
}

impl Session {
    // TODO: Move the loop logic to agent.rs
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

            let max_steps = 10;
            for step in 0..max_steps {
                let action = self.llm.run(&mut self.ctx).await?;
                if action == Action::Completed {
                    if let Some(last) = self.ctx.trajectories.last() {
                        if let Some(answer) = &last.answer {
                            println!("{}", answer);
                        }
                    }
                    break;
                }
                if step == max_steps - 1 {
                    if let Some(last) = self.ctx.trajectories.last() {
                        if let Some(thought) = &last.thought.content {
                            println!("{}", thought);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Cli {
    pub fn create_session(&self) -> Result<Session> {
        let llm_config = LLMConfig::load(self.llm.as_deref())?;
        let llm = LLM::from_config(&llm_config);
        let ctx = Context::new(SYSTEM_PROMPT.to_string());
        Ok(Session { llm, ctx })
    }
}

pub fn init_config() -> Result<()> {
    let path = config::config_path();
    if path.exists() {
        anyhow::bail!("Config already exists at {}", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let template = include_str!("../../config.toml");
    fs::write(&path, template)?;
    println!("Created config at {}", path.display());
    println!("Edit the file to add your LLM configurations.");
    Ok(())
}
