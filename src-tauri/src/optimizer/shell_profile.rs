use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const BLOCK_START: &str = "# --- SkillVault Claude Optimizer (start) ---";
const BLOCK_END: &str = "# --- SkillVault Claude Optimizer (end) ---";

/// Detect the user's shell profile path
pub fn get_shell_profile_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    // Check $SHELL env var
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("bash") {
        // Prefer .bashrc, fall back to .bash_profile
        let bashrc = home.join(".bashrc");
        if bashrc.exists() {
            return bashrc;
        }
        return home.join(".bash_profile");
    }

    // Default to zsh (macOS default)
    home.join(".zshrc")
}

/// Check if the SkillVault optimizer block exists in content
pub fn content_has_block(content: &str) -> bool {
    content.contains(BLOCK_START)
}

/// Check if the SkillVault optimizer block exists in the shell profile
pub fn has_block(profile_path: &PathBuf) -> bool {
    if let Ok(content) = fs::read_to_string(profile_path) {
        content_has_block(&content)
    } else {
        false
    }
}

/// Read env vars from the SkillVault block in content string
pub fn parse_env_vars_from_content(content: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == BLOCK_START {
            in_block = true;
            continue;
        }
        if line.trim() == BLOCK_END {
            break;
        }
        if in_block {
            if let Some(rest) = line.trim().strip_prefix("export ") {
                if let Some((key, value)) = rest.split_once('=') {
                    vars.insert(key.to_string(), value.to_string());
                }
            }
        }
    }
    vars
}

/// Read env vars from the SkillVault block in shell profile
pub fn read_env_vars_from_profile(profile_path: &PathBuf) -> HashMap<String, String> {
    match fs::read_to_string(profile_path) {
        Ok(content) => parse_env_vars_from_content(&content),
        Err(_) => HashMap::new(),
    }
}

/// Read the raw SkillVault block content from shell profile
pub fn read_block(profile_path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(profile_path).ok()?;
    let start_idx = content.find(BLOCK_START)?;
    let end_idx = content.find(BLOCK_END)?;
    if end_idx <= start_idx {
        return None; // Corrupted: end marker before start marker
    }
    let end = end_idx + BLOCK_END.len();
    Some(content[start_idx..end].to_string())
}

/// Generate the full block text (for preview or writing)
pub fn generate_block(env_exports: &str) -> String {
    format!(
        "{}\n# Applied by SkillVault Desktop\n{}\n{}",
        BLOCK_START, env_exports, BLOCK_END
    )
}

/// Write the optimization block to the shell profile
pub fn write_block(profile_path: &PathBuf, env_exports: &str) -> Result<(), String> {
    let block = generate_block(env_exports);

    let content = if profile_path.exists() {
        fs::read_to_string(profile_path)
            .map_err(|e| format!("Failed to read {}: {}", profile_path.display(), e))?
    } else {
        String::new()
    };

    // Create backup before modifying
    if profile_path.exists() {
        let backup_name = format!(
            "{}.skillvault-backup",
            profile_path.file_name().unwrap_or_default().to_string_lossy()
        );
        let backup = profile_path.with_file_name(backup_name);
        fs::copy(profile_path, &backup)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    let new_content = if content.contains(BLOCK_START) && content.contains(BLOCK_END) {
        // Replace existing complete block
        let start_idx = content
            .find(BLOCK_START)
            .ok_or("Block start marker not found")?;
        let end_idx = content
            .find(BLOCK_END)
            .ok_or("Block end marker not found")?;
        let end = end_idx + BLOCK_END.len();
        if end_idx > start_idx {
            format!("{}{}{}", &content[..start_idx], block, &content[end..])
        } else {
            // Corrupted: end before start — strip both orphaned markers, append fresh block
            let cleaned = content
                .replace(BLOCK_START, "")
                .replace(BLOCK_END, "");
            format!("{}\n\n{}\n", cleaned.trim_end(), block)
        }
    } else if content.contains(BLOCK_START) {
        // Orphaned start marker (no end) — strip it, append fresh block
        let cleaned = content.replace(BLOCK_START, "");
        format!("{}\n\n{}\n", cleaned.trim_end(), block)
    } else if content.contains(BLOCK_END) {
        // Orphaned end marker (no start) — strip it, append fresh block
        let cleaned = content.replace(BLOCK_END, "");
        format!("{}\n\n{}\n", cleaned.trim_end(), block)
    } else {
        // No existing block — append
        if content.is_empty() {
            block
        } else {
            format!("{}\n\n{}\n", content.trim_end(), block)
        }
    };

    fs::write(profile_path, new_content)
        .map_err(|e| format!("Failed to write {}: {}", profile_path.display(), e))?;

    Ok(())
}

/// Remove the optimization block from the shell profile
pub fn remove_block(profile_path: &PathBuf) -> Result<(), String> {
    if !profile_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(profile_path)
        .map_err(|e| format!("Failed to read {}: {}", profile_path.display(), e))?;

    if !content.contains(BLOCK_START) && !content.contains(BLOCK_END) {
        return Ok(());
    }

    // Handle partial blocks: if only one marker exists, just strip it
    let (before, after) = if let (Some(start_idx), Some(end_idx)) =
        (content.find(BLOCK_START), content.find(BLOCK_END))
    {
        if end_idx > start_idx {
            let end = end_idx + BLOCK_END.len();
            (content[..start_idx].trim_end(), content[end..].trim_start())
        } else {
            // Corrupted ordering — strip both markers
            let cleaned = content.replace(BLOCK_START, "").replace(BLOCK_END, "");
            fs::write(profile_path, cleaned.trim())
                .map_err(|e| format!("Failed to write {}: {}", profile_path.display(), e))?;
            return Ok(());
        }
    } else {
        // Only one marker — strip whichever exists
        let cleaned = content.replace(BLOCK_START, "").replace(BLOCK_END, "");
        fs::write(profile_path, cleaned.trim())
            .map_err(|e| format!("Failed to write {}: {}", profile_path.display(), e))?;
        return Ok(());
    };

    let new_content = if before.is_empty() && after.is_empty() {
        String::new()
    } else if before.is_empty() {
        after.to_string()
    } else if after.is_empty() {
        format!("{}\n", before)
    } else {
        format!("{}\n\n{}", before, after)
    };

    fs::write(profile_path, new_content)
        .map_err(|e| format!("Failed to write {}: {}", profile_path.display(), e))?;

    Ok(())
}
