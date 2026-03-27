use crate::state::{LocalSkill, SkillSource, SkillvaultMeta};
use std::fs;
use std::path::Path;

/// Scan a skills directory and return metadata for each installed skill
pub fn scan_skills_dir(skills_dir: &Path, project: Option<&str>) -> Vec<LocalSkill> {
    if !skills_dir.exists() {
        return vec![];
    }

    let mut skills = Vec::new();

    let entries = match fs::read_dir(skills_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dirs and .trash
        if name.starts_with('.') {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        let description = if skill_md.exists() {
            parse_skill_description(&skill_md)
        } else {
            String::new()
        };

        let file_count = count_files(&path);
        let has_scripts = path.join("scripts").is_dir();
        let has_subagents = path.join("subagents").is_dir();
        let has_references = path.join("references").is_dir();

        // Check for .skillvault-meta.json
        let meta_path = path.join(".skillvault-meta.json");
        let (source, package_id, installed_version) = if meta_path.exists() {
            match fs::read_to_string(&meta_path) {
                Ok(content) => match serde_json::from_str::<SkillvaultMeta>(&content) {
                    Ok(meta) => (
                        SkillSource::Skillvault,
                        Some(meta.package_id),
                        Some(meta.version),
                    ),
                    Err(_) => (SkillSource::Local, None, None),
                },
                Err(_) => (SkillSource::Local, None, None),
            }
        } else {
            (SkillSource::Local, None, None)
        };

        skills.push(LocalSkill {
            name,
            description,
            path: path.to_string_lossy().to_string(),
            file_count,
            has_scripts,
            has_subagents,
            has_references,
            source,
            package_id,
            installed_version,
            project: project.map(|s| s.to_string()),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Scan ~/.claude/skills/ (global) — backward compat wrapper
pub fn scan_skills(claude_dir: &Path) -> Result<Vec<LocalSkill>, String> {
    Ok(scan_skills_dir(&claude_dir.join("skills"), None))
}

/// Parse the description from SKILL.md frontmatter
pub(crate) fn parse_skill_description(path: &Path) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Parse YAML frontmatter between --- markers
    if !content.starts_with("---") {
        return content
            .lines()
            .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .unwrap_or("")
            .trim()
            .to_string();
    }

    let after_first = &content[3..];
    if let Some(end) = after_first.find("---") {
        let frontmatter = &after_first[..end];
        let lines: Vec<&str> = frontmatter.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("description:") {
                let val = trimmed["description:".len()..].trim();

                if val == ">-" || val == "|-" || val == ">" || val == "|" {
                    let mut parts = Vec::new();
                    for next_line in &lines[i + 1..] {
                        if next_line.starts_with(' ') || next_line.starts_with('\t') {
                            parts.push(next_line.trim());
                        } else {
                            break;
                        }
                    }
                    let sep = if val.starts_with('>') { " " } else { "\n" };
                    return parts.join(sep);
                }

                let val = val.trim_matches('"').trim_matches('\'');
                return val.to_string();
            }
        }
    }

    String::new()
}

pub(crate) fn count_files(dir: &Path) -> u32 {
    let mut count = 0u32;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files(&path);
            }
        }
    }
    count
}
