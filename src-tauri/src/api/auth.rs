use std::fs;
use std::path::PathBuf;

/// Get the config directory: ~/.skillvault/
fn config_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let dir = home.join(".skillvault");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    Ok(dir)
}

/// Get the config file path: ~/.skillvault/config.json
fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Config {
    #[serde(default)]
    token: Option<String>,
}

/// Store the API token in ~/.skillvault/config.json
pub fn save_token(token: &str) -> Result<(), String> {
    let path = config_path()?;
    let mut config = read_config();
    config.token = Some(token.to_string());
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))
}

/// Retrieve the API token from ~/.skillvault/config.json
pub fn get_token() -> Option<String> {
    let config = read_config();
    config.token
}

/// Delete the API token from ~/.skillvault/config.json
pub fn delete_token() -> Result<(), String> {
    let path = config_path()?;
    let mut config = read_config();
    config.token = None;
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))
}

fn read_config() -> Config {
    let path = match config_path() {
        Ok(p) => p,
        Err(_) => return Config::default(),
    };
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_get_token() {
        let test_token = "svt_test_token_for_auth_test_12345";
        save_token(test_token).unwrap();
        let retrieved = get_token();
        assert_eq!(retrieved, Some(test_token.to_string()));
        // Cleanup
        delete_token().unwrap();
        assert_eq!(get_token(), None);
    }

    #[test]
    fn test_delete_nonexistent_token() {
        // Should not error even if no token exists
        let result = delete_token();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_token_when_none() {
        // Delete any existing token first
        let _ = delete_token();
        assert_eq!(get_token(), None);
    }
}
