use crate::state::LocalAgent;
use std::fs;
use std::path::Path;

/// Scan ~/.claude/agents/ and return metadata for each agent
pub fn scan_agents(claude_dir: &Path) -> Result<Vec<LocalAgent>, String> {
    let agents_dir = claude_dir.join("agents");
    if !agents_dir.exists() {
        return Ok(vec![]);
    }

    let mut agents = Vec::new();

    let entries = fs::read_dir(&agents_dir)
        .map_err(|e| format!("Failed to read agents dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_os = entry.file_name();
        let filename = name_os.to_string_lossy();

        if filename.starts_with('.') {
            continue;
        }

        if path.is_file() && filename.ends_with(".md") {
            // Direct .md file: ~/.claude/agents/reviewer.md
            let name = filename.trim_end_matches(".md").to_string();
            let description = match fs::read_to_string(&path) {
                Ok(content) => extract_first_paragraph(&content),
                Err(_) => String::new(),
            };
            agents.push(LocalAgent {
                name,
                description,
                path: path.to_string_lossy().to_string(),
            });
        } else if path.is_dir() {
            // Subdirectory (SkillVault-installed agent): ~/.claude/agents/my-agent/my-agent.md
            // Look for a .md file inside
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                    if sub_path.is_file() && sub_name.ends_with(".md") && !sub_name.starts_with('.') {
                        let name = sub_name.trim_end_matches(".md").to_string();
                        let description = match fs::read_to_string(&sub_path) {
                            Ok(content) => extract_first_paragraph(&content),
                            Err(_) => String::new(),
                        };
                        agents.push(LocalAgent {
                            name,
                            description,
                            path: path.to_string_lossy().to_string(), // Use the directory path for uninstall
                        });
                        break; // Only take the first .md in the subdirectory
                    }
                }
            }
        }
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

fn extract_first_paragraph(content: &str) -> String {
    // Skip frontmatter
    let body = if content.starts_with("---") {
        let after = &content[3..];
        match after.find("---") {
            Some(pos) => &after[pos + 3..],
            None => content,
        }
    } else {
        content
    };

    // Find first non-empty, non-heading line
    body.lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .chars()
        .take(200)
        .collect()
}
