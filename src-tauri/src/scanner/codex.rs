use crate::state::{CodexAgent, CodexConfig, CodexRule, CodexSkill};
use std::fs;
use std::path::Path;

/// Scan ~/.codex/ and project-level .codex/ directories for Codex configuration,
/// rules, skills, and orchestrator agents.
pub fn scan_codex(claude_dir: &Path) -> (Option<CodexConfig>, Vec<CodexRule>, Vec<CodexSkill>, Vec<CodexAgent>) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (None, Vec::new(), Vec::new(), Vec::new()),
    };

    let codex_dir = home.join(".codex");

    // 1. Parse config
    let config = scan_codex_config(&codex_dir);

    // 2. Global rules
    let mut rules = scan_codex_rules(&codex_dir.join("rules"), None);

    // 3. Global skills
    let mut skills = scan_codex_skills(&codex_dir.join("skills"), None);

    // 4. Global orchestrator agents
    let mut agents = scan_codex_agents(&codex_dir.join("orchestrator"), None);

    // 5. Project-level .codex/ directories
    let projects_dir = claude_dir.join("projects");
    if projects_dir.exists() {
        if let Ok(entries) = fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let encoded = entry.file_name().to_string_lossy().to_string();
                if let Some(decoded) = super::rules::decode_project_path_pub(&encoded) {
                    let project_codex_dir = Path::new(&decoded).join(".codex");
                    if project_codex_dir.exists() {
                        let project_name = Path::new(&decoded)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| decoded.clone());

                        rules.append(&mut scan_codex_rules(
                            &project_codex_dir.join("rules"),
                            Some(&project_name),
                        ));
                        skills.append(&mut scan_codex_skills(
                            &project_codex_dir.join("skills"),
                            Some(&project_name),
                        ));
                        agents.append(&mut scan_codex_agents(
                            &project_codex_dir.join("orchestrator"),
                            Some(&project_name),
                        ));
                    }
                }
            }
        }
    }

    (config, rules, skills, agents)
}

/// Parse ~/.codex/config.toml without a TOML crate.
/// Extracts `model = "..."` and `[projects."<path>"]` sections.
pub(crate) fn scan_codex_config(codex_dir: &Path) -> Option<CodexConfig> {
    let config_path = codex_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).ok()?;

    let mut model: Option<String> = None;
    let mut trusted_projects: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Extract model = "..."
        if trimmed.starts_with("model") {
            if let Some(val) = extract_toml_string_value(trimmed) {
                model = Some(val);
            }
        }

        // Extract [projects."<path>"] sections
        if trimmed.starts_with("[projects.") {
            let inner = trimmed
                .trim_start_matches("[projects.")
                .trim_end_matches(']');
            let project_path = inner.trim_matches('"').trim_matches('\'');
            if !project_path.is_empty() {
                trusted_projects.push(project_path.to_string());
            }
        }
    }

    Some(CodexConfig {
        model,
        trusted_projects,
        config_path: config_path.to_string_lossy().to_string(),
    })
}

/// Extract the string value from a line like `key = "value"`
pub(crate) fn extract_toml_string_value(line: &str) -> Option<String> {
    let eq_pos = line.find('=')?;
    let val = line[eq_pos + 1..].trim();
    let val = val.trim_matches('"').trim_matches('\'');
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

/// Scan a rules directory for .md files
pub(crate) fn scan_codex_rules(rules_dir: &Path, project: Option<&str>) -> Vec<CodexRule> {
    let mut rules = Vec::new();
    if !rules_dir.exists() {
        return rules;
    }

    if let Ok(entries) = fs::read_dir(rules_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") || name.starts_with('.') {
                continue;
            }

            let content = fs::read_to_string(&path).unwrap_or_default();
            let preview: String = content.chars().take(200).collect();

            rules.push(CodexRule {
                name: name.trim_end_matches(".md").to_string(),
                path: path.to_string_lossy().to_string(),
                preview,
                project: project.map(|s| s.to_string()),
            });
        }
    }

    rules.sort_by(|a, b| a.name.cmp(&b.name));
    rules
}

/// Scan a skills directory for skill files/directories
pub(crate) fn scan_codex_skills(skills_dir: &Path, project: Option<&str>) -> Vec<CodexSkill> {
    let mut skills = Vec::new();
    if !skills_dir.exists() {
        return skills;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            let description = if path.is_dir() {
                // Look for SKILL.md or README.md inside
                let skill_md = path.join("SKILL.md");
                let readme_md = path.join("README.md");
                let desc_file = if skill_md.exists() {
                    Some(skill_md)
                } else if readme_md.exists() {
                    Some(readme_md)
                } else {
                    None
                };
                desc_file
                    .and_then(|f| fs::read_to_string(f).ok())
                    .map(|c| c.chars().take(200).collect::<String>())
                    .unwrap_or_default()
            } else if path.is_file() && name.ends_with(".md") {
                let content = fs::read_to_string(&path).unwrap_or_default();
                content.chars().take(200).collect()
            } else {
                continue;
            };

            // For directory skills, point to SKILL.md inside (so file-detail can read it)
            let display_path = if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    skill_md.to_string_lossy().to_string()
                } else {
                    path.to_string_lossy().to_string()
                }
            } else {
                path.to_string_lossy().to_string()
            };

            skills.push(CodexSkill {
                name: name.trim_end_matches(".md").to_string(),
                path: display_path,
                description,
                project: project.map(|s| s.to_string()),
            });
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Scan an orchestrator directory for agent files/subdirectories
pub(crate) fn scan_codex_agents(orchestrator_dir: &Path, project: Option<&str>) -> Vec<CodexAgent> {
    let mut agents = Vec::new();
    if !orchestrator_dir.exists() {
        return agents;
    }

    if let Ok(entries) = fs::read_dir(orchestrator_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            // Skip state/artifact files — only show real agent definitions
            if name.ends_with(".last-assignment")
                || name.ends_with(".log")
                || name.ends_with(".tmp")
                || name.ends_with(".json")
            {
                continue;
            }

            // Accept both files and directories as agents
            if path.is_file() || path.is_dir() {
                agents.push(CodexAgent {
                    name: name.trim_end_matches(".md").trim_end_matches(".toml").to_string(),
                    project: project.map(|s| s.to_string()),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    agents
}

#[cfg(test)]
#[path = "codex_tests.rs"]
mod tests;
