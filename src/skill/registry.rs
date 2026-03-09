use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::skill::SkillMeta;
use crate::skill::loader;

pub struct SkillRegistry {
    skills: HashMap<String, SkillMeta>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn discover(
        &mut self,
        search_paths: &[PathBuf],
    ) -> Result<(), String> {
        for base in search_paths {
            if !base.is_dir() {
                continue;
            }
            let entries = fs::read_dir(base)
                .map_err(|e| format!("failed to read {}: {}", base.display(), e))?;

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let skill_md = path.join("SKILL.md");
                if !skill_md.exists() {
                    continue;
                }
                let meta = loader::parse_frontmatter(&skill_md)?;
                self.skills.insert(meta.name.clone(), meta);
            }
        }
        Ok(())
    }

    pub fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join(".minusagent").join("skills"));
        }
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".minusagent").join("skills"));
        }
        paths
    }

    pub fn list(&self) -> Vec<&SkillMeta> {
        self.skills.values().collect()
    }

    pub fn get(&self, name: &str) -> Option<&SkillMeta> {
        self.skills.get(name)
    }
}