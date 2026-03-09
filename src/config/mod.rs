use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub llm: Vec<LlmConfig>,
    #[serde(default)]
    pub skills: SkillsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub max_steps: u32,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    pub name: String,
    pub model: String,
    pub base_url: String,
    pub api_key_env: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Default, Deserialize)]
pub struct SkillsConfig {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub disabled: Vec<String>,
}

fn default_max_tokens() -> u32 {
    4096
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".minusagent")
        .join("config.json")
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = config_path();
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse {}: {}", path.display(), e))
    }

    pub fn load_from(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        serde_json::from_str(&content).map_err(|e| format!("failed to parse {}: {}", path, e))
    }
}

impl LlmConfig {
    pub fn api_key(&self) -> Result<String, String> {
        std::env::var(&self.api_key_env)
            .map_err(|_| format!("environment variable {} is not set", self.api_key_env))
    }
}
