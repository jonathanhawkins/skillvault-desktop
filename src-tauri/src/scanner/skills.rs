use crate::state::{LocalSkill, SkillSource, SkillvaultMeta};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::UNIX_EPOCH;

/// Parse a `simple_iso_now` timestamp (format: `{unix_seconds}Z`).
/// Returns 0 for unparseable input.
fn parse_iso_seconds(s: &str) -> u64 {
    s.trim_end_matches('Z').parse::<u64>().unwrap_or(0)
}

/// Walk a skill dir and return the most recent file mtime (unix seconds).
/// Skips `.skillvault-meta.json` itself and any other dotfiles.
fn max_mtime_seconds(dir: &Path) -> u64 {
    let mut max: u64 = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Skip the meta file and hidden files/dirs — they're either ours or noise.
            if name_str.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                let sub = max_mtime_seconds(&path);
                if sub > max {
                    max = sub;
                }
            } else if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(d) = modified.duration_since(UNIX_EPOCH) {
                        let secs = d.as_secs();
                        if secs > max {
                            max = secs;
                        }
                    }
                }
            }
        }
    }
    max
}

/// Returns true when the dir has a `.skillvault-meta.json` whose `synced_at`
/// (or `installed_at` fallback) is older than the latest file mtime in the dir.
/// Returns false for any path without valid meta.
pub fn detect_local_changes(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let meta_path = path.join(".skillvault-meta.json");
    if !meta_path.exists() {
        return false;
    }
    let content = match fs::read_to_string(&meta_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let meta: SkillvaultMeta = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let ref_str = if !meta.synced_at.is_empty() {
        meta.synced_at.as_str()
    } else {
        meta.installed_at.as_str()
    };
    let ref_seconds = parse_iso_seconds(ref_str);
    if ref_seconds == 0 {
        return false;
    }
    max_mtime_seconds(path) > ref_seconds + 1
}

/// Read .skillvault-meta.json (if any) and resolve the skill's source info + whether
/// local files have been modified since the last sync.
///
/// Returns (source, package_id, installed_version, has_local_changes).
fn read_meta_and_changes(skill_dir: &Path) -> (SkillSource, Option<String>, Option<String>, bool) {
    let meta_path = skill_dir.join(".skillvault-meta.json");
    if !meta_path.exists() {
        return (SkillSource::Local, None, None, false);
    }

    let content = match fs::read_to_string(&meta_path) {
        Ok(c) => c,
        Err(_) => return (SkillSource::Local, None, None, false),
    };

    let meta: SkillvaultMeta = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return (SkillSource::Local, None, None, false),
    };

    // synced_at is preferred; fall back to installed_at for legacy meta files.
    let ref_str = if !meta.synced_at.is_empty() {
        meta.synced_at.as_str()
    } else {
        meta.installed_at.as_str()
    };
    let ref_seconds = parse_iso_seconds(ref_str);
    let has_changes = if ref_seconds == 0 {
        false
    } else {
        // 1-second grace window — filesystems can round mtimes.
        max_mtime_seconds(skill_dir) > ref_seconds + 1
    };

    (
        SkillSource::Skillvault,
        Some(meta.package_id),
        Some(meta.version),
        has_changes,
    )
}


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
        let has_statusline = path.join("statusline.sh").exists()
            || path.join("statusline.bash").exists()
            || path.join("statusline.py").exists()
            || path.join("statusline.js").exists()
            || path.join("statusline.ts").exists()
            || path.join("statuslines").is_dir();

        let (source, package_id, installed_version, has_local_changes) =
            read_meta_and_changes(&path);

        skills.push(LocalSkill {
            name,
            description,
            path: path.to_string_lossy().to_string(),
            file_count,
            has_scripts,
            has_subagents,
            has_references,
            has_statusline,
            source,
            package_id,
            installed_version,
            project: project.map(|s| s.to_string()),
            has_local_changes,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Scan ~/.claude/skills/ (global) — backward compat wrapper
pub fn scan_skills(claude_dir: &Path) -> Result<Vec<LocalSkill>, String> {
    Ok(scan_skills_dir(&claude_dir.join("skills"), None))
}

/// Discover SKILL.md files across the entire computer using Spotlight (mdfind) on macOS.
/// Returns skills found outside of the already-scanned directories.
pub fn discover_skills_system_wide(already_scanned_paths: &HashSet<String>) -> Vec<LocalSkill> {
    let mut discovered = Vec::new();

    // Use macOS Spotlight to find all SKILL.md files
    let output = match Command::new("mdfind").arg("kMDItemFSName == 'SKILL.md'").output() {
        Ok(o) => o,
        Err(_) => return discovered,
    };

    if !output.status.success() {
        return discovered;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Paths to exclude (noise from plugin caches, sessions, build artifacts)
    let exclude_patterns: &[&str] = &[
        "/plugins/cache/",
        "/plugins/marketplaces/",
        "/local-agent-mode-sessions/",
        "/node_modules/",
        "/.git/",
        "/target/",
        "--claude-worktrees-agent-",
        "/Library/Application Support/Claude/",
    ];

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Skip excluded paths
        if exclude_patterns.iter().any(|pat| line.contains(pat)) {
            continue;
        }

        // SKILL.md is inside the skill directory — get parent
        let skill_md_path = Path::new(line);
        let skill_dir = match skill_md_path.parent() {
            Some(d) => d,
            None => continue,
        };

        let skill_path_str = skill_dir.to_string_lossy().to_string();

        // Skip if already found by direct scanning
        if already_scanned_paths.contains(&skill_path_str) {
            continue;
        }

        if !skill_dir.is_dir() {
            continue;
        }

        let name = match skill_dir.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        let description = parse_skill_description(skill_md_path);
        let file_count = count_files(skill_dir);
        let has_scripts = skill_dir.join("scripts").is_dir();
        let has_subagents = skill_dir.join("subagents").is_dir();
        let has_references = skill_dir.join("references").is_dir();
        let has_statusline = skill_dir.join("statusline.sh").exists()
            || skill_dir.join("statusline.bash").exists()
            || skill_dir.join("statusline.py").exists()
            || skill_dir.join("statusline.js").exists()
            || skill_dir.join("statusline.ts").exists()
            || skill_dir.join("statuslines").is_dir();

        let (source, package_id, installed_version, has_local_changes) =
            read_meta_and_changes(skill_dir);

        // Derive project name from the path — use the containing project directory name
        let project = derive_project_name(skill_dir);

        discovered.push(LocalSkill {
            name,
            description,
            path: skill_path_str,
            file_count,
            has_scripts,
            has_subagents,
            has_references,
            has_statusline,
            source,
            package_id,
            installed_version,
            project,
            has_local_changes,
        });
    }

    discovered.sort_by(|a, b| a.name.cmp(&b.name));
    discovered
}

/// Try to derive a human-friendly project name from a skill's path.
/// e.g. /Users/bone/dev/games/patina/.claude/skills/deploy → "patina"
/// e.g. /Users/bone/dev/games/patina/mcp_agent_mail → "patina"
fn derive_project_name(skill_dir: &Path) -> Option<String> {
    let path_str = skill_dir.to_string_lossy();

    // If inside a .claude/skills/ directory, use the project root
    if let Some(idx) = path_str.find("/.claude/skills/") {
        let project_root = &path_str[..idx];
        return Path::new(project_root)
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
    }

    // Otherwise, try to use the grandparent or parent as project context
    // e.g. /Users/bone/dev/games/patina/mcp_agent_mail/SKILL.md → project = "patina"
    if let Some(parent) = skill_dir.parent() {
        return parent
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
    }

    None
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
