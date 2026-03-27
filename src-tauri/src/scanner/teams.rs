use crate::state::Team;
use std::fs;
use std::path::Path;

/// Scan ~/.claude/teams/*/ for team directories
pub fn scan_teams(claude_dir: &Path) -> Result<Vec<Team>, String> {
    let teams_dir = claude_dir.join("teams");
    if !teams_dir.exists() {
        return Ok(vec![]);
    }

    let mut teams = Vec::new();

    let entries = fs::read_dir(&teams_dir)
        .map_err(|e| format!("Failed to read teams dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        let config_path = path.join("config.json");
        let (description, member_count) = if config_path.exists() {
            parse_team_config(&config_path)
        } else {
            // Count files in directory as a rough member estimate
            let count = fs::read_dir(&path)
                .map(|entries| entries.flatten().count() as u32)
                .unwrap_or(0);
            (None, count)
        };

        teams.push(Team {
            name,
            description,
            member_count,
            path: path.to_string_lossy().to_string(),
        });
    }

    teams.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(teams)
}

fn parse_team_config(config_path: &Path) -> (Option<String>, u32) {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return (None, 0),
    };

    let config: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, 0),
    };

    let description = config
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let member_count = config
        .get("member_count")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| {
            config
                .get("members")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u64)
                .unwrap_or(0)
        }) as u32;

    (description, member_count)
}
