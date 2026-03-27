use crate::api::client::ApiClient;
use crate::state::SkillvaultMeta;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Install a package from SkillVault.
/// `install_location` controls where the skill is installed:
///   - None or Some("global") → ~/.claude/skills/<name>/
///   - Some("<project_path>") → <project_path>/.claude/skills/<name>/
pub async fn install(
    author: &str,
    name: &str,
    token: Option<&str>,
    install_location: Option<&str>,
) -> Result<String, String> {
    let client = ApiClient::new(token.map(|s| s.to_string()));

    // Get package info
    let pkg = client.get_package(author, name).await?;

    let skills_dir = resolve_skills_dir(install_location)?;
    let target_dir = skills_dir.join(name);

    // Check for conflicts
    if target_dir.exists() {
        // Backup existing skill
        let trash_dir = skills_dir.join(".trash");
        fs::create_dir_all(&trash_dir)
            .map_err(|e| format!("Failed to create .trash dir: {}", e))?;

        let timestamp = chrono_simple_timestamp();
        let backup_name = format!("{}-{}", name, timestamp);
        let backup_path = trash_dir.join(&backup_name);

        fs::rename(&target_dir, &backup_path)
            .map_err(|e| format!("Failed to backup existing skill: {}", e))?;
    }

    // Download
    let zip_bytes = client.download_package(author, name).await?;

    // Extract
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target dir: {}", e))?;

    extract_zip(&zip_bytes, &target_dir)?;

    // Check if this is a multi-skill package: look for subdirectories containing SKILL.md
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
        // Multi-skill package: move each skill subdirectory to its own skills dir
        let mut installed_names = Vec::new();
        for sub_name in &sub_skills {
            let sub_src = target_dir.join(sub_name);
            let sub_dest = skills_dir.join(sub_name);

            // Handle conflicts for sub-skill
            if sub_dest.exists() {
                let trash_dir = skills_dir.join(".trash");
                fs::create_dir_all(&trash_dir)
                    .map_err(|e| format!("Failed to create .trash dir: {}", e))?;
                let timestamp = chrono_simple_timestamp();
                let backup_name = format!("{}-{}", sub_name, timestamp);
                fs::rename(&sub_dest, trash_dir.join(&backup_name))
                    .map_err(|e| format!("Failed to backup existing skill '{}': {}", sub_name, e))?;
            }

            fs::rename(&sub_src, &sub_dest)
                .map_err(|e| format!("Failed to move skill '{}': {}", sub_name, e))?;

            // Write .skillvault-meta.json in each individual skill directory
            let meta = SkillvaultMeta {
                source: "skillvault".to_string(),
                package_id: format!("{}/{}", author, name),
                version: pkg.current_version.clone(),
                installed_at: simple_iso_now(),
                auto_update: true,
            };
            let meta_json = serde_json::to_string_pretty(&meta)
                .map_err(|e| format!("Failed to serialize meta: {}", e))?;
            fs::write(sub_dest.join(".skillvault-meta.json"), meta_json)
                .map_err(|e| format!("Failed to write meta file for '{}': {}", sub_name, e))?;

            installed_names.push(sub_name.clone());
        }

        // Remove the original container directory
        let _ = fs::remove_dir_all(&target_dir);

        Ok(format!(
            "Installed {}/{} v{} — {} skills: {}",
            author, name, pkg.current_version,
            installed_names.len(),
            installed_names.join(", ")
        ))
    } else {
        // Single skill — existing behavior
        let meta = SkillvaultMeta {
            source: "skillvault".to_string(),
            package_id: format!("{}/{}", author, name),
            version: pkg.current_version.clone(),
            installed_at: simple_iso_now(),
            auto_update: true,
        };

        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?;

        fs::write(target_dir.join(".skillvault-meta.json"), meta_json)
            .map_err(|e| format!("Failed to write meta file: {}", e))?;

        let display_path = target_dir.display();
        Ok(format!("Installed {}/{} v{} to {}", author, name, pkg.current_version, display_path))
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

        let out_path = target.join(&name);

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
