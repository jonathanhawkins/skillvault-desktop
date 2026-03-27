use crate::state::Hook;
use std::fs;
use std::path::Path;

/// Parse hooks from ~/.claude/settings.json
/// Structure: { "hooks": { "EventName": [{ "matcher": "...", "hooks": [{ "type": "command", "command": "..." }] }] } }
pub fn scan_hooks(claude_dir: &Path) -> Result<Vec<Hook>, String> {
    let settings_path = claude_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    let mut hooks = Vec::new();

    if let Some(hooks_obj) = settings.get("hooks").and_then(|v| v.as_object()) {
        for (event, hook_list) in hooks_obj {
            if let Some(arr) = hook_list.as_array() {
                for entry in arr {
                    let matcher = entry
                        .get("matcher")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    // New format: hooks are nested inside a "hooks" array
                    if let Some(inner_hooks) = entry.get("hooks").and_then(|v| v.as_array()) {
                        for inner in inner_hooks {
                            let hook_type = inner
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("command")
                                .to_string();

                            let command = inner
                                .get("command")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            if !command.is_empty() {
                                hooks.push(Hook {
                                    event: event.clone(),
                                    matcher: matcher.clone(),
                                    hook_type,
                                    command,
                                });
                            }
                        }
                    }

                    // Legacy format: command at top level
                    if let Some(command) = entry.get("command").and_then(|v| v.as_str()) {
                        if !command.is_empty() {
                            let hook_type = entry
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("command")
                                .to_string();

                            hooks.push(Hook {
                                event: event.clone(),
                                matcher: matcher.clone(),
                                hook_type,
                                command: command.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(hooks)
}
