use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
}

pub struct Skill {
    pub frontmatter: FrontMatter,
    pub instruction: String,
}

fn parse_skill_md(dir: &Path) -> Result<(FrontMatter, String)> {
    let path = dir.join("SKILL.md");
    let content = fs::read_to_string(&path)?;
    let inner = content
        .trim()
        .strip_prefix("---")
        .and_then(|s| s.split_once("\n---"))
        .ok_or_else(|| anyhow::anyhow!("invalid frontmatter in {}", path.display()))?;

    let fm: FrontMatter = serde_yaml::from_str(inner.0.trim())?;
    fm.validate()?;

    Ok((fm, inner.1.trim().to_string()))
}

impl FrontMatter {
    fn validate(&self) -> Result<()> {
        let name = &self.name;
        if name.is_empty() || name.len() > 64 {
            anyhow::bail!("name must be 1-64 characters, got {}", name.len());
        }
        if name.starts_with('-') || name.ends_with('-') {
            anyhow::bail!("name must not start or end with hyphen: {}", name);
        }
        if name.contains("--") {
            anyhow::bail!("name must not contain consecutive hyphens: {}", name);
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            anyhow::bail!(
                "name must only contain lowercase letters, digits, and hyphens: {}",
                name
            );
        }
        if self.description.is_empty() || self.description.len() > 1024 {
            anyhow::bail!(
                "description must be 1-1024 characters, got {}",
                self.description.len()
            );
        }
        Ok(())
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let (fm, _) = parse_skill_md(dir)?;
        Ok(fm)
    }

    pub fn register_all_skills(dir: &Path) -> Vec<Self> {
        let mut skills = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(fm) = Self::load(&entry.path()) {
                        skills.push(fm);
                    }
                }
            }
        }
        skills
    }
}

impl Skill {
    pub fn load(dir: &Path) -> Result<Self> {
        let (frontmatter, instruction) = parse_skill_md(dir)?;
        Ok(Skill {
            frontmatter,
            instruction,
        })
    }

    pub fn search(dir: &Path, name: &str) -> Option<Self> {
        let path = dir.join(name);
        if path.is_dir() { Self::load(&path).ok() } else { None }
    }

    pub fn load_instructions(dir: &Path, names: &[String]) -> String {
        let mut parts = Vec::new();
        for name in names {
            match Self::search(dir, name) {
                Some(skill) => parts.push(format!("[{}]\n{}", skill.frontmatter.name, skill.instruction)),
                None => parts.push(format!("[error] skill '{}' not found", name)),
            }
        }
        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_skill(dir: &Path, content: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn test_parse_valid_skill() {
        let dir = Path::new("/tmp/minusagent-test-valid");
        setup_skill(dir, "---\nname: my-skill\ndescription: A test skill\n---\nStep 1: Do something");

        let skill = Skill::load(dir).unwrap();
        assert_eq!(skill.frontmatter.name, "my-skill");
        assert_eq!(skill.frontmatter.description, "A test skill");
        assert_eq!(skill.instruction, "Step 1: Do something");
        assert!(skill.frontmatter.allowed_tools.is_none());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_parse_with_allowed_tools() {
        let dir = Path::new("/tmp/minusagent-test-tools");
        setup_skill(dir, "---\nname: code-review\ndescription: Review code\nallowed-tools: Bash Read\n---\nCheck the code");

        let fm = FrontMatter::load(dir).unwrap();
        assert_eq!(fm.allowed_tools, Some("Bash Read".to_string()));

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_invalid_name_uppercase() {
        let dir = Path::new("/tmp/minusagent-test-upper");
        setup_skill(dir, "---\nname: My-Skill\ndescription: bad\n---\ninstruction");

        assert!(Skill::load(dir).is_err());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_invalid_name_consecutive_hyphens() {
        let dir = Path::new("/tmp/minusagent-test-hyphens");
        setup_skill(dir, "---\nname: my--skill\ndescription: bad\n---\ninstruction");

        assert!(Skill::load(dir).is_err());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_missing_frontmatter() {
        let dir = Path::new("/tmp/minusagent-test-missing");
        setup_skill(dir, "no frontmatter here");

        assert!(Skill::load(dir).is_err());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_instruction_preserves_inner_dashes() {
        let dir = Path::new("/tmp/minusagent-test-dash");
        setup_skill(dir, "---\nname: my-skill\ndescription: test\n---\nLine 1\n---\nLine 2");

        let skill = Skill::load(dir).unwrap();
        assert_eq!(skill.instruction, "Line 1\n---\nLine 2");

        fs::remove_dir_all(dir).ok();
    }
}