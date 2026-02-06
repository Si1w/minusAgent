use std::env;

use anyhow::Result;
use clap::{Parser, Subcommand};

use super::utils::{start_thinking, stop_thinking};
use crate::core::Context;
use crate::feature::cot::ChainOfThought;
use crate::feature::llm::{Llm, StreamCallback};
use crate::interface::interactive::Interactive;

#[derive(Parser)]
#[command(name = "minusagent")]
#[command(about = "A minimal LLM agent CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Prompt {
        text: String,
    },
    Cot {
        text: String,
        #[arg(long)]
        max_turns: Option<usize>,
    },
    Interactive {
        #[arg(long)]
        cot: bool,
    },
}

fn create_llm() -> Result<Llm> {
    dotenvy::dotenv().ok();

    let base_url = env::var("LLM_BASE_URL")
        .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
    let model = env::var("LLM_MODEL")
        .unwrap_or_else(|_| "codestral-latest".to_string());
    let api_key = env::var("LLM_API_KEY")
        .map_err(|_| anyhow::anyhow!("LLM_API_KEY environment variable required"))?;

    Ok(Llm::new(&base_url, &model, &api_key))
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prompt { text } => prompt(&text).await,
        Commands::Cot { text, max_turns } => cot(&text, max_turns).await,
        Commands::Interactive { cot } => interactive(cot).await,
    }
}

async fn prompt(text: &str) -> Result<()> {
    let llm = create_llm()?;
    let messages = serde_json::json!([{"role": "user", "content": text}]);

    let callback: StreamCallback = Box::new(|chunk| {
        print!("{}", chunk);
        use std::io::Write;
        std::io::stdout().flush().ok();
    });

    llm.exec_stream(Some(messages), callback).await?;
    println!();

    Ok(())
}

async fn cot(text: &str, max_turns: Option<usize>) -> Result<()> {
    let llm = create_llm()?;
    let mut cot = ChainOfThought::new(llm);
    if let Some(n) = max_turns {
        cot = cot.with_max_turns(n);
    }

    let mut ctx = Context::new();
    ctx.set_user_message(text);

    let (running, handle) = start_thinking();
    cot.run(&mut ctx).await?;
    stop_thinking(running, handle).await;

    let last = ctx.last_content();
    let output = last
        .and_then(|v| v["answer"].as_str())
        .unwrap_or_default();

    if output.is_empty() {
        eprintln!("No answer received. Last response: {:?}", last);
    } else {
        println!("{}", output);
    }

    Ok(())
}

async fn interactive(cot: bool) -> Result<()> {
    let llm = create_llm()?;
    let mut ctx = Context::new();
    let mut chat = Interactive::new(llm, cot);
    chat.run(&mut ctx).await
}
