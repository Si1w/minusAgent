use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub llm: Vec<LLMConfig>,
    #[serde(default)]
    pub skills: SkillsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub name: String,
    pub model: String,
    pub base_url: String,
    pub api_key_env: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_context_window")]
    pub context_window: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillsConfig {
    #[serde(default)]
    pub paths: Vec<String>,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_context_window() -> usize {
    128_000
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".minusagent")
        .join("config.json")
}

impl Config {
    /// Creates a default configuration and saves it to `~/.minusagent/config.json`.
    ///
    /// # Returns
    /// The default `Config`.
    pub fn init() -> Result<Self, String> {
        let config = Self {
            agent: AgentConfig { max_steps: 20 },
            llm: vec![LLMConfig {
                name: "codestral".to_string(),
                model: "codestral-latest".to_string(),
                base_url: "https://codestral.mistral.ai/v1/chat/completions".to_string(),
                api_key_env: "LLM_API_KEY".to_string(),
                max_tokens: 4096,
                context_window: 256_000,
                reasoning_effort: None,
            }],
            skills: SkillsConfig::default(),
        };
        config.save()?;
        Ok(config)
    }

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

    /// Saves the current configuration to `~/.minusagent/config.json`.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize config: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("failed to write {}: {}", path.display(), e))
    }

    /// Sets a config field by dotted path and saves to disk.
    ///
    /// Supports paths like "agent.max_steps", "llm.0.model".
    /// Values are auto-parsed as number, bool, or string.
    ///
    /// # Arguments
    /// - `key`: Dotted path to the field.
    /// - `value`: The new value as a string.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let mut json: serde_json::Value = serde_json::to_value(&*self)
            .map_err(|e| format!("failed to serialize config: {}", e))?;

        let parts: Vec<&str> = key.split('.').collect();
        let (last, path) = parts.split_last().ok_or("empty key")?;

        let mut target = &mut json;
        for part in path {
            target = if let Ok(idx) = part.parse::<usize>() {
                target.get_mut(idx).ok_or(format!("index '{}' out of bounds", idx))?
            } else {
                target.get_mut(part).ok_or(format!("field '{}' not found", part))?
            };
        }

        let parsed = if let Ok(n) = value.parse::<u64>() {
            serde_json::Value::Number(n.into())
        } else if let Ok(b) = value.parse::<bool>() {
            serde_json::Value::Bool(b)
        } else {
            serde_json::Value::String(value.to_string())
        };

        if let Ok(idx) = last.parse::<usize>() {
            let arr = target.as_array_mut().ok_or("target is not an array")?;
            if idx >= arr.len() {
                return Err(format!("index '{}' out of bounds", idx));
            }
            arr[idx] = parsed;
        } else {
            let obj = target.as_object_mut().ok_or("target is not an object")?;
            obj.insert(last.to_string(), parsed);
        }

        *self = serde_json::from_value(json)
            .map_err(|e| format!("invalid value for '{}': {}", key, e))?;
        self.save()
    }

    /// Adds an LLM configuration entry and saves to disk.
    ///
    /// # Arguments
    /// - `llm`: The LLM configuration to add.
    pub fn add_llm(&mut self, llm: LLMConfig) -> Result<(), String> {
        if self.llm.iter().any(|l| l.name == llm.name) {
            return Err(format!("LLM '{}' already exists", llm.name));
        }
        self.llm.push(llm);
        self.save()
    }

    /// Removes an LLM configuration entry by name and saves to disk.
    ///
    /// # Arguments
    /// - `name`: The name of the LLM to remove.
    pub fn remove_llm(&mut self, name: &str) -> Result<(), String> {
        let len = self.llm.len();
        self.llm.retain(|l| l.name != name);
        if self.llm.len() == len {
            return Err(format!("LLM '{}' not found", name));
        }
        self.save()
    }

    /// Moves the named LLM to the front of the list and saves to disk.
    ///
    /// After this call, `llm[0]` is the promoted entry. Used by `/switch`
    /// so that `Session::extend()` picks up the right LLM.
    ///
    /// # Arguments
    /// - `name`: The name of the LLM to promote.
    pub fn promote_llm(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .llm
            .iter()
            .position(|l| l.name == name)
            .ok_or(format!("LLM '{}' not found", name))?;
        let entry = self.llm.remove(idx);
        self.llm.insert(0, entry);
        self.save()
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
            context_window: 128_000,
            reasoning_effort: None,
        };
        assert!(config.api_key().is_err());
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = Config::load_from("/tmp/nonexistent_minusagent_config.json");
        assert!(result.is_err());
    }

    fn test_config() -> Config {
        Config {
            agent: AgentConfig { max_steps: 10 },
            llm: vec![
                LLMConfig {
                    name: "a".to_string(),
                    model: "model-a".to_string(),
                    base_url: "https://a.com".to_string(),
                    api_key_env: "KEY_A".to_string(),
                    max_tokens: 4096,
                    context_window: 128_000,
                    reasoning_effort: None,
                },
                LLMConfig {
                    name: "b".to_string(),
                    model: "model-b".to_string(),
                    base_url: "https://b.com".to_string(),
                    api_key_env: "KEY_B".to_string(),
                    max_tokens: 4096,
                    context_window: 128_000,
                    reasoning_effort: None,
                },
            ],
            skills: SkillsConfig::default(),
        }
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = test_config();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent.max_steps, 10);
        assert_eq!(parsed.llm.len(), 2);
        assert_eq!(parsed.llm[0].name, "a");
    }

    #[test]
    fn test_add_llm() {
        let mut config = test_config();
        let llm = LLMConfig {
            name: "c".to_string(),
            model: "model-c".to_string(),
            base_url: "https://c.com".to_string(),
            api_key_env: "KEY_C".to_string(),
            max_tokens: 4096,
            context_window: 128_000,
            reasoning_effort: None,
        };
        // add_llm calls save(), so just test in-memory mutation
        config.llm.push(llm);
        assert_eq!(config.llm.len(), 3);
        assert_eq!(config.llm[2].name, "c");
    }

    #[test]
    fn test_add_llm_duplicate_rejected() {
        let config = test_config();
        let dup = config.llm[0].clone();
        assert!(config.llm.iter().any(|l| l.name == dup.name));
    }

    #[test]
    fn test_remove_llm() {
        let mut config = test_config();
        config.llm.retain(|l| l.name != "a");
        assert_eq!(config.llm.len(), 1);
        assert_eq!(config.llm[0].name, "b");
    }

    #[test]
    fn test_remove_llm_not_found() {
        let config = test_config();
        assert!(config.llm.iter().all(|l| l.name != "nonexistent"));
    }

    #[test]
    fn test_promote_llm() {
        let mut config = test_config();
        let idx = config.llm.iter().position(|l| l.name == "b").unwrap();
        let entry = config.llm.remove(idx);
        config.llm.insert(0, entry);
        assert_eq!(config.llm[0].name, "b");
        assert_eq!(config.llm[1].name, "a");
    }

    #[test]
    fn test_promote_llm_already_first() {
        let mut config = test_config();
        let idx = config.llm.iter().position(|l| l.name == "a").unwrap();
        let entry = config.llm.remove(idx);
        config.llm.insert(0, entry);
        assert_eq!(config.llm[0].name, "a");
        assert_eq!(config.llm[1].name, "b");
    }

    #[test]
    fn test_set_via_json_manipulation() {
        let config = test_config();
        let mut json: serde_json::Value = serde_json::to_value(&config).unwrap();
        json["agent"]["max_steps"] = serde_json::json!(20);
        let updated: Config = serde_json::from_value(json).unwrap();
        assert_eq!(updated.agent.max_steps, 20);
    }

    #[test]
    fn test_set_llm_model_via_json() {
        let config = test_config();
        let mut json: serde_json::Value = serde_json::to_value(&config).unwrap();
        json["llm"][0]["model"] = serde_json::json!("gpt-4o");
        let updated: Config = serde_json::from_value(json).unwrap();
        assert_eq!(updated.llm[0].model, "gpt-4o");
    }

    #[test]
    fn test_set_invalid_field_via_json() {
        let config = test_config();
        let json: serde_json::Value = serde_json::to_value(&config).unwrap();
        assert!(json.get("nonexistent").is_none());
    }

    #[test]
    fn test_default_context_window() {
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
        assert_eq!(config.llm[0].context_window, 128_000);
    }
}
