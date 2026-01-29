use std::env;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;
use serde_yaml::from_str as from_yaml;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::context::{ChatContext, Context, Message};
use crate::core::Node;
use crate::cot::{ChainOfThought, Execute, Plan};
use crate::llm::Llm;

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
    },
    Interactive,
}

fn create_llm() -> Result<Llm> {
    dotenvy::dotenv().ok();

    let base_url = env::var("LLM_BASE_URL")
        .unwrap_or_else(|_| "https://codestral.mistral.ai/v1/chat/completions".to_string());
    let model = env::var("LLM_MODEL")
        .unwrap_or_else(|_| "codestral-2508".to_string());
    let api_key = env::var("LLM_API_KEY")
        .map_err(|_| anyhow::anyhow!("LLM_API_KEY environment variable required"))?;

    Ok(Llm::new(&base_url, &model, &api_key))
}

fn start_thinking() -> (Arc<AtomicBool>, JoinHandle<()>) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let handle = tokio::spawn(async move {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        while r.load(Ordering::Relaxed) {
            print!("\r{} Thinking...", frames[i % frames.len()]);
            io::stdout().flush().ok();
            sleep(Duration::from_millis(80)).await;
            i += 1;
        }
        print!("\r              \r");
        io::stdout().flush().ok();
    });
    (running, handle)
}

async fn stop_thinking(running: Arc<AtomicBool>, handle: JoinHandle<()>) {
    running.store(false, Ordering::Relaxed);
    let _ = handle.await;
}

fn parse_final(content: &str) -> String {
    let yaml_str = if let Some(start) = content.find("```yaml") {
        let start = start + 7;
        let end = content[start..].find("```").map(|i| start + i).unwrap_or(content.len());
        &content[start..end]
    } else {
        content
    };
    from_yaml::<Value>(yaml_str.trim())
        .ok()
        .and_then(|v| v["final"].as_str().map(String::from))
        .unwrap_or_else(|| content.to_string())
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prompt { text } => prompt(&text).await,
        Commands::Cot { text } => cot(&text).await,
        Commands::Interactive => interactive().await,
    }
}

async fn prompt(text: &str) -> Result<()> {
    let mut llm = create_llm()?;
    let mut ctx = ChatContext::new();
    ctx.push(Message::user(text));

    let (running, handle) = start_thinking();
    llm.run(&mut ctx).await?;
    stop_thinking(running, handle).await;

    if let Some(content) = ctx.last_content() {
        println!("{}", content);
    }

    Ok(())
}

async fn cot(text: &str) -> Result<()> {
    let llm1 = create_llm()?;
    let llm2 = create_llm()?;
    let plan = Plan::new(llm1);
    let execute = Execute::new(llm2);
    let mut cot = ChainOfThought::new(plan, execute);

    let mut ctx = ChatContext::new();
    ctx.push(Message::user(text));

    let (running, handle) = start_thinking();
    cot.run(&mut ctx).await?;
    stop_thinking(running, handle).await;

    if let Some(content) = ctx.last_content() {
        println!("{}", parse_final(content));
    }

    Ok(())
}

async fn interactive() -> Result<()> {
    let mut llm = create_llm()?;
    let mut ctx = ChatContext::new();

    println!("Interactive mode. Type 'exit' to quit.\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" || input == "quit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        ctx.push(Message::user(input));

        let (running, handle) = start_thinking();
        llm.run(&mut ctx).await?;
        stop_thinking(running, handle).await;

        if let Some(content) = ctx.last_content() {
            println!("\n{}\n", content);
        }
    }

    Ok(())
}
