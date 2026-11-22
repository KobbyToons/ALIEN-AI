// ── ALIEN Safety Blocklist Engine ── Phase H: Polish & Privacy ───────────────
//
// Protects user privacy and security by preventing ALIEN from capturing or
// injecting input into sensitive applications, such as password managers,
// banking portals, checkout screens, and Windows Security surfaces.

use std::sync::RwLock;

pub struct SafetyBlocklistService {
    enabled: RwLock<bool>,
    blocked_keywords: RwLock<Vec<String>>,
}

impl SafetyBlocklistService {
    pub fn new() -> Self {
        Self {
            enabled: RwLock::new(true),
            blocked_keywords: RwLock::new(vec![
                "1password".into(),
                "bitwarden".into(),
                "keepass".into(),
                "lastpass".into(),
                "dashlane".into(),
                "enpass".into(),
                "roboform".into(),
                "authenticator".into(),
                "banking".into(),
                "credit card".into(),
                "paypal".into(),
                "checkout".into(),
                "windows security".into(),
                "credential manager".into(),
                "task manager".into(),
                "sign in".into(),
                "login".into(),
            ]),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.read().map(|g| *g).unwrap_or(true)
    }

    pub fn set_enabled(&self, val: bool) {
        if let Ok(mut lock) = self.enabled.write() {
            *lock = val;
        }
    }

    pub fn get_keywords(&self) -> Vec<String> {
        self.blocked_keywords.read().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn add_keyword(&self, kw: String) -> Vec<String> {
        let clean = kw.trim().to_lowercase();
        if !clean.is_empty() {
            if let Ok(mut lock) = self.blocked_keywords.write() {
                if !lock.contains(&clean) {
                    lock.push(clean);
                }
            }
        }
        self.get_keywords()
    }

    pub fn remove_keyword(&self, kw: String) -> Vec<String> {
        let clean = kw.trim().to_lowercase();
        if let Ok(mut lock) = self.blocked_keywords.write() {
            lock.retain(|item| item != &clean);
        }
        self.get_keywords()
    }

    /// Check if window title contains any blocked keyword.
    pub fn check_window_safety(&self, window_title: &str) -> Result<(), String> {
        if !self.is_enabled() {
            return Ok(());
        }

        let title_lower = window_title.to_lowercase();
        if let Ok(list) = self.blocked_keywords.read() {
            for kw in list.iter() {
                if title_lower.contains(kw) {
                    return Err(format!(
                        "Sensitive window matched blocked keyword '{}' (Window: \"{}\"). Control paused for privacy.",
                        kw, window_title
                    ));
                }
            }
        }
        Ok(())
    }
}
