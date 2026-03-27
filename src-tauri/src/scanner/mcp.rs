use crate::state::McpServer;
use std::fs;
use std::path::Path;

/// Parse MCP servers from ~/.claude/settings.json → mcpServers
pub fn scan_mcp_servers(claude_dir: &Path) -> Result<Vec<McpServer>, String> {
    let settings_path = claude_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    let mut servers = Vec::new();

    if let Some(mcp_obj) = settings.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, config) in mcp_obj {
            let server_type = config
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("stdio")
                .to_string();

            let url = config
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let command = config
                .get("command")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            servers.push(McpServer {
                name: name.clone(),
                server_type,
                url,
                command,
            });
        }
    }

    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}
