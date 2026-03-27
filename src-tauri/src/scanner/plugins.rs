use crate::state::InstalledPlugin;
use std::fs;
use std::path::Path;

/// Parse installed plugins from ~/.claude/plugins/installed_plugins.json
pub fn scan_plugins(claude_dir: &Path) -> Result<Vec<InstalledPlugin>, String> {
    let plugins_path = claude_dir.join("plugins").join("installed_plugins.json");
    if !plugins_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&plugins_path)
        .map_err(|e| format!("Failed to read installed_plugins.json: {}", e))?;

    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse installed_plugins.json: {}", e))?;

    let mut plugins = Vec::new();

    if let Some(plugins_obj) = data.get("plugins").and_then(|v| v.as_object()) {
        for (full_name, installs) in plugins_obj {
            // Name format: "plugin-name@marketplace"
            let parts: Vec<&str> = full_name.splitn(2, '@').collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let marketplace = parts.get(1).unwrap_or(&"").to_string();

            if let Some(arr) = installs.as_array() {
                for install in arr {
                    let version = install
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let scope = install
                        .get("scope")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    plugins.push(InstalledPlugin {
                        name: name.clone(),
                        marketplace: marketplace.clone(),
                        version,
                        scope,
                    });
                }
            }
        }
    }

    Ok(plugins)
}
