use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

const DEFAULT_MAX_TOKENS: usize = 4096;

#[derive(Deserialize)]
struct FileConfig {
    default: LLMConfig,
    llm: Option<Vec<LLMConfig>>,
}

#[derive(Deserialize, Clone)]
pub struct LLMConfig {
    pub name: String,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: Option<usize>,
}

impl LLMConfig {
    pub fn max_tokens(&self) -> usize {
        self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS)
    }

    pub fn load(name: Option<&str>) -> Result<Self> {
        dotenvy::dotenv().ok();
        let path = config_path();
        let content = fs::read_to_string(&path)?;
        let file_config: FileConfig = toml::from_str(&content)?;

        match name {
            Some(n) => file_config.llm.unwrap_or_default()
                .into_iter().find(|l| l.name == n)
                .ok_or_else(|| anyhow::anyhow!("'{}' not found in config", n)),
            None => Ok(file_config.default),
        }
    }
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".minusagent").join("config.toml")
}
