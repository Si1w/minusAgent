pub mod loader;
pub mod registry;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}
