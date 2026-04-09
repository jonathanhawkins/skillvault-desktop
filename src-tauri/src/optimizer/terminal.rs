use crate::state::{DetectedTerminal, OptimizationProfile};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Find the next available tmux session name (base, base-1, base-2, ...)
fn next_tmux_session_name(base: &str) -> String {
    // List existing tmux sessions
    let existing = Command::new("tmux")
        .arg("list-sessions")
        .arg("-F")
        .arg("#{session_name}")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let sessions: Vec<&str> = existing.lines().collect();

    if !sessions.contains(&base) {
        return base.to_string();
    }

    for i in 1..100 {
        let candidate = format!("{}-{}", base, i);
        if !sessions.iter().any(|s| *s == candidate.as_str()) {
            return candidate;
        }
    }

    format!("{}-{}", base, std::process::id())
}

/// Detect installed terminal emulators on macOS
pub fn detect_terminals() -> Vec<DetectedTerminal> {
    let mut terminals = Vec::new();

    // Check /Applications/ for known terminal .app bundles
    let checks = vec![
        ("Ghostty", "/Applications/Ghostty.app", "ghostty"),
        ("Warp", "/Applications/Warp.app", "warp"),
        ("iTerm2", "/Applications/iTerm.app", "iterm2"),
        ("Kitty", "/Applications/kitty.app", "kitty"),
        ("Alacritty", "/Applications/Alacritty.app", "alacritty"),
        ("WezTerm", "/Applications/WezTerm.app", "wezterm"),
        ("Hyper", "/Applications/Hyper.app", "hyper"),
    ];

    for (name, app_path, icon_name) in &checks {
        if Path::new(app_path).exists() {
            terminals.push(DetectedTerminal {
                name: name.to_string(),
                app_path: app_path.to_string(),
                icon_name: icon_name.to_string(),
            });
        }
    }

    // Always include Terminal.app
    terminals.push(DetectedTerminal {
        name: "Terminal".to_string(),
        app_path: "/System/Applications/Utilities/Terminal.app".to_string(),
        icon_name: "terminal".to_string(),
    });

    // Also check for CLI-installed terminals via `which`
    let cli_checks = vec![
        ("ghostty", "Ghostty"),
        ("kitty", "Kitty"),
        ("alacritty", "Alacritty"),
        ("wezterm", "WezTerm"),
    ];

    for (binary, name) in &cli_checks {
        // Skip if already detected via .app
        if terminals.iter().any(|t| t.name == *name) {
            continue;
        }
        if let Ok(output) = Command::new("which").arg(binary).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                terminals.push(DetectedTerminal {
                    name: name.to_string(),
                    app_path: path,
                    icon_name: binary.to_string(),
                });
            }
        }
    }

    // Sort: put user's current terminal first based on TERM_PROGRAM
    let current_term = std::env::var("TERM_PROGRAM").unwrap_or_default();
    terminals.sort_by(|a, b| {
        let a_current = a.name.to_lowercase().contains(&current_term.to_lowercase());
        let b_current = b.name.to_lowercase().contains(&current_term.to_lowercase());
        b_current.cmp(&a_current).then(a.name.cmp(&b.name))
    });

    terminals
}

/// Launch a terminal with Claude Code in the given project directory
pub fn launch_terminal(
    terminal_name: &str,
    project_path: &str,
    profile: &OptimizationProfile,
) -> Result<String, String> {
    let mut env_inline = super::profile::build_env_inline(profile);

    // Auto-set task list ID from project dir name when agent teams is on
    if profile.experimental_agent_teams && profile.task_list_id.is_empty() {
        if let Some(name) = std::path::Path::new(project_path).file_name() {
            env_inline = format!("CLAUDE_CODE_TASK_LIST_ID={} {}", name.to_string_lossy(), env_inline);
        }
    }

    // Build claude command with CLI args
    let mut claude_cmd = "claude".to_string();
    if profile.skip_permissions && !profile.extra_cli_args.contains("--dangerously-skip-permissions") {
        claude_cmd.push_str(" --dangerously-skip-permissions");
    }
    if profile.experimental_agent_teams && !profile.extra_cli_args.contains("--teammate-mode") {
        claude_cmd.push_str(" --teammate-mode tmux");
    }
    if !profile.extra_cli_args.is_empty() {
        claude_cmd.push(' ');
        claude_cmd.push_str(profile.extra_cli_args.trim());
    }

    // Wrap in tmux if enabled
    if profile.use_tmux {
        let base_name = std::path::Path::new(project_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "claude".to_string());
        let session_name = next_tmux_session_name(&base_name);
        claude_cmd = format!(
            "tmux new-session -A -s {} '{} {}'",
            session_name, env_inline, claude_cmd
        );
        // env vars are inside tmux command now, clear them from outer shell
        env_inline = String::new();
    }

    let env_exports = if env_inline.is_empty() {
        String::new()
    } else {
        format!("export {}", env_inline.replace(' ', "; export "))
    };

    match terminal_name {
        "Terminal" => launch_terminal_app(project_path, &env_exports, &claude_cmd),
        "iTerm2" => launch_iterm2(project_path, &env_exports, &claude_cmd),
        "Ghostty" => launch_ghostty(project_path, &env_inline, &claude_cmd),
        "Kitty" => launch_kitty(project_path, &env_inline, &claude_cmd),
        "Alacritty" => launch_alacritty(project_path, &env_inline, &claude_cmd),
        "WezTerm" => launch_wezterm(project_path, &env_inline, &claude_cmd),
        "Warp" => launch_warp(project_path, &env_inline, &claude_cmd),
        "Hyper" => launch_hyper(project_path, &env_inline, &claude_cmd),
        _ => Err(format!("Unsupported terminal: {}", terminal_name)),
    }
}

fn launch_terminal_app(project_path: &str, env_exports: &str, claude_cmd: &str) -> Result<String, String> {
    let escaped_path = project_path.replace('\'', "'\\''");
    let cmd = if env_exports.is_empty() {
        format!("cd '{}' && {}", escaped_path, claude_cmd)
    } else {
        format!("cd '{}' && {} && {}", escaped_path, env_exports, claude_cmd)
    };
    let script = format!(
        "tell application \"Terminal\"\n  activate\n  do script \"{}\"\nend tell",
        cmd.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to launch Terminal: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Terminal.app AppleScript failed: {}", stderr.trim()));
    }

    Ok("Launched Claude Code in Terminal".to_string())
}

fn launch_iterm2(project_path: &str, env_exports: &str, claude_cmd: &str) -> Result<String, String> {
    let escaped_path = project_path.replace('\'', "'\\''");
    let cmd = if env_exports.is_empty() {
        format!("cd '{}' && {}", escaped_path, claude_cmd)
    } else {
        format!("cd '{}' && {} && {}", escaped_path, env_exports, claude_cmd)
    };
    let script = format!(
        r#"tell application "iTerm2"
    activate
    set newWindow to (create window with default profile)
    tell current session of newWindow
        write text "{}"
    end tell
end tell"#,
        cmd.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to launch iTerm2: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("iTerm2 AppleScript failed: {}", stderr.trim()));
    }

    Ok("Launched Claude Code in iTerm2".to_string())
}

fn launch_ghostty(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    // Try CLI binary first
    if let Ok(output) = Command::new("which").arg("ghostty").output() {
        if output.status.success() {
            let ghostty = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let cmd = format!("cd '{}' && {} {}", project_path.replace('\'', "'\\''"), env_inline, claude_cmd);
            Command::new(&ghostty)
                .arg("-e")
                .arg("bash")
                .arg("-c")
                .arg(&cmd)
                .spawn()
                .map_err(|e| format!("Failed to launch Ghostty: {}", e))?;
            return Ok("Launched Claude Code in Ghostty".to_string());
        }
    }

    // Fallback: open app + copy command to clipboard
    Command::new("open")
        .arg("-a")
        .arg("Ghostty")
        .spawn()
        .map_err(|e| format!("Failed to launch Ghostty: {}", e))?;

    let escaped_path = project_path.replace('\'', "'\\''");
    let full_cmd = format!("cd '{}' && {} {}", escaped_path, env_inline, claude_cmd);
    set_clipboard(&full_cmd)?;

    Ok("Ghostty opened. Command copied to clipboard — paste to launch.".to_string())
}

fn launch_kitty(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    let cmd = format!("{} {}", env_inline, claude_cmd);
    Command::new("kitty")
        .arg("--directory")
        .arg(project_path)
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&cmd)
        .spawn()
        .map_err(|e| format!("Failed to launch Kitty: {}", e))?;

    Ok("Launched Claude Code in Kitty".to_string())
}

fn launch_alacritty(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    let cmd = format!("{} {}", env_inline, claude_cmd);
    Command::new("alacritty")
        .arg("--working-directory")
        .arg(project_path)
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&cmd)
        .spawn()
        .map_err(|e| format!("Failed to launch Alacritty: {}", e))?;

    Ok("Launched Claude Code in Alacritty".to_string())
}

fn launch_wezterm(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    let cmd = format!("{} {}", env_inline, claude_cmd);
    Command::new("wezterm")
        .arg("start")
        .arg("--cwd")
        .arg(project_path)
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg(&cmd)
        .spawn()
        .map_err(|e| format!("Failed to launch WezTerm: {}", e))?;

    Ok("Launched Claude Code in WezTerm".to_string())
}

fn launch_warp(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    // Warp has limited scripting — open directory + copy command to clipboard
    Command::new("open")
        .arg("-a")
        .arg("Warp")
        .arg(project_path)
        .spawn()
        .map_err(|e| format!("Failed to launch Warp: {}", e))?;

    let full_cmd = format!("{} {}", env_inline, claude_cmd);
    set_clipboard(&full_cmd)?;

    Ok("Warp opened. Command copied to clipboard — paste to launch.".to_string())
}

fn launch_hyper(project_path: &str, env_inline: &str, claude_cmd: &str) -> Result<String, String> {
    // Hyper has limited scripting — open app + copy command to clipboard
    Command::new("open")
        .arg("-a")
        .arg("Hyper")
        .spawn()
        .map_err(|e| format!("Failed to launch Hyper: {}", e))?;

    let escaped_path = project_path.replace('\'', "'\\''");
    let full_cmd = format!("cd '{}' && {} {}", escaped_path, env_inline, claude_cmd);
    set_clipboard(&full_cmd)?;

    Ok("Hyper opened. Command copied to clipboard — paste to launch.".to_string())
}

fn set_clipboard(text: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
    }
    child
        .wait()
        .map_err(|e| format!("Clipboard error: {}", e))?;

    Ok(())
}
