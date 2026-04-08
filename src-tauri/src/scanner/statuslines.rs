use crate::state::Statusline;
use std::fs;
use std::path::Path;

/// Scan for statusline scripts and directories in ~/.claude/
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
                        if let Some(sl) = make_statusline_from_file(script_path) {
                            seen_paths.insert(sl.path.clone());
                            statuslines.push(sl);
                        }
                    }
                }
            }
        }
    }

    // 2. Scan ~/.claude/ for statusline.* single files
    let extensions = ["sh", "bash", "py", "js", "ts"];
    for ext in &extensions {
        let filename = format!("statusline.{}", ext);
        let path = claude_dir.join(&filename);
        if path.exists() && path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            if !seen_paths.contains(&path_str) {
                if let Some(sl) = make_statusline_from_file(&path) {
                    seen_paths.insert(sl.path.clone());
                    statuslines.push(sl);
                }
            }
        }
    }

    // 3. Scan ~/.claude/statusline/ (singular) — treat as a statusline package directory
    let statusline_dir = claude_dir.join("statusline");
    if statusline_dir.exists() && statusline_dir.is_dir() {
        let path_str = statusline_dir.to_string_lossy().to_string();
        if !seen_paths.contains(&path_str) {
            if let Some(sl) = make_statusline_from_dir(&statusline_dir) {
                seen_paths.insert(sl.path.clone());
                statuslines.push(sl);
            }
        }
    }

    // 4. Scan ~/.claude/statuslines/ (plural) — each entry is a statusline
    let statuslines_dir = claude_dir.join("statuslines");
    if statuslines_dir.exists() && statuslines_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&statuslines_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let path_str = path.to_string_lossy().to_string();
                if seen_paths.contains(&path_str) {
                    continue;
                }

                if path.is_dir() {
                    if let Some(sl) = make_statusline_from_dir(&path) {
                        seen_paths.insert(sl.path.clone());
                        statuslines.push(sl);
                    }
                } else if path.is_file() {
                    if let Some(sl) = make_statusline_from_file(&path) {
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

/// Create a Statusline entry from a single script file
fn make_statusline_from_file(path: &Path) -> Option<Statusline> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let name = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.clone());

    let language = detect_language(path);
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

/// Create a Statusline entry from a directory (package with multiple files)
fn make_statusline_from_dir(dir: &Path) -> Option<Statusline> {
    let name = dir.file_name()?.to_string_lossy().to_string();

    // Find the main script — look for statusline.sh, then any .sh, then any script
    let main_script = find_main_script(dir);
    let language = main_script.as_ref()
        .map(|p| detect_language(p))
        .unwrap_or_else(|| "shell".to_string());

    // Read description from README.md if present
    let readme_path = dir.join("README.md");
    let preview = if readme_path.exists() {
        let content = fs::read_to_string(&readme_path).unwrap_or_default();
        // Get first non-empty, non-heading line
        content.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .take(2)
            .collect::<Vec<_>>()
            .join(" ")
    } else if let Some(ref script) = main_script {
        let content = fs::read_to_string(script).unwrap_or_default();
        content.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with("#!"))
            .take(2)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    // Total size of all files
    let size_bytes = dir_size(dir);

    Some(Statusline {
        name,
        path: dir.to_string_lossy().to_string(),
        language,
        size_bytes,
        preview,
    })
}

/// Find the main entry script in a statusline directory
fn find_main_script(dir: &Path) -> Option<std::path::PathBuf> {
    // Priority order: statusline.sh, then any .sh, then .ts, .js, .py
    let candidates = [
        "statusline.sh",
        "statusline.bash",
        "statusline.ts",
        "statusline.js",
        "statusline.py",
    ];
    for name in &candidates {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }

    // Fall back to first script file found
    let script_exts = ["sh", "bash", "ts", "js", "py"];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if script_exts.contains(&ext.to_string_lossy().as_ref()) {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn detect_language(path: &Path) -> String {
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    match ext.as_str() {
        "sh" | "bash" => "bash",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        _ => "shell",
    }.to_string()
}

fn dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += dir_size(&path);
            }
        }
    }
    total
}
