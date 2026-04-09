pub mod profile;
pub mod shell_profile;
pub mod launch_script;
pub mod terminal;

use std::sync::{Mutex, OnceLock};

/// Global mutex to serialize optimizer file operations
static OPTIMIZER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn lock() -> std::sync::MutexGuard<'static, ()> {
    OPTIMIZER_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("optimizer lock poisoned")
}
