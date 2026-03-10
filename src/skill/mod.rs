use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for a discovered skill.
///
/// # Fields
/// - `name`: Unique skill name from frontmatter.
/// - `description`: Human-readable description from frontmatter.
/// - `path`: Directory containing the SKILL.md file.
#[derive(Debug, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

/// Registry of discovered skills, indexed by name.
///
/// Scans directories for subdirectories containing a `SKILL.md` file with
/// YAML frontmatter (`name` and `description` fields). Skills are looked up
/// by name to load their instruction body.
///
/// # Fields
/// - `skills`: Map from skill name to metadata.
pub struct SkillRegistry {
    skills: HashMap<String, SkillMeta>,
}

impl SkillRegistry {
    /// Creates a registry by scanning the given paths for skill directories.
    ///
    /// Each subdirectory must contain a `SKILL.md` with YAML frontmatter
    /// declaring `name` and `description`.
    ///
    /// # Arguments
    /// - `paths`: Directories to scan for skill subdirectories.
    pub fn new(paths: &[String]) -> Result<Self, String> {
        let mut skills = HashMap::new();
        for base in paths.iter().map(PathBuf::from) {
            if !base.is_dir() {
                continue;
            }
            let entries = fs::read_dir(&base)
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
                let meta = parse_frontmatter(&skill_md)?;
                skills.insert(meta.name.clone(), meta);
            }
        }
        Ok(Self { skills })
    }

    /// Returns all registered skills as a vector.
    pub fn skills(&self) -> Vec<SkillMeta> {
        self.skills.values().cloned().collect()
    }
}

/// Splits a SKILL.md file into frontmatter region and the rest.
///
/// # Arguments
/// - `content`: Raw file content (trimmed of leading whitespace).
///
/// # Returns
/// A tuple of (frontmatter text, body text after closing `---`).
fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err("missing frontmatter".to_string());
    }
    let after_open = &content[3..];
    let close_pos = after_open
        .find("---")
        .ok_or("unclosed frontmatter")?;
    Ok((&after_open[..close_pos], after_open[close_pos + 3..].trim()))
}

/// Parses SKILL.md frontmatter into a `SkillMeta`.
///
/// # Arguments
/// - `path`: Path to the SKILL.md file.
///
/// # Returns
/// The parsed `SkillMeta` with name, description, and parent directory.
fn parse_frontmatter(path: &Path) -> Result<SkillMeta, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let (frontmatter, _) = split_frontmatter(&content)
        .map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut name = None;
    let mut description = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(val) = line.strip_prefix("name:") {
            name = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("description:") {
            description = Some(val.trim().to_string());
        }
    }

    let name = name.ok_or_else(|| format!("{}: missing 'name' in frontmatter", path.display()))?;
    let description = description
        .ok_or_else(|| format!("{}: missing 'description' in frontmatter", path.display()))?;

    Ok(SkillMeta {
        name,
        description,
        path: path.parent().unwrap().to_path_buf(),
    })
}

/// Loads the body content of a SKILL.md file (everything after frontmatter).
///
/// # Arguments
/// - `path`: Path to the SKILL.md file.
///
/// # Returns
/// The body text after the closing `---`.
pub fn load_body(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let (_, body) = split_frontmatter(&content)
        .map_err(|e| format!("{}: {}", path.display(), e))?;

    Ok(body.to_string())
}
