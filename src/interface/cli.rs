use std::fs;
use std::io::{self, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::core::config;
use crate::core::Action;
use crate::interface::session::Session;
use crate::interface::spinner::Spinner;

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
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Some(Commands::Init) => Self::init_config(),
            Some(Commands::New) | None => {
                let mut session = Session::new(self.llm.as_deref())?;
                Self::repl(&mut session).await
            }
        }
    }

    async fn repl(session: &mut Session) -> Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() || input == "exit" {
                break;
            }

            let spinner = Spinner::start("Thinking...");
            let result = session.run(input).await;
            spinner.stop().await;
            result?;

            if let Some(last_traj) = session.ctx.trajectories.last() {
                match &last_traj.action {
                    Action::Completed => {
                        if let Some(answer) = &last_traj.answer {
                            println!("{}", answer);
                        } else {
                            println!("Task completed");
                        }
                    }
                    _ => {}
                }
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