use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, from_str};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub fn parse_content(content: &str) -> Result<Value> {
    let json_str = if let Some(start) = content.find("```json") {
        let start = start + 7;
        let end = content[start..].find("```").map(|i| start + i).unwrap_or(content.len());
        &content[start..end]
    } else {
        content
    };

    let value: Value = from_str(json_str.trim())?;
    Ok(value)
}

pub fn start_thinking() -> (Arc<AtomicBool>, JoinHandle<()>) {
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

pub async fn stop_thinking(running: Arc<AtomicBool>, handle: JoinHandle<()>) {
    running.store(false, Ordering::Relaxed);
    let _ = handle.await;
}
