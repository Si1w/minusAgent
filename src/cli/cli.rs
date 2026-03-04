use std::fs;
use std::io::{self, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::memory::Memory;
use crate::config;
use crate::session::Session;

#[derive(Parser)]
#[command(name = "MinusAgent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long)]
    pub llm: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    New,
    List,
    Resume { session_id: String },
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Some(Commands::Init) => Self::init_config(),
            Some(Commands::List) => Self::list_sessions(),
            Some(Commands::Resume { session_id }) => {
                let memory = Memory::from_id(&session_id);
                let ctx = memory.load()?;
                let mut session = Session::new(self.llm.as_deref())?;
                session.ctx = ctx;
                println!("Resumed session: {}", session_id);
                Self::run_loop(&mut session).await
            }
            Some(Commands::New) | None => {
                let mut session = Session::new(self.llm.as_deref())?;
                Self::run_loop(&mut session).await
            }
        }
    }

    async fn run_loop(session: &mut Session) -> Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() || input == "exit" {
                break;
            }

            if let Some(handled) = Self::handle_command(input, session) {
                handled?;
                continue;
            }

            session.query(input).await?;
        }
        Ok(())
    }

    fn handle_command(input: &str, session: &Session) -> Option<Result<()>> {
        match input {
            "/save" => {
                let memory = Memory::new();
                let result = memory.save(&session.ctx);
                match &result {
                    Ok(()) => println!("Session saved: {}", memory.session_id()),
                    Err(e) => println!("Failed to save: {}", e),
                }
                Some(result)
            }
            "/list" => Some(Self::list_sessions()),
            cmd if cmd.starts_with("/load ") => {
                let session_id = cmd.strip_prefix("/load ").unwrap().trim();
                let memory = Memory::from_id(session_id);
                match memory.load() {
                    Ok(_ctx) => {
                        println!("Use 'minusagent resume {}' to load a session", session_id);
                        Some(Ok(()))
                    }
                    Err(e) => {
                        println!("Failed to load: {}", e);
                        Some(Err(e))
                    }
                }
            }
            _ if input.starts_with('/') => {
                println!("Unknown command: {}", input);
                Some(Ok(()))
            }
            _ => None,
        }
    }

    fn list_sessions() -> Result<()> {
        let sessions = Memory::list()?;
        if sessions.is_empty() {
            println!("No saved sessions");
        } else {
            for id in &sessions {
                println!("  {}", id);
            }
        }
        Ok(())
    }

    fn init_config() -> Result<()> {
        let path = config::config_path();
        if path.exists() {
            anyhow::bail!("Config already exists at {}", path.display());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let template = include_str!("../../config.json");
        fs::write(&path, template)?;
        println!("Created config at {}", path.display());
        println!("Edit the file to add your LLM configurations.");
        Ok(())
    }
}