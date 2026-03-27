pub mod skills;
pub mod agents;
pub mod hooks;
pub mod plugins;
pub mod mcp;
pub mod teams;
pub mod rules;
pub mod codex;

#[cfg(test)]
mod tests;

use crate::state::LocalState;

/// Scan ~/.claude/ and return the full local state
pub fn scan_all() -> Result<LocalState, String> {
    let claude_dir = get_claude_dir()?;

    // Global skills
    let mut all_skills = skills::scan_skills(&claude_dir).unwrap_or_default();

    // Project-level skills: scan each project's .claude/skills/
    let projects_dir = claude_dir.join("projects");
    if projects_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let encoded = entry.file_name().to_string_lossy().to_string();
                if let Some(decoded) = rules::decode_project_path_pub(&encoded) {
                    let project_skills_dir = std::path::Path::new(&decoded).join(".claude").join("skills");
                    if project_skills_dir.exists() {
                        let project_name = std::path::Path::new(&decoded)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| decoded.clone());
                        let mut project_skills = skills::scan_skills_dir(&project_skills_dir, Some(&project_name));
                        all_skills.append(&mut project_skills);
                    }
                }
            }
        }
    }

    let agents = agents::scan_agents(&claude_dir).unwrap_or_default();
    let hooks = hooks::scan_hooks(&claude_dir).unwrap_or_default();
    let plugins = plugins::scan_plugins(&claude_dir).unwrap_or_default();
    let mcp_servers = mcp::scan_mcp_servers(&claude_dir).unwrap_or_default();
    let teams = teams::scan_teams(&claude_dir).unwrap_or_default();
    let rules = rules::scan_rules(&claude_dir).unwrap_or_default();

    let (codex_config, codex_rules, codex_skills, codex_agents) =
        codex::scan_codex(&claude_dir);

    Ok(LocalState {
        skills: all_skills,
        agents,
        hooks,
        plugins,
        mcp_servers,
        teams,
        rules,
        claude_dir: claude_dir.to_string_lossy().to_string(),
        codex_config,
        codex_rules,
        codex_skills,
        codex_agents,
    })
}

fn get_claude_dir() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_dir = home.join(".claude");
    if !claude_dir.exists() {
        return Err(format!("Claude directory not found: {}", claude_dir.display()));
    }
    Ok(claude_dir)
}
