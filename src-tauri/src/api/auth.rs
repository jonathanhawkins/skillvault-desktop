use keyring::Entry;

const SERVICE_NAME: &str = "skillvault-desktop";
const ACCOUNT_NAME: &str = "api-token";

/// Store the API token in the OS keychain
pub fn save_token(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| format!("Keychain error: {}", e))?;
    entry
        .set_password(token)
        .map_err(|e| format!("Failed to save token: {}", e))
}

/// Retrieve the API token from the OS keychain
pub fn get_token() -> Option<String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).ok()?;
    entry.get_password().ok()
}

/// Delete the API token from the OS keychain
pub fn delete_token() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| format!("Keychain error: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete token: {}", e))
}
