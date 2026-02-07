use std::fs;
use std::path::Path;

use anyhow::Result;

pub const DEFAULT_SKILLS_DIR: &str = "./skills";

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
}

impl Skill {
    pub fn load(dir: &Path) -> Result<Self> {
        let skill_file = dir.join("SKILL.md");
        let content = fs::read_to_string(&skill_file)?;
        let dir_name = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        Self::parse(&content, &dir_name)
    }

    pub fn parse(content: &str, default_name: &str) -> Result<Self> {
        let (meta, script) = parse_frontmatter(content);

        let name = meta.get("name")
            .cloned()
            .unwrap_or_else(|| default_name.to_string());

        let description = meta.get("description")
            .cloned()
            .unwrap_or_else(|| extract_first_paragraph(&script));

        let context = match meta.get("context").map(|s| s.as_str()) {
            Some("fork") => SkillContext::Fork,
            _ => SkillContext::Inline,
        };

        let disable_model_invocation = meta.get("disable-model-invocation")
            .map(|v| v == "true")
            .unwrap_or(false);

        Ok(Self {
            name,
            description,
            context,
            disable_model_invocation,
            script,
        })
    }

    pub fn render(&self, args: &str) -> String {
        let mut result = self.script.clone();

        let parts: Vec<&str> = args.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            result = result.replace(&format!("$ARGUMENTS[{}]", i), part);
            result = result.replace(&format!("${}", i), part);
        }

        result = result.replace("$ARGUMENTS", args);
        result
    }

    pub fn to_prompt_hint(&self) -> String {
        format!("- {}: {}", self.name, self.description)
    }
}

pub fn load_default_skills() -> Result<Vec<Skill>> {
    load_skills(Path::new(DEFAULT_SKILLS_DIR))
}

pub fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !dir.exists() {
        return Ok(skills);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            match Skill::load(&path) {
                Ok(skill) => skills.push(skill),
                Err(e) => eprintln!("Failed to load skill from {:?}: {}", path, e),
            }
        }
    }

    Ok(skills)
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

fn extract_first_paragraph(content: &str) -> String {
    content
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
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
        assert_eq!(skill.description, "Just some instructions");
        assert_eq!(skill.context, SkillContext::Inline);
        assert!(!skill.disable_model_invocation);
    }

    #[test]
    fn test_render_arguments() {
        let skill = Skill {
            name: "test".into(),
            description: "test".into(),
            context: SkillContext::Inline,
            disable_model_invocation: false,
            script: "Deploy $0 to $1\nFull: $ARGUMENTS".into(),
        };

        let rendered = skill.render("app production");
        assert_eq!(rendered, "Deploy app to production\nFull: app production");
    }

    #[test]
    fn test_render_indexed_arguments() {
        let skill = Skill {
            name: "test".into(),
            description: "test".into(),
            context: SkillContext::Inline,
            disable_model_invocation: false,
            script: "Migrate $ARGUMENTS[0] from $ARGUMENTS[1] to $ARGUMENTS[2]".into(),
        };

        let rendered = skill.render("Button React Vue");
        assert_eq!(rendered, "Migrate Button from React to Vue");
    }

    #[test]
    fn test_to_prompt_hint() {
        let skill = Skill {
            name: "search".into(),
            description: "Search for information".into(),
            context: SkillContext::Inline,
            disable_model_invocation: false,
            script: String::new(),
        };

        assert_eq!(skill.to_prompt_hint(), "- search: Search for information");
    }
}
