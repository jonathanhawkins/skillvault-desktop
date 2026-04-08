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
                // Move contents INTO the destination (no extra nesting)
                fs::create_dir_all(&item_dest)
                    .map_err(|e| format!("Failed to create {}: {}", item_dest.display(), e))?;
                if let Ok(entries) = fs::read_dir(&item_src) {
                    for entry in entries.flatten() {
                        let src_path = entry.path();
                        let file_name = entry.file_name();
                        let dest_path = item_dest.join(&file_name);
                        // Overwrite existing files
                        if dest_path.exists() {
                            if dest_path.is_dir() {
                                let _ = fs::remove_dir_all(&dest_path);
                            } else {
                                let _ = fs::remove_file(&dest_path);
                            }
                        }
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

/// Uninstall a skill (soft delete to .trash/)
pub fn uninstall(skill_name: &str) -> Result<(), String> {
    let skills_dir = get_skills_dir()?;
    let skill_dir = skills_dir.join(skill_name);

    if !skill_dir.exists() {
        return Err(format!("Skill '{}' not found", skill_name));
    }

    let trash_dir = skills_dir.join(".trash");
    fs::create_dir_all(&trash_dir)
        .map_err(|e| format!("Failed to create .trash dir: {}", e))?;

    let timestamp = chrono_simple_timestamp();
    let backup_name = format!("{}-{}", skill_name, timestamp);
    let backup_path = trash_dir.join(&backup_name);

    fs::rename(&skill_dir, &backup_path)
        .map_err(|e| format!("Failed to move skill to trash: {}", e))?;

    Ok(())
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
