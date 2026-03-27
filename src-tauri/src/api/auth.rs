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

    // Tests use the Config struct directly to avoid writing to the real ~/.skillvault/config.json

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config { token: Some("svt_test123".to_string()) };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.token, Some("svt_test123".to_string()));
    }

    #[test]
    fn test_config_default_has_no_token() {
        let config = Config::default();
        assert!(config.token.is_none());
    }

    #[test]
    fn test_config_deserialize_empty_json() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert!(config.token.is_none());
    }

    #[test]
    fn test_config_deserialize_null_token() {
        let config: Config = serde_json::from_str(r#"{"token": null}"#).unwrap();
        assert!(config.token.is_none());
    }

    #[test]
    fn test_config_deserialize_with_token() {
        let config: Config = serde_json::from_str(r#"{"token": "svt_abc123"}"#).unwrap();
        assert_eq!(config.token, Some("svt_abc123".to_string()));
    }
}
