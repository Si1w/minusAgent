use std::fs;
use std::path::Path;

use crate::skill::SkillMeta;

pub fn parse_frontmatter(path: &Path) -> Result<SkillMeta, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(format!("{}: missing frontmatter", path.display()));
    }

    let after_open = &content[3..];
    let close_pos = after_open
        .find("---")
        .ok_or_else(|| format!("{}: unclosed frontmatter", path.display()))?;

    let frontmatter = &after_open[..close_pos];

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

    let dir = path.parent().unwrap().to_path_buf();
    Ok(SkillMeta {
        name,
        description,
        path: dir,
    })
}

pub fn load_body(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(format!("{}: missing frontmatter", path.display()));
    }

    let after_open = &content[3..];
    let close_pos = after_open
        .find("---")
        .ok_or_else(|| format!("{}: unclosed frontmatter", path.display()))?;

    Ok(after_open[close_pos + 3..].trim().to_string())
}