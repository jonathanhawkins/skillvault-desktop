use crate::api::client::ApiClient;
use crate::state::SkillvaultMeta;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Manifest entry from .skillvault-manifest.json inside a package zip
#[derive(serde::Deserialize)]
struct ManifestItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    #[allow(dead_code)]
    install_dir: Option<String>, // present in manifest but routing is based on item_type
}

#[derive(serde::Deserialize)]
struct Manifest {
    items: Vec<ManifestItem>,
}

/// Resolve the correct ~/.claude/<subdir>/ for a given item type.
/// Returns (parent_dir, use_item_name_as_subdir).
/// For most types, items install as parent_dir/<name>/.
/// For statusline, the item IS the directory (no extra nesting).
fn resolve_install_dir(claude_dir: &Path, item_type: &str) -> (std::path::PathBuf, bool) {
    match item_type {
        "agent" => (claude_dir.join("agents"), true),
        "team" => (claude_dir.join("teams"), true),
        "rule" => (claude_dir.join("rules"), true),
        "statusline" => (claude_dir.join("statusline"), false), // install files directly into ~/.claude/statusline/
        _ => (claude_dir.join("skills"), true),
    }
}

/// Install a package from SkillVault.
/// Uses .skillvault-manifest.json to route each item to its correct ~/.claude/ subdirectory.
/// Falls back to ~/.claude/skills/ for packages without a manifest (backwards compat).
pub async fn install(
    author: &str,
    name: &str,
    token: Option<&str>,
    install_location: Option<&str>,
) -> Result<String, String> {
    let client = ApiClient::new(token.map(|s| s.to_string()));
    let pkg = client.get_package(author, name).await?;

    // Download zip
    let zip_bytes = client.download_package(author, name).await?;

    // Extract to a temp directory first so we can read the manifest
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_dir = home.join(".claude");
    let tmp_dir = claude_dir.join("downloads").join(format!("{}-{}", name, chrono_simple_timestamp()));
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    extract_zip(&zip_bytes, &tmp_dir)?;

    // Try to read .skillvault-manifest.json
    let manifest_path = tmp_dir.join(".skillvault-manifest.json");
    let manifest: Option<Manifest> = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path).ok();
        content.and_then(|c| serde_json::from_str(&c).ok())
    } else {
        None
    };

    let mut installed_items: Vec<String> = Vec::new();

    if let Some(manifest) = manifest {
        // Manifest-based install: route each item to its correct directory
        for item in &manifest.items {
            let item_src = tmp_dir.join(&item.name);
            if !item_src.exists() {
                installed_items.push(format!("{} (skipped — missing from zip)", item.name));
                continue;
            }

            let (dest_dir, use_name_subdir) = if install_location.is_some() && install_location != Some("global") && item.item_type == "skill" {
                (resolve_skills_dir(install_location)?, true)
            } else {
                resolve_install_dir(&claude_dir, &item.item_type)
            };

            fs::create_dir_all(&dest_dir)
                .map_err(|e| format!("Failed to create {}: {}", dest_dir.display(), e))?;

            // For most types: install as dest_dir/<name>/
            // For statusline: install files directly into dest_dir/ (no extra nesting)
            let item_dest = if use_name_subdir {
                dest_dir.join(&item.name)
            } else {
                dest_dir.clone()
            };

            // Backup existing
            if item_dest.exists() && use_name_subdir {
                let trash_dir = claude_dir.join("skills").join(".trash");
                fs::create_dir_all(&trash_dir)
                    .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
                let backup_name = format!("{}-{}", item.name, chrono_simple_timestamp());
                let _ = fs::rename(&item_dest, trash_dir.join(&backup_name));
            }

            if use_name_subdir {
                // Move the whole directory
                fs::rename(&item_src, &item_dest)
                    .map_err(|e| format!("Failed to install '{}' to {}: {}", item.name, dest_dir.display(), e))?;
            } else {
                // Move contents INTO the destination (no extra nesting).
                // Backup existing directory first if it has content.
                if item_dest.exists() && item_dest.is_dir() {
                    // Check if there's anything to back up (non-empty dir)
                    let has_content = fs::read_dir(&item_dest)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false);
                    if has_content {
                        let trash_dir = claude_dir.join("skills").join(".trash");
                        fs::create_dir_all(&trash_dir)
                            .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
                        let backup_name = format!("{}-{}", item.name, chrono_simple_timestamp());
                        // Copy existing to trash before overwriting
                        let backup_path = trash_dir.join(&backup_name);
                        let _ = fs::rename(&item_dest, &backup_path);
                        fs::create_dir_all(&item_dest)
                            .map_err(|e| format!("Failed to recreate {}: {}", item_dest.display(), e))?;
                    }
                } else {
                    fs::create_dir_all(&item_dest)
                        .map_err(|e| format!("Failed to create {}: {}", item_dest.display(), e))?;
                }

                if let Ok(entries) = fs::read_dir(&item_src) {
                    for entry in entries.flatten() {
                        let src_path = entry.path();
                        let file_name = entry.file_name();
                        let dest_path = item_dest.join(&file_name);
                        fs::rename(&src_path, &dest_path)
                            .map_err(|e| format!("Failed to install '{}': {}", file_name.to_string_lossy(), e))?;
                    }
                }
            }

            // Write .skillvault-meta.json for trackable items
            if item_dest.is_dir() {
                let meta = SkillvaultMeta {
                    source: "skillvault".to_string(),
                    package_id: format!("{}/{}", author, name),
                    version: pkg.current_version.clone(),
                    installed_at: simple_iso_now(),
                    auto_update: true,
                };
                let meta_json = serde_json::to_string_pretty(&meta)
                    .map_err(|e| format!("Failed to serialize meta: {}", e))?;
                fs::write(item_dest.join(".skillvault-meta.json"), meta_json)
                    .map_err(|e| format!("Failed to write meta for '{}': {}", item.name, e))?;
            }

            // For statuslines: wire up settings.json so Claude Code actually uses it
            if item.item_type == "statusline" {
                wire_statusline_settings(&claude_dir, &item_dest);
            }

            installed_items.push(format!("{} ({})", item.name, item.item_type));
        }

        // Cleanup temp dir
        let _ = fs::remove_dir_all(&tmp_dir);

        Ok(format!(
            "Installed {}/{} v{} — {}: {}",
            author, name, pkg.current_version,
            installed_items.len(),
            installed_items.join(", ")
        ))
    } else {
        // Legacy install: no manifest, everything goes to skills dir (backwards compat)
        let skills_dir = resolve_skills_dir(install_location)?;
        let target_dir = skills_dir.join(name);

        // Backup existing
        if target_dir.exists() {
            let trash_dir = skills_dir.join(".trash");
            fs::create_dir_all(&trash_dir)
                .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
            let backup_name = format!("{}-{}", name, chrono_simple_timestamp());
            let _ = fs::rename(&target_dir, trash_dir.join(&backup_name));
        }

        // Move from temp to skills dir
        fs::rename(&tmp_dir, &target_dir)
            .map_err(|e| format!("Failed to move to skills dir: {}", e))?;

        // Check for multi-skill sub-directories
        let mut sub_skills: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        sub_skills.push(dir_name.to_string());
                    }
                }
            }
        }

        if !sub_skills.is_empty() {
            for sub_name in &sub_skills {
                let sub_src = target_dir.join(sub_name);
                let sub_dest = skills_dir.join(sub_name);
                if sub_dest.exists() {
                    let trash_dir = skills_dir.join(".trash");
                    fs::create_dir_all(&trash_dir).ok();
                    let _ = fs::rename(&sub_dest, trash_dir.join(format!("{}-{}", sub_name, chrono_simple_timestamp())));
                }
                let _ = fs::rename(&sub_src, &sub_dest);

                let meta = SkillvaultMeta {
                    source: "skillvault".to_string(),
                    package_id: format!("{}/{}", author, name),
                    version: pkg.current_version.clone(),
                    installed_at: simple_iso_now(),
                    auto_update: true,
                };
                if let Ok(json) = serde_json::to_string_pretty(&meta) {
                    let _ = fs::write(sub_dest.join(".skillvault-meta.json"), json);
                }
            }
            let _ = fs::remove_dir_all(&target_dir);
            Ok(format!("Installed {}/{} v{} — {} skills: {}", author, name, pkg.current_version, sub_skills.len(), sub_skills.join(", ")))
        } else {
            let meta = SkillvaultMeta {
                source: "skillvault".to_string(),
                package_id: format!("{}/{}", author, name),
                version: pkg.current_version.clone(),
                installed_at: simple_iso_now(),
                auto_update: true,
            };
            if let Ok(json) = serde_json::to_string_pretty(&meta) {
                let _ = fs::write(target_dir.join(".skillvault-meta.json"), json);
            }
            Ok(format!("Installed {}/{} v{} to {}", author, name, pkg.current_version, target_dir.display()))
        }
    }
}

/// Uninstall an item by name. Searches skills, agents, teams, rules, statuslines.
pub fn uninstall(item_name: &str) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_dir = home.join(".claude");
    uninstall_from(item_name, &claude_dir)
}

/// Testable uninstall: searches the given claude_dir for an item across all type directories.
/// Soft-deletes to .trash/. For statuslines: also removes the statusLine entry from settings.json,
/// but ONLY if it points to the script we installed.
pub(crate) fn uninstall_from(item_name: &str, claude_dir: &Path) -> Result<(), String> {
    let trash_dir = claude_dir.join("skills").join(".trash");

    // Search for the item across all type directories
    let search_dirs = [
        ("skill", claude_dir.join("skills")),
        ("agent", claude_dir.join("agents")),
        ("team", claude_dir.join("teams")),
        ("rule", claude_dir.join("rules")),
    ];

    for (item_type, parent_dir) in &search_dirs {
        let item_dir = parent_dir.join(item_name);
        if item_dir.exists() {
            fs::create_dir_all(&trash_dir)
                .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
            let backup_name = format!("{}-{}-{}", item_type, item_name, chrono_simple_timestamp());
            fs::rename(&item_dir, trash_dir.join(&backup_name))
                .map_err(|e| format!("Failed to move {} to trash: {}", item_type, e))?;
            return Ok(());
        }
    }

    // Check statusline — special case: it's a directory, not a named subdirectory
    let statusline_dir = claude_dir.join("statusline");
    if item_name == "statusline" && statusline_dir.exists() {
        // Check that this was installed by SkillVault (has our meta file)
        let meta_path = statusline_dir.join(".skillvault-meta.json");
        if !meta_path.exists() {
            return Err("Statusline was not installed by SkillVault — refusing to remove".to_string());
        }

        // Remove the statusLine entry from settings.json, but ONLY if it points to our script
        unwire_statusline_settings(claude_dir, &statusline_dir);

        // Soft delete the directory
        fs::create_dir_all(&trash_dir)
            .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
        let backup_name = format!("statusline-{}", chrono_simple_timestamp());
        fs::rename(&statusline_dir, trash_dir.join(&backup_name))
            .map_err(|e| format!("Failed to move statusline to trash: {}", e))?;
        return Ok(());
    }

    Err(format!("'{}' not found in any ~/.claude/ directory", item_name))
}

/// Remove the statusLine entry from settings.json, but ONLY if it points
/// to a script inside the given statusline directory.
fn unwire_statusline_settings(claude_dir: &Path, statusline_dir: &Path) {
    let settings_path = claude_dir.join("settings.json");
    if !settings_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Check if statusLine.command references a script inside our directory.
    // Extract the script path from the command (e.g. "bash /path/to/statusline.sh" → "/path/to/statusline.sh")
    // and check if it starts with our directory path using canonical comparison.
    let should_remove = settings.get("statusLine")
        .and_then(|sl| sl.get("command"))
        .and_then(|c| c.as_str())
        .map(|cmd| {
            // Extract the actual script path from the command
            let script_str = cmd.trim();
            let interpreters = ["python3 ", "ts-node ", "bash ", "zsh ", "python ", "node ", "npx ", "tsx ", "deno ", "sh "];
            let path_part = interpreters.iter()
                .find(|i| script_str.starts_with(*i))
                .map(|i| script_str[i.len()..].trim().split_whitespace().next().unwrap_or(""))
                .unwrap_or(script_str.split_whitespace().next().unwrap_or(""));

            // Resolve ~ in the extracted path
            let resolved = if path_part.starts_with("~/") {
                dirs::home_dir().map(|h| h.join(&path_part[2..]).to_string_lossy().to_string())
                    .unwrap_or_else(|| path_part.to_string())
            } else {
                path_part.to_string()
            };

            // Check if the script is inside our statusline directory
            let script_path = std::path::Path::new(&resolved);
            if let (Ok(script_canon), Ok(dir_canon)) = (
                std::fs::canonicalize(script_path).or_else(|_| Ok::<_, std::io::Error>(script_path.to_path_buf())),
                std::fs::canonicalize(statusline_dir).or_else(|_| Ok::<_, std::io::Error>(statusline_dir.to_path_buf())),
            ) {
                script_canon.starts_with(&dir_canon)
            } else {
                false
            }
        })
        .unwrap_or(false);

    if should_remove {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("statusLine");
        }
        if let Ok(json_str) = serde_json::to_string_pretty(&settings) {
            let _ = fs::write(&settings_path, json_str);
        }
    }
}

pub(crate) fn extract_zip(data: &[u8], target: &Path) -> Result<(), String> {
    let reader = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("Invalid zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        let name = file.name().to_string();

        // Security: skip entries with path traversal
        if name.contains("..") {
            continue;
        }

        // Reject absolute paths
        if std::path::Path::new(&name).is_absolute() {
            continue;
        }

        let out_path = target.join(&name);

        // Verify the resolved path is still within target
        if !out_path.starts_with(target) {
            continue;
        }

        if file.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;

            fs::write(&out_path, &buf)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
    }

    Ok(())
}

fn get_skills_dir() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let skills_dir = home.join(".claude").join("skills");
    fs::create_dir_all(&skills_dir)
        .map_err(|e| format!("Failed to create skills dir: {}", e))?;
    Ok(skills_dir)
}

/// Resolve the skills directory based on install location.
pub(crate) fn resolve_skills_dir(install_location: Option<&str>) -> Result<std::path::PathBuf, String> {
    match install_location {
        None | Some("global") => get_skills_dir(),
        Some(project_path) => {
            let base = Path::new(project_path);
            if !base.exists() {
                return Err(format!("Project path does not exist: {}", project_path));
            }
            let skills_dir = base.join(".claude").join("skills");
            fs::create_dir_all(&skills_dir)
                .map_err(|e| format!("Failed to create project skills dir: {}", e))?;
            Ok(skills_dir)
        }
    }
}

/// After installing a statusline, wire it up in ~/.claude/settings.json
/// so Claude Code actually uses it.
fn wire_statusline_settings(claude_dir: &Path, statusline_dir: &Path) {
    let settings_path = claude_dir.join("settings.json");

    // Find the main script in the statusline directory
    let main_script = ["statusline.sh", "statusline.bash", "statusline.py", "statusline.ts", "statusline.js"]
        .iter()
        .map(|name| statusline_dir.join(name))
        .find(|p| p.exists());

    let script_path = match main_script {
        Some(p) => p,
        None => {
            // Fall back to first .sh file
            if let Ok(entries) = fs::read_dir(statusline_dir) {
                let found = entries.flatten()
                    .find(|e| e.path().extension().map(|ext| ext == "sh").unwrap_or(false));
                match found {
                    Some(e) => e.path(),
                    None => return, // No script found, skip
                }
            } else {
                return;
            }
        }
    };

    let command = format!("bash {}", script_path.to_string_lossy());

    // Read existing settings — NEVER silently replace on parse failure
    let mut settings: serde_json::Value = if settings_path.exists() {
        match fs::read_to_string(&settings_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => {
                    // Settings.json exists but is invalid JSON — don't corrupt it.
                    // Skip wiring; user can fix manually or run /statusline in Claude Code.
                    return;
                }
            },
            Err(_) => return, // Can't read file — skip
        }
    } else {
        serde_json::json!({})
    };

    // Add statusLine config
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": command
    });

    // Write back
    if let Ok(json_str) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(&settings_path, json_str);
    }
}

fn chrono_simple_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

fn simple_iso_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // Simple ISO-ish timestamp without chrono dependency
    format!("{}Z", now.as_secs())
}

#[cfg(test)]
mod tests;
