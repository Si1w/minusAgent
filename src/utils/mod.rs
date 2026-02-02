use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::Value;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::context::{Action, Context, Message};

pub fn parse_action(exec_res: &Option<Value>, ctx: &mut Context) -> Action {
    let content = exec_res
        .as_ref()
        .and_then(|r| r["choices"][0]["message"]["content"].as_str())
        .unwrap_or("");
    let json_str = extract_json(content);
    let parsed = serde_json::from_str(json_str).unwrap_or(Value::String(content.to_string()));
    let action = match parsed["action"].as_str() {
        Some("continue") => Action::Continue,
        Some("stop") | None => Action::Stop,
        Some(other) => Action::CallTool(other.to_string()),
    };
    ctx.push_history(Message::assistant(parsed));
    action
}

fn extract_json(content: &str) -> &str {
    if let Some(start) = content.find("```") {
        let after = &content[start + 3..];
        let json_start = after.find('\n').map(|i| i + 1).unwrap_or(0);
        let inner = &after[json_start..];
        if let Some(end) = inner.find("```") {
            return inner[..end].trim();
        }
    }
    content
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
