use std::fs;

use anyhow::Result;
use serde::Deserialize;

use crate::core::router::Router;

const DEFAULT_MAX_TOKENS: usize = 4096;
const DEFAULT_MAX_ITERATIONS: usize = 10;

#[derive(Deserialize, Clone)]
pub struct AgentConfig {
    pub max_iterations: Option<usize>,
    pub default_llm: String,
}

#[derive(Deserialize, Clone)]
pub struct LLMConfig {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub max_tokens: Option<usize>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub agent: AgentConfig,
    pub llm: Vec<LLMConfig>,
}

impl LLMConfig {
    pub fn max_tokens(&self) -> usize {
        self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS)
    }
}

impl AgentConfig {
    pub fn max_iterations(&self) -> usize {
        self.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS)
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        let path = Router::new().path("config.json");
        let content = fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn get_llm(&self, model: Option<&str>) -> Result<LLMConfig> {
        let target = model.unwrap_or(&self.agent.default_llm);
        self.llm.iter()
            .find(|l| l.model == target)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("'{}' not found in config", target))
    }
}

