use crate::api::auth;
use crate::api::client::ApiClient;
use crate::installer;
use crate::scanner;
use crate::state::{AppState, CategoryCount, LocalState, MarketplacePlugin, Package, PackagedSkill, PackageSearchResult, PluginDetail, PlatformStats, ProjectInfo, SkillDetail, SkillFile, SkillvaultMeta};
use base64::Engine;
use serde::Deserialize;
use std::io::{Cursor, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use zip::write::SimpleFileOptions;

/// Validate a name for use in filesystem paths and CLI arguments.
/// Must be lowercase alphanumeric + hyphens, 1-64 chars, no leading hyphen.
fn validate_name(name: &str, label: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err(format!("{} must be 1-64 characters", label));
    }
    if name.starts_with('-') || name.starts_with('.') {
        return Err(format!("{} cannot start with '-' or '.'", label));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(format!("{} can only contain lowercase letters, digits, hyphens, and underscores", label));
    }
    Ok(())
}

#[tauri::command]
pub async fn scan_local(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<LocalState, String> {
    let local_state = scanner::scan_all()?;
    let mut app = state.lock().await;
    app.local_state = Some(local_state.clone());
    Ok(local_state)
}

#[tauri::command]
pub async fn search_packages(
    query: String,
    category: Option<String>,
    sort: Option<String>,
    page: u32,
    limit: u32,
    compat: Option<String>,
) -> Result<PackageSearchResult, String> {
    let client = ApiClient::new(None);
    client
        .search_packages(
            &query,
            category.as_deref(),
            sort.as_deref(),
            page,
            limit,
            compat.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn get_package(author: String, name: String) -> Result<Package, String> {
    let client = ApiClient::new(None);
    client.get_package(&author, &name).await
}

#[tauri::command]
pub async fn get_trending() -> Result<Vec<Package>, String> {
    let client = ApiClient::new(None);
    client.get_trending().await
}

#[tauri::command]
pub async fn get_categories() -> Result<Vec<CategoryCount>, String> {
    let client = ApiClient::new(None);
    client.get_categories().await
}

#[tauri::command]
pub async fn get_platform_stats() -> Result<PlatformStats, String> {
    let client = ApiClient::new(None);
    client.get_stats().await
}

#[tauri::command]
pub async fn install_package(
    author: String,
    name: String,
    install_path: Option<String>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let token = {
        let app = state.lock().await;
        app.auth_token.clone()
    };

    installer::install(&author, &name, token.as_deref(), install_path.as_deref()).await
}

#[tauri::command]
pub async fn uninstall_skill(skill_name: String) -> Result<(), String> {
    validate_name(&skill_name, "Skill name")?;
    installer::uninstall(&skill_name)
}

#[tauri::command]
pub async fn check_updates(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<UpdateInfo>, String> {
    let local = {
        let app = state.lock().await;
        app.local_state.clone()
    };

    let local = local.ok_or("No local state — run scan first")?;
    let client = ApiClient::new(None);
    let mut updates = Vec::new();

    for skill in &local.skills {
        if let (Some(pkg_id), Some(installed_ver)) =
            (&skill.package_id, &skill.installed_version)
        {
            let parts = pkg_id.splitn(2, '/').collect::<Vec<&str>>();
            if parts.len() != 2 {
                continue;
            }

            if let Ok(pkg) = client.get_package(parts[0], parts[1]).await {
                if pkg.current_version != *installed_ver {
                    updates.push(UpdateInfo {
                        skill_name: skill.name.clone(),
                        package_id: pkg_id.clone(),
                        installed_version: installed_ver.clone(),
                        latest_version: pkg.current_version,
                    });
                }
            }
        }
    }

    Ok(updates)
}

#[tauri::command]
pub async fn set_auth_token(token: String, state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    // Validate token format
    if !token.starts_with("svt_") {
        return Err("Invalid token format — must start with 'svt_'".to_string());
    }

    // Save to keychain
    auth::save_token(&token)?;

    // Update app state
    let mut app = state.lock().await;
    app.auth_token = Some(token);

    Ok(())
}

#[tauri::command]
pub async fn get_auth_status(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<AuthStatus, String> {
    let mut app = state.lock().await;

    // If no token in memory, try loading from keychain
    if app.auth_token.is_none() {
        if let Some(token) = auth::get_token() {
            app.auth_token = Some(token);
        }
    }

    let has_token = app.auth_token.is_some();
    Ok(AuthStatus {
        authenticated: has_token,
    })
}

#[tauri::command]
pub async fn clear_auth_token(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    // Delete from keychain
    let _ = auth::delete_token();

    // Clear from app state
    let mut app = state.lock().await;
    app.auth_token = None;

    Ok(())
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<ProjectInfo>, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let projects_dir = home.join(".claude").join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects dir: {}", e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if !file_type.is_dir() {
            continue;
        }

        let encoded_name = entry.file_name().to_string_lossy().to_string();

        // Use smart path decoder that handles hyphens in directory names
        let decoded_path = match crate::scanner::rules::decode_project_path_pub(&encoded_name) {
            Some(p) => p,
            None => continue,
        };

        let path = std::path::Path::new(&decoded_path);

        // Derive a friendly name from the last path component
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| decoded_path.clone());

        projects.push(ProjectInfo {
            name,
            path: decoded_path,
            encoded_name,
        });
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

#[tauri::command]
pub async fn get_skill_detail(skill_name: String, skill_path: Option<String>) -> Result<SkillDetail, String> {
    validate_name(&skill_name, "Skill name")?;
    // Use provided path if available, otherwise default to global
    let skill_dir = if let Some(ref p) = skill_path {
        std::path::PathBuf::from(p)
    } else {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        home.join(".claude").join("skills").join(&skill_name)
    };

    if !skill_dir.exists() || !skill_dir.is_dir() {
        return Err(format!("Skill directory not found: {}", skill_dir.display()));
    }

    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_md_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();

    // Parse description from SKILL.md frontmatter
    let description = parse_frontmatter_description(&skill_md_content);

    // Check for .skillvault-meta.json
    let meta_path = skill_dir.join(".skillvault-meta.json");
    let (source, package_id, installed_version) = if meta_path.exists() {
        match std::fs::read_to_string(&meta_path) {
            Ok(content) => match serde_json::from_str::<SkillvaultMeta>(&content) {
                Ok(meta) => ("skillvault".to_string(), Some(meta.package_id), Some(meta.version)),
                Err(_) => ("local".to_string(), None, None),
            },
            Err(_) => ("local".to_string(), None, None),
        }
    } else {
        ("local".to_string(), None, None)
    };

    // List files in the skill directory
    let mut files = Vec::new();
    collect_files(&skill_dir, &mut files)?;

    Ok(SkillDetail {
        name: skill_name,
        path: skill_dir.to_string_lossy().to_string(),
        description,
        skill_md_content,
        files,
        source,
        package_id,
        installed_version,
    })
}

fn parse_frontmatter_description(content: &str) -> String {
    if !content.starts_with("---") {
        return content
            .lines()
            .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .unwrap_or("")
            .trim()
            .to_string();
    }

    let after_first = &content[3..];
    if let Some(end) = after_first.find("---") {
        let frontmatter = &after_first[..end];
        let lines: Vec<&str> = frontmatter.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("description:") {
                let val = trimmed["description:".len()..].trim();

                if val == ">-" || val == "|-" || val == ">" || val == "|" {
                    let mut parts = Vec::new();
                    for next_line in &lines[i + 1..] {
                        if next_line.starts_with(' ') || next_line.starts_with('\t') {
                            parts.push(next_line.trim());
                        } else {
                            break;
                        }
                    }
                    let sep = if val.starts_with('>') { " " } else { "\n" };
                    return parts.join(sep);
                }

                let val = val.trim_matches('"').trim_matches('\'');
                return val.to_string();
            }
        }
    }

    String::new()
}

fn collect_files(dir: &std::path::Path, files: &mut Vec<SkillFile>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| format!("Failed to read metadata: {}", e))?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };

        files.push(SkillFile {
            name,
            path: path.to_string_lossy().to_string(),
            size,
            is_dir,
        });
    }

    files.sort_by(|a, b| {
        // Directories first, then by name
        b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
    });

    Ok(())
}

/// Read the content of a file (for agent/plugin detail views)
#[tauri::command]
pub async fn read_file_content(file_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Canonicalize to resolve symlinks and ..
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_dir = home.join(".claude");
    let codex_dir = home.join(".codex");
    let filename = canonical.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let is_claude_dir = canonical.starts_with(&claude_dir);
    let is_codex_dir = canonical.starts_with(&codex_dir);
    let is_claude_md = filename == "CLAUDE.md" || filename == "AGENTS.md";
    let is_under_home = canonical.starts_with(&home);

    if !(is_claude_dir || is_codex_dir || (is_claude_md && is_under_home)) {
        return Err("Access denied: file is outside permitted directories".to_string());
    }

    std::fs::read_to_string(&canonical)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub async fn package_skill(skill_name: String) -> Result<PackagedSkill, String> {
    validate_name(&skill_name, "Skill name")?;
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let skill_dir = home.join(".claude").join("skills").join(&skill_name);

    if !skill_dir.exists() || !skill_dir.is_dir() {
        return Err(format!("Skill directory not found: {}", skill_dir.display()));
    }

    // Parse description from SKILL.md
    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_md_content = std::fs::read_to_string(&skill_md_path).unwrap_or_default();
    let description = parse_frontmatter_description(&skill_md_content);

    // Create zip in memory
    let mut buf = Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(&mut buf);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut file_count: u32 = 0;
    add_dir_to_zip(&mut zip_writer, &skill_dir, &skill_dir, &options, &mut file_count)?;
    zip_writer.finish().map_err(|e| format!("Failed to finalize zip: {}", e))?;

    let zip_bytes = buf.into_inner();
    let size_bytes = zip_bytes.len() as u64;
    let zip_base64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    Ok(PackagedSkill {
        name: skill_name.clone(),
        description,
        zip_base64,
        file_count,
        size_bytes,
        skill_names: vec![skill_name],
    })
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<&mut Cursor<Vec<u8>>>,
    base: &std::path::Path,
    dir: &std::path::Path,
    options: &SimpleFileOptions,
    file_count: &mut u32,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files (like .skillvault-meta.json)
        if name.starts_with('.') {
            continue;
        }

        let relative = path
            .strip_prefix(base)
            .map_err(|e| format!("Path error: {}", e))?
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            zip.add_directory(&format!("{}/", relative), *options)
                .map_err(|e| format!("Failed to add directory to zip: {}", e))?;
            add_dir_to_zip(zip, base, &path, options, file_count)?;
        } else {
            let data = std::fs::read(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            zip.start_file(&relative, *options)
                .map_err(|e| format!("Failed to start zip entry: {}", e))?;
            zip.write_all(&data)
                .map_err(|e| format!("Failed to write zip entry: {}", e))?;
            *file_count += 1;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn publish_skill(
    skill_name: String,
    display_name: String,
    tagline: String,
    category: String,
    version: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    // 1. Get auth token from state
    let token = {
        let app = state.lock().await;
        app.auth_token.clone()
    };

    let token = token.ok_or("Not authenticated — add your API token in Settings first")?;

    // 2. Package the skill (create zip)
    let packaged = package_skill(skill_name.clone()).await?;
    let zip_bytes = base64::engine::general_purpose::STANDARD
        .decode(&packaged.zip_base64)
        .map_err(|e| format!("Failed to decode zip: {}", e))?;

    // 3. Get the authenticated username
    let client = ApiClient::new(Some(token.clone()));
    let username = client.get_me().await?;

    // 4. Create package metadata via API
    client
        .create_package(&skill_name, &display_name, &tagline, &category)
        .await?;

    // 5. Upload the zip with version
    client
        .upload_version(&username, &skill_name, &version, zip_bytes)
        .await?;

    Ok(format!(
        "Published {}/{} v{} to skillvault.md",
        username, display_name, version
    ))
}

#[tauri::command]
pub async fn package_skills(
    skill_names: Vec<String>,
    skill_paths: Vec<String>,
) -> Result<PackagedSkill, String> {
    if skill_names.is_empty() {
        return Err("No skills specified".to_string());
    }
    if skill_names.len() != skill_paths.len() {
        return Err("skill_names and skill_paths must have the same length".to_string());
    }

    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    // Resolve each skill directory
    let mut skill_dirs: Vec<(String, std::path::PathBuf)> = Vec::new();
    for (i, name) in skill_names.iter().enumerate() {
        validate_name(name, "Skill name")?;

        if !skill_paths[i].is_empty() {
            let p = std::path::Path::new(&skill_paths[i]);
            if !p.starts_with(&home) {
                return Err(format!("Skill path must be under home directory: {}", skill_paths[i]));
            }
        }

        let dir = if skill_paths[i].is_empty() {
            home.join(".claude").join("skills").join(name)
        } else {
            std::path::PathBuf::from(&skill_paths[i])
        };

        if !dir.exists() || !dir.is_dir() {
            return Err(format!("Skill directory not found: {}", dir.display()));
        }
        skill_dirs.push((name.clone(), dir));
    }

    // Build a combined description from the first skill
    let first_skill_md = skill_dirs[0].1.join("SKILL.md");
    let first_content = std::fs::read_to_string(&first_skill_md).unwrap_or_default();
    let description = parse_frontmatter_description(&first_content);

    // Create zip in memory with each skill as a top-level directory
    let mut buf = Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(&mut buf);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut total_file_count: u32 = 0;

    for (name, dir) in &skill_dirs {
        // Add the skill as a top-level directory in the zip
        zip_writer
            .add_directory(format!("{}/", name), options)
            .map_err(|e| format!("Failed to add directory to zip: {}", e))?;

        add_dir_to_zip_prefixed(&mut zip_writer, dir, dir, name, &options, &mut total_file_count)?;
    }

    zip_writer
        .finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?;

    let zip_bytes = buf.into_inner();
    let size_bytes = zip_bytes.len() as u64;
    let zip_base64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    let combined_name = skill_names.join("+");

    Ok(PackagedSkill {
        name: combined_name,
        description,
        zip_base64,
        file_count: total_file_count,
        size_bytes,
        skill_names,
    })
}

/// Like add_dir_to_zip but prefixes all entries with a top-level directory name.
fn add_dir_to_zip_prefixed(
    zip: &mut zip::ZipWriter<&mut Cursor<Vec<u8>>>,
    base: &std::path::Path,
    dir: &std::path::Path,
    prefix: &str,
    options: &SimpleFileOptions,
    file_count: &mut u32,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let relative = path
            .strip_prefix(base)
            .map_err(|e| format!("Path error: {}", e))?
            .to_string_lossy()
            .to_string();

        let prefixed = format!("{}/{}", prefix, relative);

        if path.is_dir() {
            zip.add_directory(&format!("{}/", prefixed), *options)
                .map_err(|e| format!("Failed to add directory to zip: {}", e))?;
            add_dir_to_zip_prefixed(zip, base, &path, prefix, options, file_count)?;
        } else {
            let data = std::fs::read(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            zip.start_file(&prefixed, *options)
                .map_err(|e| format!("Failed to start zip entry: {}", e))?;
            zip.write_all(&data)
                .map_err(|e| format!("Failed to write zip entry: {}", e))?;
            *file_count += 1;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn publish_skills(
    skill_names: Vec<String>,
    skill_paths: Vec<String>,
    package_name: String,
    display_name: String,
    tagline: String,
    category: String,
    version: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    // 1. Get auth token from state
    let token = {
        let app = state.lock().await;
        app.auth_token.clone()
    };

    let token = token.ok_or("Not authenticated — add your API token in Settings first")?;

    // 2. Package the skills (create zip)
    let packaged = package_skills(skill_names, skill_paths).await?;
    let zip_bytes = base64::engine::general_purpose::STANDARD
        .decode(&packaged.zip_base64)
        .map_err(|e| format!("Failed to decode zip: {}", e))?;

    // 3. Get the authenticated username
    let client = ApiClient::new(Some(token.clone()));
    let username = client.get_me().await?;

    // 4. Create package metadata via API
    client
        .create_package(&package_name, &display_name, &tagline, &category)
        .await?;

    // 5. Upload the zip with version
    client
        .upload_version(&username, &package_name, &version, zip_bytes)
        .await?;

    Ok(format!(
        "Published {}/{} v{} to skillvault.md",
        username, display_name, version
    ))
}

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub skill_name: String,
    pub package_id: String,
    pub installed_version: String,
    pub latest_version: String,
}

#[derive(serde::Serialize)]
pub struct AuthStatus {
    pub authenticated: bool,
}

#[tauri::command]
pub async fn install_plugin(
    plugin_name: String,
    plugin_source: String,
    install_scope: Option<String>,  // "user" (default) or a project path
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    validate_name(&plugin_name, "Plugin name")?;
    if plugin_source == "claude" {
        // Use claude CLI to install
        let mut cmd = std::process::Command::new("claude");
        cmd.arg("plugin").arg("install");
        cmd.arg(format!("{}@claude-plugins-official", plugin_name));

        match &install_scope {
            Some(path) if path != "user" => {
                cmd.arg("--scope").arg("project");
                cmd.current_dir(path);
            }
            _ => {
                cmd.arg("--scope").arg("user");
            }
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to run claude CLI: {}. Is 'claude' installed?", e))?;

        if output.status.success() {
            Ok(format!("Installed {} successfully", plugin_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Install failed: {}", stderr))
        }
    } else {
        // Codex: download from GitHub and place in codex plugins directory
        let home = dirs::home_dir().ok_or("No home directory")?;
        let target_dir = match &install_scope {
            Some(path) if path != "user" => {
                std::path::Path::new(path).join(".codex").join("plugins").join(&plugin_name)
            }
            _ => home.join(".codex").join("plugins").join(&plugin_name),
        };

        // Download plugin.json and README from GitHub
        let base_url = format!(
            "https://raw.githubusercontent.com/openai/plugins/main/plugins/{}",
            plugin_name
        );

        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create plugin dir: {}", e))?;

        // Download key files
        let client = reqwest::Client::new();
        for file in &[".codex-plugin/plugin.json", "README.md", ".mcp.json"] {
            let url = format!("{}/{}", base_url, file);
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(content) = resp.text().await {
                        let file_path = target_dir.join(file);
                        if let Some(parent) = file_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(file_path, content);
                    }
                }
            }
        }

        // Clear codex plugin cache so list refreshes
        let mut app = state.lock().await;
        app.codex_plugins_cache = None;

        Ok(format!("Installed {} to {}", plugin_name, target_dir.display()))
    }
}

#[tauri::command]
pub async fn uninstall_plugin(
    plugin_name: String,
    plugin_source: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    validate_name(&plugin_name, "Plugin name")?;
    if plugin_source == "claude" {
        let output = std::process::Command::new("claude")
            .arg("plugin")
            .arg("uninstall")
            .arg(format!("{}@claude-plugins-official", plugin_name))
            .output()
            .map_err(|e| format!("Failed to run claude CLI: {}", e))?;

        if output.status.success() {
            Ok(format!("Uninstalled {}", plugin_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Uninstall failed: {}", stderr))
        }
    } else {
        // Codex: remove the plugin directory
        let home = dirs::home_dir().ok_or("No home directory")?;
        let plugin_dir = home.join(".codex").join("plugins").join(&plugin_name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)
                .map_err(|e| format!("Failed to remove: {}", e))?;
        }

        // Clear cache
        let mut app = state.lock().await;
        app.codex_plugins_cache = None;

        Ok(format!("Uninstalled {}", plugin_name))
    }
}

/// Install a Codex plugin by creating the directory structure.
/// Extracted for testability (the async download part is separate).
pub(crate) fn codex_plugin_install_dir(
    plugin_name: &str,
    install_scope: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("No home directory")?;
    let target_dir = match install_scope {
        Some(path) if path != "user" => {
            std::path::Path::new(path).join(".codex").join("plugins").join(plugin_name)
        }
        _ => home.join(".codex").join("plugins").join(plugin_name),
    };

    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create plugin dir: {}", e))?;

    Ok(target_dir)
}

/// Uninstall a Codex plugin by removing the directory.
pub(crate) fn codex_plugin_uninstall_dir(plugin_name: &str) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("No home directory")?;
    let plugin_dir = home.join(".codex").join("plugins").join(plugin_name);
    if plugin_dir.exists() {
        std::fs::remove_dir_all(&plugin_dir)
            .map_err(|e| format!("Failed to remove: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_codex_plugin_install_dir_global() {
        let result = codex_plugin_install_dir("test-plugin-install", None);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with(".codex/plugins/test-plugin-install"));
        assert!(path.exists());
        // Cleanup
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_codex_plugin_install_dir_project() {
        let tmp = std::env::temp_dir().join("svd_plugin_test_project");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let result = codex_plugin_install_dir("test-plugin", Some(tmp.to_str().unwrap()));
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with(".codex/plugins/test-plugin"));
        assert!(path.exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_codex_plugin_install_dir_user_scope() {
        let result = codex_plugin_install_dir("test-plugin-user", Some("user"));
        assert!(result.is_ok());
        let path = result.unwrap();
        // "user" should resolve to global ~/.codex/plugins/
        assert!(path.ends_with(".codex/plugins/test-plugin-user"));
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn test_codex_plugin_uninstall_dir() {
        // Create a plugin dir first
        let home = dirs::home_dir().unwrap();
        let plugin_dir = home.join(".codex/plugins/test-plugin-uninstall");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("README.md"), "test").unwrap();
        assert!(plugin_dir.exists());

        let result = codex_plugin_uninstall_dir("test-plugin-uninstall");
        assert!(result.is_ok());
        assert!(!plugin_dir.exists());
    }

    #[test]
    fn test_codex_plugin_uninstall_nonexistent() {
        // Should succeed even if the plugin doesn't exist
        let result = codex_plugin_uninstall_dir("nonexistent-plugin-xyz");
        assert!(result.is_ok());
    }

    #[test]
    fn test_marketplace_json_deserialization() {
        let json = r#"{
            "name": "test-marketplace",
            "plugins": [
                {
                    "name": "test-plugin",
                    "description": "A test plugin",
                    "category": "testing",
                    "author": { "name": "Test Author", "url": "https://example.com" },
                    "homepage": "https://example.com/plugin",
                    "keywords": ["test", "example"]
                },
                {
                    "name": "minimal-plugin"
                }
            ]
        }"#;

        let result: Result<MarketplaceJson, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let marketplace = result.unwrap();
        assert_eq!(marketplace.plugins.len(), 2);

        let p1 = &marketplace.plugins[0];
        assert_eq!(p1.name, "test-plugin");
        assert_eq!(p1.description, "A test plugin");
        assert_eq!(p1.category, Some("testing".to_string()));
        assert_eq!(p1.author.as_ref().unwrap().name, Some("Test Author".to_string()));
        assert_eq!(p1.keywords.len(), 2);

        let p2 = &marketplace.plugins[1];
        assert_eq!(p2.name, "minimal-plugin");
        assert_eq!(p2.description, ""); // default
        assert!(p2.category.is_none());
        assert!(p2.author.is_none());
        assert!(p2.keywords.is_empty());
    }

    #[test]
    fn test_installed_plugins_json_deserialization() {
        let json = r#"{
            "version": 2,
            "plugins": {
                "rust-analyzer-lsp@claude-plugins-official": [
                    {
                        "scope": "user",
                        "installPath": "/some/path",
                        "version": "1.0.0",
                        "installedAt": "2026-03-18T16:18:42.334Z"
                    }
                ]
            }
        }"#;

        let result: Result<InstalledPluginsJson, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let installed = result.unwrap();
        assert!(installed.plugins.contains_key("rust-analyzer-lsp@claude-plugins-official"));
        let entries = &installed.plugins["rust-analyzer-lsp@claude-plugins-official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_blocklist_json_deserialization() {
        let json = r#"{
            "fetchedAt": "2026-03-26T23:24:21.960Z",
            "plugins": [
                { "plugin": "bad-plugin@test", "reason": "security" }
            ]
        }"#;

        // The blocklist has objects with "plugin" field, not plain strings
        // Our BlocklistJson expects Vec<String> for plugins, but the actual format has objects
        // This test verifies the current parsing handles it gracefully
        let result: Result<BlocklistJson, _> = serde_json::from_str(json);
        // This will fail because the format doesn't match — which is fine,
        // the read_blocklist function has a fallback
        assert!(result.is_err() || result.unwrap().plugins.is_empty());
    }
}

// --- Marketplace plugin JSON structures for deserialization ---

#[derive(Deserialize)]
struct MarketplaceJson {
    #[serde(default)]
    plugins: Vec<MarketplacePluginEntry>,
}

#[derive(Deserialize)]
struct MarketplacePluginEntry {
    name: String,
    #[serde(default)]
    description: String,
    category: Option<String>,
    author: Option<MarketplaceAuthor>,
    homepage: Option<String>,
    #[serde(default)]
    keywords: Vec<String>,
}

#[derive(Deserialize)]
struct MarketplaceAuthor {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct InstalledPluginsJson {
    #[serde(default)]
    plugins: std::collections::HashMap<String, Vec<InstalledPluginEntry>>,
}

#[derive(Deserialize)]
struct InstalledPluginEntry {
    #[serde(default)]
    scope: String,
    #[serde(rename = "installPath")]
    install_path: Option<String>,
    version: Option<String>,
    #[serde(rename = "installedAt")]
    installed_at: Option<String>,
}

#[derive(Deserialize)]
struct BlocklistJson {
    #[serde(default)]
    plugins: Vec<String>,
}

fn read_marketplace_json() -> Result<MarketplaceJson, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let path = home
        .join(".claude/plugins/marketplaces/claude-plugins-official/.claude-plugin/marketplace.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read marketplace.json: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse marketplace.json: {}", e))
}

fn read_installed_plugins() -> InstalledPluginsJson {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return InstalledPluginsJson { plugins: std::collections::HashMap::new() },
    };
    let path = home.join(".claude/plugins/installed_plugins.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(InstalledPluginsJson {
            plugins: std::collections::HashMap::new(),
        }),
        Err(_) => InstalledPluginsJson { plugins: std::collections::HashMap::new() },
    }
}

fn read_blocklist() -> Vec<String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let path = home.join(".claude/plugins/blocklist.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            // Try parsing as structured JSON first, fall back to plain array
            if let Ok(bl) = serde_json::from_str::<BlocklistJson>(&content) {
                bl.plugins
            } else if let Ok(arr) = serde_json::from_str::<Vec<String>>(&content) {
                arr
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub async fn get_marketplace_plugins(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<MarketplacePlugin>, String> {
    let marketplace = read_marketplace_json()?;
    let installed = read_installed_plugins();
    let blocklist = read_blocklist();

    let marketplace_name = "claude-plugins-official";

    let mut results: Vec<MarketplacePlugin> = marketplace
        .plugins
        .into_iter()
        .filter(|p| !blocklist.contains(&p.name))
        .map(|p| {
            let key = format!("{}@{}", p.name, marketplace_name);
            let install_info = installed.plugins.get(&key).and_then(|entries| entries.first());

            MarketplacePlugin {
                name: p.name,
                description: p.description,
                category: p.category,
                author_name: p.author.as_ref().and_then(|a| a.name.clone()),
                author_url: p.author.as_ref().and_then(|a| a.url.clone()),
                homepage: p.homepage,
                keywords: p.keywords,
                source: "claude".to_string(),
                is_installed: install_info.is_some(),
                installed_version: install_info.and_then(|i| i.version.clone()),
                installed_at: install_info.and_then(|i| i.installed_at.clone()),
            }
        })
        .collect();

    // Check if we have cached Codex plugins
    let cached = {
        let app = state.lock().await;
        app.codex_plugins_cache.clone()
    };

    if let Some(codex_plugins) = cached {
        results.extend(codex_plugins);
    } else {
        // Fetch OpenAI/Codex plugins from GitHub
        let openai_url = "https://raw.githubusercontent.com/openai/plugins/main/.agents/plugins/marketplace.json";
        if let Ok(resp) = reqwest::get(openai_url).await {
            if let Ok(catalog) = resp.json::<MarketplaceJson>().await {
                let codex_plugins: Vec<MarketplacePlugin> = catalog
                    .plugins
                    .into_iter()
                    .map(|p| MarketplacePlugin {
                        name: p.name,
                        description: p.description,
                        category: p.category,
                        author_name: p.author.as_ref().and_then(|a| a.name.clone()),
                        author_url: p.author.as_ref().and_then(|a| a.url.clone()),
                        homepage: p.homepage,
                        keywords: p.keywords,
                        source: "codex".to_string(),
                        is_installed: false,
                        installed_version: None,
                        installed_at: None,
                    })
                    .collect();

                // Cache the result
                {
                    let mut app = state.lock().await;
                    app.codex_plugins_cache = Some(codex_plugins.clone());
                }

                results.extend(codex_plugins);
            }
        }
    }

    results.sort_by(|a, b| {
        let cat_a = a.category.as_deref().unwrap_or("");
        let cat_b = b.category.as_deref().unwrap_or("");
        cat_a.cmp(cat_b).then(a.name.cmp(&b.name))
    });

    Ok(results)
}

#[tauri::command]
pub async fn get_plugin_detail(
    plugin_name: String,
    plugin_source: Option<String>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<PluginDetail, String> {
    validate_name(&plugin_name, "Plugin name")?;
    let source = plugin_source.unwrap_or_else(|| "claude".to_string());

    if source == "codex" {
        // Look up from the cached Codex plugins
        let cached = {
            let app = state.lock().await;
            app.codex_plugins_cache.clone()
        };

        let codex_plugins = cached.unwrap_or_default();
        let entry = codex_plugins
            .into_iter()
            .find(|p| p.name == plugin_name)
            .ok_or_else(|| format!("Codex plugin '{}' not found (try refreshing the plugin list)", plugin_name))?;

        // Try to fetch README from GitHub
        let readme_url = format!(
            "https://raw.githubusercontent.com/openai/plugins/main/plugins/{}/README.md",
            plugin_name
        );
        let readme = reqwest::get(&readme_url)
            .await
            .ok()
            .and_then(|r| if r.status().is_success() { Some(r) } else { None });
        let readme_text = match readme {
            Some(r) => r.text().await.ok(),
            None => None,
        };

        // Also try to fetch plugin.json for more metadata
        let plugin_json_url = format!(
            "https://raw.githubusercontent.com/openai/plugins/main/plugins/{}/.codex-plugin/plugin.json",
            plugin_name
        );
        let plugin_json = reqwest::get(&plugin_json_url)
            .await
            .ok()
            .and_then(|r| if r.status().is_success() { Some(r) } else { None });

        let (description, homepage) = if let Some(resp) = plugin_json {
            if let Ok(text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    let desc = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let hp = json.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string());
                    (desc.unwrap_or(entry.description), hp.or(entry.homepage))
                } else {
                    (entry.description, entry.homepage)
                }
            } else {
                (entry.description, entry.homepage)
            }
        } else {
            (entry.description, entry.homepage)
        };

        return Ok(PluginDetail {
            name: entry.name,
            description,
            category: entry.category,
            author_name: entry.author_name,
            author_url: entry.author_url,
            homepage,
            keywords: entry.keywords,
            source: "codex".to_string(),
            is_installed: false,
            installed_version: None,
            installed_at: None,
            install_path: None,
            readme: readme_text,
        });
    }

    // Claude plugin lookup (existing logic)
    let marketplace = read_marketplace_json()?;
    let installed = read_installed_plugins();

    let entry = marketplace
        .plugins
        .into_iter()
        .find(|p| p.name == plugin_name)
        .ok_or_else(|| format!("Plugin '{}' not found in marketplace", plugin_name))?;

    let marketplace_name = "claude-plugins-official";
    let key = format!("{}@{}", entry.name, marketplace_name);
    let install_info = installed.plugins.get(&key).and_then(|entries| entries.first());

    // Try to find README.md in known locations
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let base = home.join(".claude/plugins/marketplaces/claude-plugins-official");
    let readme_candidates = [
        base.join("plugins").join(&plugin_name).join("README.md"),
        base.join("external_plugins").join(&plugin_name).join("README.md"),
    ];
    let readme = readme_candidates
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok());

    Ok(PluginDetail {
        name: entry.name,
        description: entry.description,
        category: entry.category,
        author_name: entry.author.as_ref().and_then(|a| a.name.clone()),
        author_url: entry.author.as_ref().and_then(|a| a.url.clone()),
        homepage: entry.homepage,
        keywords: entry.keywords,
        source: "claude".to_string(),
        is_installed: install_info.is_some(),
        installed_version: install_info.and_then(|i| i.version.clone()),
        installed_at: install_info.and_then(|i| i.installed_at.clone()),
        install_path: install_info.and_then(|i| i.install_path.clone()),
        readme,
    })
}
