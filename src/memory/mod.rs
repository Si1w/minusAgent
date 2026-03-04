use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::Context;
use crate::config;

pub struct Memory {
    session_id: String,
    dir: PathBuf,
}

impl Memory {
    pub fn new() -> Self {
        let session_id = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        Self {
            session_id,
            dir: config::sessions_dir(),
        }
    }

    pub fn from_id(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            dir: config::sessions_dir(),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn save(&self, ctx: &Context) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(format!("{}.json", self.session_id));
        let content = serde_json::to_string_pretty(ctx)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Context> {
        let path = self.dir.join(format!("{}.json", self.session_id));
        let content = fs::read_to_string(&path)?;
        let ctx: Context = serde_json::from_str(&content)?;
        Ok(ctx)
    }

    pub fn list() -> Result<Vec<String>> {
        let dir = config::sessions_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut sessions: Vec<String> = fs::read_dir(&dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                name.strip_suffix(".json").map(|s| s.to_string())
            })
            .collect();
        sessions.sort();
        Ok(sessions)
    }
}
