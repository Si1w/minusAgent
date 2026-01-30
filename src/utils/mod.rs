use anyhow::Result;
use serde_json::{Value, from_str};

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
