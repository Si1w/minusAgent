use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::core::config;
use crate::interface::session::Session;

#[derive(Parser)]
#[command(name = "MinusAgent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long)]
    pub llm: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    New,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Init => Self::init_config(),
            Commands::New => {
                let mut session = self.create_session()?;
                session.run().await
            }
        }
    }

    pub fn create_session(&self) -> Result<Session> {
        Session::new(self.llm.as_deref())
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
}