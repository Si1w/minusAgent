use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use reqwest::Response;
use serde_json::Value;

fn key_pressed() -> bool {
    if event::poll(Duration::ZERO).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            return key.kind == KeyEventKind::Press;
        }
    }
    false
}

/// Process SSE stream, calling on_chunk for each content piece.
/// Returns (content, interrupted) - interrupted is true if user pressed a key to stop.
pub async fn process_sse_stream(
    mut resp: Response,
    on_chunk: impl Fn(&str),
) -> Result<(String, bool)> {
    let mut buffer = String::new();
    let mut full_content = String::new();
    let mut interrupted = false;

    while let Some(chunk) = resp.chunk().await? {
        if key_pressed() {
            interrupted = true;
            break;
        }

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim();
            if let Some(data) = line.strip_prefix("data: ") {
                if data != "[DONE]" {
                    if let Some(content) = serde_json::from_str::<Value>(data)
                        .ok()
                        .and_then(|v| v["choices"][0]["delta"]["content"].as_str().map(String::from))
                    {
                        on_chunk(&content);
                        full_content.push_str(&content);
                    }
                }
            }
            buffer = buffer[pos + 1..].to_string();
        }
    }
    Ok((full_content, interrupted))
}
