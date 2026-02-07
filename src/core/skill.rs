use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SkillContext {
    #[default]
    Inline,
    Fork,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub context: SkillContext,
    pub disable_model_invocation: bool,
    pub script: String,
    pub parameters: Option<Value>,
}

impl Skill {
    pub fn parse(content: &str, default_name: &str) -> Result<Self> {
        let (meta, script) = parse_frontmatter(content);

        let name = meta.get("name")
            .cloned()
            .unwrap_or_else(|| default_name.to_string());

        let description = meta.get("description")
            .cloned()
            .unwrap_or_default();

        let context = match meta.get("context").map(|s| s.as_str()) {
            Some("fork") => SkillContext::Fork,
            _ => SkillContext::Inline,
        };

        let disable_model_invocation = meta.get("disable-model-invocation")
            .map(|v| v == "true")
            .unwrap_or(false);

        let parameters = meta.get("parameters")
            .and_then(|v| serde_json::from_str(v).ok());

        Ok(Self {
            name,
            description,
            context,
            disable_model_invocation,
            script,
            parameters,
        })
    }

}

fn parse_frontmatter(content: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut meta = std::collections::HashMap::new();

    if !content.starts_with("---") {
        return (meta, content.to_string());
    }

    let rest = &content[3..];
    let Some(end) = rest.find("\n---") else {
        return (meta, content.to_string());
    };

    let frontmatter = &rest[..end];
    let body = rest[end + 4..].trim_start().to_string();

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            meta.insert(key, value);
        }
    }

    (meta, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let content = r#"---
name: deploy
description: Deploy the application
context: fork
disable-model-invocation: true
---

Deploy steps:
1. Build
2. Push"#;

        let skill = Skill::parse(content, "default").unwrap();
        assert_eq!(skill.name, "deploy");
        assert_eq!(skill.description, "Deploy the application");
        assert_eq!(skill.context, SkillContext::Fork);
        assert!(skill.disable_model_invocation);
        assert!(skill.script.contains("Deploy steps:"));
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let content = "Just some instructions\n\nMore details here.";
        let skill = Skill::parse(content, "my-skill").unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "");
        assert_eq!(skill.context, SkillContext::Inline);
        assert!(!skill.disable_model_invocation);
    }

}
