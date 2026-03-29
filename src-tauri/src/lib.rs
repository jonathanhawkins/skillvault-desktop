mod api;
mod commands;
mod installer;
mod scanner;
mod state;
mod watcher;

use state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::Mutex;
use watcher::start_file_watcher;

/// Check for skill updates without requiring Tauri managed state.
/// Returns the list of skills that have newer versions available.
async fn check_updates_internal(
    app_state: Arc<Mutex<AppState>>,
) -> Result<Vec<commands::UpdateInfo>, String> {
    let local = {
        let app = app_state.lock().await;
        app.local_state.clone()
    };

    let local = match local {
        Some(l) => l,
        None => return Ok(Vec::new()),
    };

    let client = api::client::ApiClient::new(None);
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
                    updates.push(commands::UpdateInfo {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::scan_local,
            commands::search_packages,
            commands::get_package,
            commands::get_trending,
            commands::get_categories,
            commands::install_package,
            commands::uninstall_skill,
            commands::check_updates,
            commands::set_auth_token,
            commands::get_auth_status,
            commands::clear_auth_token,
            commands::get_platform_stats,
            commands::list_projects,
            commands::get_skill_detail,
            commands::read_file_content,
            commands::get_marketplace_plugins,
            commands::get_plugin_detail,
            commands::package_skill,
            commands::publish_skill,
            commands::package_skills,
            commands::publish_skills,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::update_package,
            commands::delete_package,
            commands::get_my_packages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
