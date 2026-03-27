use notify::{Event, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri::Emitter;

pub fn start_file_watcher(app_handle: tauri::AppHandle) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(_event) = res {
            let _ = tx.send(());
        }
    })
    .map_err(|e| format!("Watcher error: {}", e))?;

    let home = dirs::home_dir().ok_or("No home dir")?;
    let claude_dir = home.join(".claude");

    // Watch key directories
    for dir in &["skills", "agents", "plugins"] {
        let path = claude_dir.join(dir);
        if path.exists() {
            let _ = watcher.watch(&path, RecursiveMode::Recursive);
        }
    }

    // Debounce loop — keep watcher alive by holding it in scope
    let _watcher = watcher;
    let mut last_emit = Instant::now();
    loop {
        if rx.recv_timeout(Duration::from_secs(60)).is_ok() {
            // Drain any additional events within debounce window
            while rx.try_recv().is_ok() {}

            if last_emit.elapsed() > Duration::from_millis(500) {
                last_emit = Instant::now();
                let _ = app_handle.emit("local-state-changed", ());
            }
        }
    }
}
