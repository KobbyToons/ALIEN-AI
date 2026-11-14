// ── Windows Credential Manager Service ──────────────────────────────
//
// Uses the `keyring` crate to securely write and read API keys
// from the native Windows Credential Manager.
// Plaintext keys are never stored on disk.

use keyring::Entry;
use serde_json::Value;

const SERVICE_NAME: &str = "com.alien.desktop";
const LEGACY_SERVICE_NAMES: &[&str] = &["com.alien.app", "com.buddy.app"];

pub struct CredentialService;

impl CredentialService {
    /// Save an API key securely for a specific provider
    pub fn set_key(provider: &str, api_key: &str) -> Result<(), String> {
        let clean = Self::extract_token_if_json(api_key.trim());
        let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
        entry.set_password(&clean).map_err(|e| e.to_string())
    }

    /// Retrieve an API key from Windows Credential Manager
    pub fn get_key(provider: &str) -> Result<String, String> {
        if provider == "gemini" {
            return Self::get_gemini_key();
        }

        if let Ok(entry) = Entry::new(SERVICE_NAME, provider) {
            if let Ok(pwd) = entry.get_password() {
                if !pwd.trim().is_empty() {
                    return Ok(pwd);
                }
            }
        }

        for legacy in LEGACY_SERVICE_NAMES {
            if let Ok(entry) = Entry::new(legacy, provider) {
                if let Ok(pwd) = entry.get_password() {
                    if !pwd.trim().is_empty() {
                        return Ok(pwd);
                    }
                }
            }
        }

        let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
        entry.get_password().map_err(|e| e.to_string())
    }

    /// Specialized getter for Gemini: Checks com.alien.desktop, then legacy, then system Google Account
    pub fn get_gemini_key() -> Result<String, String> {
        // 1. Try com.alien.desktop / gemini for an explicit AI Studio API key (AIza...)
        if let Ok(entry) = Entry::new(SERVICE_NAME, "gemini") {
            if let Ok(pwd) = entry.get_password() {
                let cleaned = Self::extract_token_if_json(pwd.trim());
                if cleaned.starts_with("AIza") {
                    return Ok(cleaned);
                }
            }
        }

        // 1b. Try legacy service names / gemini for an explicit AI Studio key
        for legacy in LEGACY_SERVICE_NAMES {
            if let Ok(entry) = Entry::new(legacy, "gemini") {
                if let Ok(pwd) = entry.get_password() {
                    let cleaned = Self::extract_token_if_json(pwd.trim());
                    if cleaned.starts_with("AIza") {
                        let _ = Self::set_key("gemini", &cleaned);
                        return Ok(cleaned);
                    }
                }
            }
        }

        // 2. Check system Google Account from Antigravity / Gemini CLI in Credential Manager
        if let Ok(entry) = Entry::new("gemini-cli-api-key", "default-api-key") {
            if let Ok(pwd) = entry.get_password() {
                let cleaned = Self::extract_token_if_json(pwd.trim());
                if cleaned.starts_with("AIza") {
                    let _ = Self::set_key("gemini", &cleaned);
                    return Ok(cleaned);
                }
            }
        }

        // 3. Permanent Google Account connection via Antigravity agy engine
        if let Some(email) = Self::get_google_account_email() {
            return Ok(format!("google_account:{email}"));
        }

        if let Some(home) = dirs::home_dir() {
            if home.join(".gemini").join("bin").join("agy.exe").exists() {
                return Ok("google_account_antigravity".into());
            }
        }

        Err("No Gemini API key or connected Google Account found. Please sign in or provide a key.".into())
    }

    /// Remove an API key
    pub fn delete_key(provider: &str) -> Result<(), String> {
        for legacy in LEGACY_SERVICE_NAMES {
            if let Ok(entry) = Entry::new(legacy, provider) {
                let _ = entry.delete_password();
            }
        }
        let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
        match entry.delete_password() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Check if an API key exists without exposing the secret to the frontend
    pub fn has_key(provider: &str) -> bool {
        if provider == "ollama" || provider == "lmstudio" {
            return true;
        }
        if provider == "gemini" {
            return Self::get_google_account_email().is_some() || Self::get_key(provider).is_ok();
        }
        Self::get_key(provider).is_ok()
    }

    /// Detect active Google account email from ~/.gemini/google_accounts.json
    pub fn get_google_account_email() -> Option<String> {
        let home = dirs::home_dir()?;
        let path = home.join(".gemini").join("google_accounts.json");
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&data) {
                if let Some(active) = val.get("active").and_then(|v| v.as_str()) {
                    if !active.is_empty() {
                        return Some(active.to_string());
                    }
                }
            }
        }
        None
    }

    /// Check ~/.gemini/oauth_creds.json
    #[allow(dead_code)]
    fn get_oauth_creds_token() -> Option<String> {
        let home = dirs::home_dir()?;
        let path = home.join(".gemini").join("oauth_creds.json");
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&data) {
                if let Some(tok) = val.get("access_token").and_then(|v| v.as_str()) {
                    if !tok.is_empty() {
                        return Some(tok.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract clean token if string is JSON or formatted
    pub fn extract_token_if_json(raw: &str) -> String {
        let trimmed = raw.trim().trim_matches('"');
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if let Some(tok) = val.pointer("/token/accessToken").and_then(|v| v.as_str()) {
                    return tok.to_string();
                }
                if let Some(tok) = val.pointer("/token/access_token").and_then(|v| v.as_str()) {
                    return tok.to_string();
                }
                if let Some(tok) = val.get("accessToken").and_then(|v| v.as_str()) {
                    return tok.to_string();
                }
                if let Some(tok) = val.get("access_token").and_then(|v| v.as_str()) {
                    return tok.to_string();
                }
                if let Some(tok) = val.get("key").and_then(|v| v.as_str()) {
                    return tok.to_string();
                }
            }
        }
        trimmed.to_string()
    }
}
