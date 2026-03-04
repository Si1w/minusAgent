use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub instruction: String,
}

pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl Skill {
    pub fn from_str(content: &str) -> Result<Self> {
        let content = content.trim();
        if !content.starts_with("---") {
            anyhow::bail!("SKILL.md must start with YAML frontmatter (---)");
        }

        let after_first = &content[3..];
        let end = after_first.find("---")
            .ok_or_else(|| anyhow::anyhow!("Missing closing --- in frontmatter"))?;

        let yaml_str = &after_first[..end];
        let body = after_first[end + 3..].trim().to_string();

        let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;
        let name = frontmatter["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' in frontmatter"))?
            .to_string();
        let description = frontmatter["description"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'description' in frontmatter"))?
            .to_string();

        Ok(Skill { name, description, instruction: body })
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut skills = Vec::new();

        let global_dir = config::skills_dir();
        if global_dir.exists() {
            if let Ok(entries) = fs::read_dir(&global_dir) {
                for entry in entries.flatten() {
                    let skill_md = entry.path().join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(skill) = Self::load_skill(&skill_md) {
                            skills.push(skill);
                        }
                    }
                }
            }
        }

        SkillRegistry { skills }
    }

    fn load_skill(path: &Path) -> Result<Skill> {
        let content = fs::read_to_string(path)?;
        Skill::from_str(&content)
    }

    pub fn metadata_prompt(&self) -> Option<String> {
        if self.skills.is_empty() {
            return None;
        }
        let mut prompt = String::from("## Available Skills\n\n");
        for skill in &self.skills {
            prompt.push_str(&format!("- `{}`: {}\n", skill.name, skill.description));
        }
        Some(prompt)
    }

    pub fn activate(&self, names: &[String]) -> String {
        let mut instructions = String::new();
        for name in names {
            if let Some(skill) = self.skills.iter().find(|s| s.name == *name) {
                instructions.push_str(&format!("## Skill: {}\n\n{}\n\n", skill.name, skill.instruction));
            } else {
                instructions.push_str(&format!("## Skill: {} (not found)\n\n", name));
            }
        }
        instructions
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }
}