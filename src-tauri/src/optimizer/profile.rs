use crate::state::{OptimizationProfile, OptimizationStatus};
use std::fs;
use std::path::PathBuf;

/// Strip single-line (//) and multi-line (/* */) comments from JSONC content
fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    result.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            result.push(c);
        } else if c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    // Single-line comment: skip until newline
                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                }
                Some(&'*') => {
                    // Multi-line comment: skip until */
                    chars.next(); // consume *
                    let mut prev = ' ';
                    for ch in chars.by_ref() {
                        if prev == '*' && ch == '/' {
                            break;
                        }
                        prev = ch;
                    }
                }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse settings.json content, handling empty files and JSONC comments
fn parse_settings(content: &str) -> Result<serde_json::Value, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(serde_json::json!({}));
    }
    // Try standard JSON first
    if let Ok(val) = serde_json::from_str(trimmed) {
        return Ok(val);
    }
    // Try stripping JSONC comments
    let stripped = strip_jsonc_comments(trimmed);
    let stripped = stripped.trim();
    if stripped.is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(stripped)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))
}

/// Returns the path to ~/.claude/settings.json
pub fn settings_json_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
        .join("settings.json")
}

/// Path to the saved optimization profile
fn profile_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".skillvault")
        .join("optimizer-profile.json")
}

/// Save profile preferences to disk
pub fn save_profile(profile: &OptimizationProfile) -> Result<(), String> {
    let path = profile_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .skillvault dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(profile)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to save profile: {}", e))?;
    Ok(())
}

/// Load saved profile preferences from disk
pub fn load_profile() -> Result<Option<OptimizationProfile>, String> {
    let path = profile_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile: {}", e))?;
    let profile: OptimizationProfile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse profile: {}", e))?;
    Ok(Some(profile))
}

/// Validate profile values are within acceptable ranges
pub fn validate_profile(profile: &OptimizationProfile) -> Result<(), String> {
    if profile.max_thinking_tokens < 5000 || profile.max_thinking_tokens > 200000 {
        return Err(format!(
            "max_thinking_tokens must be between 5000 and 200000, got {}",
            profile.max_thinking_tokens
        ));
    }
    if profile.autocompact_pct < 10 || profile.autocompact_pct > 95 {
        return Err(format!(
            "autocompact_pct must be between 10 and 95, got {}",
            profile.autocompact_pct
        ));
    }
    Ok(())
}

/// Returns the default recommended optimization profile
pub fn default_profile() -> OptimizationProfile {
    OptimizationProfile {
        max_thinking_tokens: 50000,
        autocompact_pct: 40,
        disable_adaptive_thinking: true,
        always_thinking_enabled: true,
        auto_background_tasks: false,
        no_flicker: false,
        skip_permissions: false,
        use_tmux: false,
        experimental_agent_teams: false,
        task_list_id: String::new(),
        extra_cli_args: String::new(),
        model: String::from("claude-opus-4-7"),
        effort_level: String::from("high"),
    }
}

/// Build env var export lines for shell profile
pub fn build_env_export_block(profile: &OptimizationProfile) -> String {
    let mut lines = Vec::new();
    if profile.disable_adaptive_thinking {
        lines.push("export CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1".to_string());
    }
    if profile.auto_background_tasks {
        lines.push("export CLAUDE_AUTO_BACKGROUND_TASKS=1".to_string());
    }
    if profile.no_flicker {
        lines.push("export CLAUDE_CODE_NO_FLICKER=1".to_string());
    }
    if profile.experimental_agent_teams {
        lines.push("export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1".to_string());
    }
    lines.push(format!("export MAX_THINKING_TOKENS={}", profile.max_thinking_tokens));
    lines.push(format!("export CLAUDE_AUTOCOMPACT_PCT_OVERRIDE={}", profile.autocompact_pct));
    if !profile.model.is_empty() {
        lines.push(format!("export ANTHROPIC_MODEL={}", profile.model));
    }
    if !profile.effort_level.is_empty() {
        lines.push(format!("export CLAUDE_CODE_EFFORT_LEVEL={}", profile.effort_level));
    }
    lines.join("\n")
}

/// Build inline env var string for launch scripts (KEY=VAL KEY2=VAL2 format)
pub fn build_env_inline(profile: &OptimizationProfile) -> String {
    let mut parts = Vec::new();
    if profile.disable_adaptive_thinking {
        parts.push("CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING=1".to_string());
    }
    if profile.auto_background_tasks {
        parts.push("CLAUDE_AUTO_BACKGROUND_TASKS=1".to_string());
    }
    if profile.no_flicker {
        parts.push("CLAUDE_CODE_NO_FLICKER=1".to_string());
    }
    if profile.experimental_agent_teams {
        parts.push("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1".to_string());
    }
    if !profile.task_list_id.is_empty() {
        parts.push(format!("CLAUDE_CODE_TASK_LIST_ID={}", profile.task_list_id));
    }
    parts.push(format!("MAX_THINKING_TOKENS={}", profile.max_thinking_tokens));
    parts.push(format!("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE={}", profile.autocompact_pct));
    if !profile.model.is_empty() {
        parts.push(format!("ANTHROPIC_MODEL={}", profile.model));
    }
    if !profile.effort_level.is_empty() {
        parts.push(format!("CLAUDE_CODE_EFFORT_LEVEL={}", profile.effort_level));
    }
    parts.join(" ")
}

/// Read current optimization status from settings.json + shell profile
pub fn get_status() -> Result<OptimizationStatus, String> {
    let settings_path = settings_json_path();
    let settings_exists = settings_path.exists();

    // Read alwaysThinkingEnabled from settings.json
    let always_thinking = if settings_exists {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        let parsed = parse_settings(&content)?;
        parsed
            .get("alwaysThinkingEnabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    };

    // Read shell profile once, extract both block presence and env vars
    let shell_path = super::shell_profile::get_shell_profile_path();
    let shell_path_str = shell_path.to_string_lossy().to_string();
    let shell_content = fs::read_to_string(&shell_path).unwrap_or_default();
    let block_exists = super::shell_profile::content_has_block(&shell_content);
    let env_vars = super::shell_profile::parse_env_vars_from_content(&shell_content);

    // Check shell profile first, fall back to current process environment
    let disable_adaptive = env_vars
        .get("CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING")
        .cloned()
        .or_else(|| std::env::var("CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING").ok());
    let max_thinking = env_vars
        .get("MAX_THINKING_TOKENS")
        .cloned()
        .or_else(|| std::env::var("MAX_THINKING_TOKENS").ok());
    let autocompact = env_vars
        .get("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE")
        .cloned()
        .or_else(|| std::env::var("CLAUDE_AUTOCOMPACT_PCT_OVERRIDE").ok());

    // Calculate score
    let mut score: u8 = 0;
    if always_thinking {
        score += 1;
    }
    if disable_adaptive.is_some() {
        score += 1;
    }
    if max_thinking.is_some() {
        score += 1;
    }
    if autocompact.is_some() {
        score += 1;
    }

    Ok(OptimizationStatus {
        always_thinking_enabled: always_thinking,
        disable_adaptive_thinking: disable_adaptive,
        max_thinking_tokens: max_thinking,
        autocompact_pct_override: autocompact,
        optimization_score: score,
        settings_json_exists: settings_exists,
        shell_profile_path: shell_path_str,
        shell_block_exists: block_exists,
    })
}

/// Set alwaysThinkingEnabled in settings.json
pub fn set_always_thinking(enabled: bool) -> Result<(), String> {
    let settings_path = settings_json_path();

    // Read existing settings or create empty object
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        parse_settings(&content)?
    } else {
        // Ensure ~/.claude/ directory exists
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
        }
        serde_json::json!({})
    };

    // Set or remove the key
    if let Some(obj) = settings.as_object_mut() {
        if enabled {
            obj.insert(
                "alwaysThinkingEnabled".to_string(),
                serde_json::Value::Bool(true),
            );
        } else {
            obj.remove("alwaysThinkingEnabled");
        }
    }

    // Write atomically: temp file + rename
    let output = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    let tmp_path = settings_path.with_extension("json.tmp");
    fs::write(&tmp_path, &output)
        .map_err(|e| format!("Failed to write temp settings file: {}", e))?;
    fs::rename(&tmp_path, &settings_path)
        .map_err(|e| format!("Failed to rename settings.json: {}", e))?;

    Ok(())
}
