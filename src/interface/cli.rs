use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::core::config::{self, LLMConfig};
use crate::core::context::Context;
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
