use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub llm: Vec<LLMConfig>,
    #[serde(default)]
    pub skills: SkillsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub max_steps: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
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

impl LLMConfig {
    pub fn api_key(&self) -> Result<String, String> {
        std::env::var(&self.api_key_env)
            .map_err(|_| format!("environment variable {} is not set", self.api_key_env))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let json = r#"{
            "agent": { "max_steps": 10 },
            "llm": [{
                "name": "test",
                "model": "gpt-4",
                "base_url": "https://api.example.com/v1/chat/completions",
                "api_key_env": "TEST_API_KEY",
                "max_tokens": 2048
            }],
            "skills": { "paths": ["/tmp/skills"] }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.agent.max_steps, 10);
        assert_eq!(config.llm.len(), 1);
        assert_eq!(config.llm[0].model, "gpt-4");
        assert_eq!(config.llm[0].max_tokens, 2048);
        assert_eq!(config.skills.paths, vec!["/tmp/skills"]);
    }

    #[test]
    fn test_default_max_tokens() {
        let json = r#"{
            "agent": { "max_steps": 5 },
            "llm": [{
                "name": "test",
                "model": "gpt-4",
                "base_url": "https://api.example.com",
                "api_key_env": "KEY"
            }]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.llm[0].max_tokens, 4096);
    }

    #[test]
    fn test_default_skills_config() {
        let json = r#"{
            "agent": { "max_steps": 5 },
            "llm": [{
                "name": "test",
                "model": "gpt-4",
                "base_url": "https://api.example.com",
                "api_key_env": "KEY"
            }]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.skills.paths.is_empty());
    }

    #[test]
    fn test_multiple_llm_providers() {
        let json = r#"{
            "agent": { "max_steps": 5 },
            "llm": [
                { "name": "a", "model": "gpt-4", "base_url": "https://a.com", "api_key_env": "A" },
                { "name": "b", "model": "claude", "base_url": "https://b.com", "api_key_env": "B" }
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.llm.len(), 2);
        assert_eq!(config.llm[0].name, "a");
        assert_eq!(config.llm[1].name, "b");
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{ "agent": { "max_steps": 5 } }"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_api_key_missing_env() {
        let config = LLMConfig {
            name: "test".to_string(),
            model: "gpt-4".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_key_env: "NONEXISTENT_VAR_12345".to_string(),
            max_tokens: 4096,
        };
        assert!(config.api_key().is_err());
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = Config::load_from("/tmp/nonexistent_minusagent_config.json");
        assert!(result.is_err());
    }
}
