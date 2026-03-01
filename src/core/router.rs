use std::path::PathBuf;

pub struct Router {
    base: PathBuf,
    instructions: PathBuf,
}

impl Router {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Router {
            base: PathBuf::from(home).join(".minusagent"),
            instructions: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/instructions"),
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.base.join(name)
    }

    pub fn instructions_path(&self, name: &str) -> PathBuf {
        self.instructions.join(name)
    }

    pub fn skills_path(&self) -> PathBuf {
        self.instructions.join("skills")
    }
}