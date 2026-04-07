use crate::state::Statusline;
use std::fs;
use std::path::Path;

/// Scan for statusline scripts in ~/.claude/ and referenced from settings.json
pub fn scan_statuslines(claude_dir: &Path) -> Result<Vec<Statusline>, String> {
    let mut statuslines = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Check settings.json for statusLine.command reference
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(cmd) = json.get("statusLine").and_then(|sl| sl.get("command")).and_then(|c| c.as_str()) {
                    // Resolve ~ to home dir
                    let expanded = if cmd.starts_with("~/") {
                        if let Some(home) = dirs::home_dir() {
                            home.join(&cmd[2..]).to_string_lossy().to_string()
                        } else {
                            cmd.to_string()
                        }
                    } else {
                        cmd.to_string()
                    };

                    let script_path = Path::new(&expanded);
                    if script_path.exists() && script_path.is_file() {
                        if let Some(sl) = make_statusline(script_path) {
                            seen_paths.insert(sl.path.clone());
                            statuslines.push(sl);
                        }
                    }
                }
            }
        }
    }

    // 2. Scan ~/.claude/ for statusline.* files
    let extensions = ["sh", "bash", "py", "js", "ts"];
    for ext in &extensions {
        let filename = format!("statusline.{}", ext);
        let path = claude_dir.join(&filename);
        if path.exists() && path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            if !seen_paths.contains(&path_str) {
                if let Some(sl) = make_statusline(&path) {
                    seen_paths.insert(sl.path.clone());
                    statuslines.push(sl);
                }
            }
        }
    }

    // 3. Scan ~/.claude/statuslines/ directory if it exists
    let statuslines_dir = claude_dir.join("statuslines");
    if statuslines_dir.exists() && statuslines_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&statuslines_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let path_str = path.to_string_lossy().to_string();
                if !seen_paths.contains(&path_str) {
                    if let Some(sl) = make_statusline(&path) {
                        seen_paths.insert(sl.path.clone());
                        statuslines.push(sl);
                    }
                }
            }
        }
    }

    statuslines.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(statuslines)
}

fn make_statusline(path: &Path) -> Option<Statusline> {
    let file_name = path.file_name()?.to_string_lossy().to_string();

    // Derive name: strip extension, use stem
    let name = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.clone());

    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let language = match ext.as_str() {
        "sh" | "bash" => "bash",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        _ => "shell",
    }.to_string();

    let metadata = fs::metadata(path).ok()?;
    let size_bytes = metadata.len();

    let content = fs::read_to_string(path).unwrap_or_default();
    let preview = content.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("#!"))
        .take(3)
        .collect::<Vec<_>>()
        .join("\n");

    Some(Statusline {
        name,
        path: path.to_string_lossy().to_string(),
        language,
        size_bytes,
        preview,
    })
}
