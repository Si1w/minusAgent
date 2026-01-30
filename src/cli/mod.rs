use std::env;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::context::{Context, Message};
use crate::core::Node;
use crate::cot::{ChainOfThought, PlanNode, ThinkingNode};
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
        #[arg(long)]
        max_turns: Option<usize>,
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

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prompt { text } => prompt(&text).await,
        Commands::Cot { text, max_turns } => cot(&text, max_turns).await,
        Commands::Interactive => interactive().await,
    }
}

async fn prompt(text: &str) -> Result<()> {
    let mut llm = create_llm()?;
    let mut ctx = Context::new();
    ctx.push_history(Message::user(text));

    let (running, handle) = start_thinking();
    llm.run(&mut ctx).await?;
    stop_thinking(running, handle).await;

    if let Some(content) = ctx.last_content() {
        println!("{}", content);
    }

    Ok(())
}

async fn cot(text: &str, max_turns: Option<usize>) -> Result<()> {
    let llm = create_llm()?;
    let plan_node = PlanNode::new(llm.clone());
    let thinking_node = ThinkingNode::new(llm);
    let mut cot = ChainOfThought::new(plan_node, thinking_node);
    if let Some(n) = max_turns {
        cot = cot.with_max_turns(n);
    }

    let mut ctx = Context::new();
    ctx.set_user_message(text);

    let (running, handle) = start_thinking();
    cot.run(&mut ctx).await?;
    stop_thinking(running, handle).await;

    let output = ctx.last_content()
        .and_then(|c| serde_json::from_str::<serde_json::Value>(c).ok())
        .and_then(|v| v["answer"].as_str().map(String::from))
        .unwrap_or_default();
    println!("{}", &output);

    Ok(())
}

async fn interactive() -> Result<()> {
    let mut llm = create_llm()?;
    let mut ctx = Context::new();

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

        ctx.push_history(Message::user(input));

        let (running, handle) = start_thinking();
        llm.run(&mut ctx).await?;
        stop_thinking(running, handle).await;

        if let Some(content) = ctx.last_content() {
            println!("\n{}\n", content);
        }
    }

    Ok(())
}
