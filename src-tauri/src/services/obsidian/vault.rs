use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::schema::{CategoryStats, MemoryNote, VaultInfo, CATEGORIES};

pub struct ObsidianVault {
    app_data_dir: PathBuf,
    vault_path: RwLock<Option<PathBuf>>,
}

impl ObsidianVault {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let saved_file = app_data_dir.join("vault_path.txt");

        // 1. Try current app data dir
        let mut initial_path = if saved_file.exists() {
            if let Ok(content) = fs::read_to_string(&saved_file) {
                let p = PathBuf::from(content.trim().trim_matches('"'));
                if p.exists() && p.is_dir() {
                    Some(p)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 2. Try legacy com.buddy.app vault_path.txt
        if initial_path.is_none() {
            let legacy_path = dirs::data_dir()
                .map(|d| d.join("com.buddy.app").join("vault_path.txt"))
                .or_else(|| dirs::home_dir().map(|h| h.join("AppData").join("Roaming").join("com.buddy.app").join("vault_path.txt")));

            if let Some(legacy) = legacy_path {
                if legacy.exists() {
                    if let Ok(content) = fs::read_to_string(&legacy) {
                        let p = PathBuf::from(content.trim().trim_matches('"'));
                        if p.exists() && p.is_dir() {
                            let _ = fs::create_dir_all(&app_data_dir);
                            let _ = fs::write(&saved_file, content.trim().as_bytes());
                            initial_path = Some(p);
                        }
                    }
                }
            }
        }

        // 3. Try default "Future Heroes/ALIEN's Mind" in Documents
        if initial_path.is_none() {
            if let Some(doc_dir) = dirs::document_dir() {
                let default_vault = doc_dir.join("Future Heroes").join("ALIEN's Mind");
                if default_vault.exists() && default_vault.is_dir() {
                    let _ = fs::create_dir_all(&app_data_dir);
                    let _ = fs::write(&saved_file, default_vault.to_string_lossy().as_bytes());
                    initial_path = Some(default_vault);
                }
            }
        }

        // Ensure category directories exist
        if let Some(ref p) = initial_path {
            for cat in CATEGORIES {
                let _ = fs::create_dir_all(p.join(cat));
            }
        }

        Self {
            app_data_dir,
            vault_path: RwLock::new(initial_path),
        }
    }

    pub fn set_path(&self, raw_path: PathBuf) -> Result<VaultInfo, String> {
        let clean_str = raw_path
            .to_string_lossy()
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();

        if clean_str.is_empty() {
            return Err("Path cannot be empty.".into());
        }

        let path = PathBuf::from(&clean_str);
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(&path) {
                return Err(format!("Folder does not exist and could not be created: {e}"));
            }
        }

        if !path.is_dir() {
            return Err("The selected path is a file, not a directory.".into());
        }

        // Initialize category folders if missing
        for cat in CATEGORIES {
            let cat_dir = path.join(cat);
            let _ = fs::create_dir_all(&cat_dir);
        }

        // Persist the vault path to disk
        let _ = fs::create_dir_all(&self.app_data_dir);
        let save_target = self.app_data_dir.join("vault_path.txt");
        let _ = fs::write(&save_target, path.to_string_lossy().as_bytes());

        let mut lock = self.vault_path.write().map_err(|e| e.to_string())?;
        *lock = Some(path);

        drop(lock);
        self.get_info()
    }

    fn count_all_md_files(dir: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let file_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if file_name.starts_with('.') {
                    continue;
                }
                if p.is_dir() {
                    count += Self::count_all_md_files(&p);
                } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn get_info(&self) -> Result<VaultInfo, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        match &*lock {
            Some(path) => {
                let total_notes = Self::count_all_md_files(path);
                let mut stats = Vec::new();

                for cat in CATEGORIES {
                    let cat_dir = path.join(cat);
                    let mut count = 0;
                    if let Ok(entries) = fs::read_dir(cat_dir) {
                        for e in entries.flatten() {
                            if e.path().extension().and_then(|s| s.to_str()) == Some("md") {
                                count += 1;
                            }
                        }
                    }
                    stats.push(CategoryStats {
                        name: cat.to_string(),
                        count,
                    });
                }

                Ok(VaultInfo {
                    path: path.to_string_lossy().to_string(),
                    is_connected: true,
                    note_count: total_notes,
                    categories: stats,
                })
            }
            None => Ok(VaultInfo {
                path: String::new(),
                is_connected: false,
                note_count: 0,
                categories: Vec::new(),
            }),
        }
    }

    /// Create a verified test note to validate live Obsidian connection
    pub fn create_test_note(&self) -> Result<String, String> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let content = format!(
            "# 🟢 ALIEN Vault Connection Verified\n\n- **Timestamp**: {}\n- **Status**: Live connection confirmed\n- **Vault Link**: [[Conversational Knowledge]]\n- **Daily Log**: [[Daily/{}]]\n\nALIEN is successfully connected to this Obsidian vault and will continuously record insights.",
            now,
            chrono::Utc::now().format("%Y-%m-%d")
        );
        self.write_note("Daily", "ALIEN Connection Verified", &content)
    }

    /// Open vault folder in Windows Explorer
    pub fn open_in_explorer(&self) -> Result<(), String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        if let Some(path) = &*lock {
            if cfg!(target_os = "windows") {
                let _ = std::process::Command::new("explorer")
                    .arg(path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        } else {
            Err("No Obsidian vault connected.".into())
        }
    }

    pub fn write_note(&self, category: &str, title: &str, content: &str) -> Result<String, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        let root = lock.as_ref().ok_or("No Obsidian vault connected.")?;

        let safe_title = title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let file_name = if safe_title.ends_with(".md") {
            safe_title
        } else {
            format!("{safe_title}.md")
        };

        let cat_dir = root.join(category);
        if !cat_dir.exists() {
            fs::create_dir_all(&cat_dir).map_err(|e| e.to_string())?;
        }

        let file_path = cat_dir.join(&file_name);

        // Non-destructive update: If file exists, append timestamped update
        let final_content = if file_path.exists() {
            let existing = fs::read_to_string(&file_path).unwrap_or_default();
            format!(
                "{}\n\n---\n### Update: {}\n{}",
                existing.trim(),
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
                content
            )
        } else {
            format!(
                "# {}\nCreated: {}\n\n{}",
                title,
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
                content
            )
        };

        fs::write(&file_path, final_content).map_err(|e| e.to_string())?;
        Ok(format!("Saved memory note: [[{}]] in {}", title, category))
    }

    /// Automatically records learnings, insights and daily logs from user interactions
    pub fn record_interaction(&self, user_prompt: &str, assistant_reply: &str) {
        let lock = match self.vault_path.read() {
            Ok(l) => l,
            Err(_) => return,
        };

        if lock.is_none() {
            return;
        }
        drop(lock);

        let now = chrono::Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M UTC").to_string();

        // 1. Log to Daily journal
        let clean_user = user_prompt.trim().replace('\n', " ");
        let clean_ai = assistant_reply.trim().lines().take(3).collect::<Vec<&str>>().join(" ");
        let daily_entry = format!(
            "- **{}**\n  - **User**: {}\n  - **ALIEN**: {}\n",
            time_str, clean_user, clean_ai
        );
        let _ = self.write_note("Daily", &date_str, &daily_entry);

        // 2. Extract preferences or insights
        let lower = user_prompt.to_lowercase();
        if lower.contains("i like")
            || lower.contains("i prefer")
            || lower.contains("my favorite")
            || lower.contains("remember that")
            || lower.contains("don't")
            || lower.contains("never")
            || lower.contains("always")
        {
            let pref_entry = format!(
                "- **User Preference ({})**: {}\n  - Context: [[Daily/{}]]\n",
                time_str, user_prompt.trim(), date_str
            );
            let _ = self.write_note("Preferences", "User Preferences", &pref_entry);
        }

        // 3. Extract projects or topics
        if lower.contains("project")
            || lower.contains("build")
            || lower.contains("develop")
            || lower.contains("alien")
            || lower.contains("app")
            || lower.contains("music")
            || lower.contains("art")
        {
            let project_entry = format!(
                "- **Work Log ({})**: {}\n  - Output: {}\n",
                time_str, clean_user, clean_ai
            );
            let _ = self.write_note("Projects", "Active Projects", &project_entry);
        }

        // 4. Log to Learned/Knowledge
        if assistant_reply.len() > 100 {
            let learned_entry = format!(
                "- **Insight ({})**: Discussed {}\n  - Core takeaway: {}\n",
                time_str, clean_user, clean_ai
            );
            let _ = self.write_note("Learned", "Conversational Knowledge", &learned_entry);
        }

        // 5. Extract user teachings / instructions / procedural rules
        if lower.contains("when i")
            || lower.contains("whenever i")
            || lower.contains("teach you")
            || lower.contains("how i want you to")
            || lower.contains("i want you to always")
            || lower.contains("rule for")
            || lower.contains("instruction")
            || lower.contains("workflow")
            || lower.contains("in blender")
            || lower.contains("in discord")
            || lower.contains("in clip studio")
        {
            let safe_title = user_prompt
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ')
                .take(35)
                .collect::<String>()
                .trim()
                .to_string();
            let title = if safe_title.is_empty() { "Custom Workflow".into() } else { safe_title };
            let instr_entry = format!(
                "## Workflow: {}\n- **Taught**: {}\n- **Context**: [[Daily/{}]]\n- **Requirement**:\n{}\n\n- **ALIEN Execution Strategy**:\n{}\n",
                title, time_str, date_str, user_prompt.trim(), assistant_reply.trim()
            );
            let _ = self.write_note("Instructions", &title, &instr_entry);
        }
    }

    /// Cleanly rewrite or update a note in the vault, replacing outdated facts with modernized structure
    pub fn rewrite_note(&self, category: &str, title: &str, content: &str) -> Result<String, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        let root = lock.as_ref().ok_or("No Obsidian vault connected.")?;

        let safe_title = title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let file_name = if safe_title.ends_with(".md") {
            safe_title
        } else {
            format!("{safe_title}.md")
        };

        let cat_dir = root.join(category);
        if !cat_dir.exists() {
            fs::create_dir_all(&cat_dir).map_err(|e| e.to_string())?;
        }

        let file_path = cat_dir.join(&file_name);
        let header = format!(
            "# {}\nUpdated: {}\n\n",
            title,
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );
        let full_text = format!("{header}{}", content.trim());

        fs::write(&file_path, full_text).map_err(|e| e.to_string())?;
        Ok(format!("Optimized & modernized note: [[{}]] in {}", title, category))
    }

    /// List all markdown notes across the vault
    pub fn list_all_notes(&self) -> Result<Vec<MemoryNote>, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        let root = match &*lock {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        for cat in CATEGORIES {
            let cat_dir = root.join(cat);
            if let Ok(entries) = fs::read_dir(cat_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("md") {
                        let title = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        let content = fs::read_to_string(&p).unwrap_or_default();
                        results.push(MemoryNote {
                            category: cat.to_string(),
                            title,
                            content,
                            path: p.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn read_note(&self, category: &str, title: &str) -> Result<String, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        let root = lock.as_ref().ok_or("No Obsidian vault connected.")?;

        let safe_title = title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let file_name = if safe_title.ends_with(".md") {
            safe_title
        } else {
            format!("{safe_title}.md")
        };

        let file_path = root.join(category).join(file_name);
        fs::read_to_string(&file_path).map_err(|e| format!("Could not read note: {e}"))
    }

    pub fn search(&self, query: &str) -> Result<Vec<MemoryNote>, String> {
        let lock = self.vault_path.read().map_err(|e| e.to_string())?;
        let root = match &*lock {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let q = query.to_lowercase();
        let mut results = Vec::new();

        for cat in CATEGORIES {
            let cat_dir = root.join(cat);
            if let Ok(entries) = fs::read_dir(cat_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Ok(content) = fs::read_to_string(&p) {
                            let title = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            if title.to_lowercase().contains(&q) || content.to_lowercase().contains(&q) {
                                results.push(MemoryNote {
                                    category: cat.to_string(),
                                    title,
                                    content,
                                    path: p.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Retrieve relevant memory snippet for prompt context
    pub fn retrieve_context(&self, user_prompt: &str) -> String {
        let lock = match self.vault_path.read() {
            Ok(l) => l,
            Err(_) => return String::new(),
        };

        let root = match &*lock {
            Some(p) => p,
            None => return String::new(),
        };

        let words: Vec<&str> = user_prompt
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        let mut matches = Vec::new();

        // Always check Profile/User.md if available
        let profile_path = root.join("Profile").join("User.md");
        if let Ok(prof) = fs::read_to_string(profile_path) {
            matches.push(format!("### User Profile\n{}", prof.trim()));
        }

        // Always search Instructions first so taught rules and workflows are respected
        let instr_dir = root.join("Instructions");
        if let Ok(entries) = fs::read_dir(instr_dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("md") {
                    let title = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let matched = words.is_empty()
                        || words.iter().any(|&w| title.to_lowercase().contains(&w.to_lowercase()))
                        || user_prompt.to_lowercase().contains(&title.to_lowercase());
                    if matched {
                        if let Ok(content) = fs::read_to_string(&p) {
                            matches.push(format!("### Taught Workflow [[{}]] (Instructions)\n{}", title, content.trim()));
                        }
                    }
                }
            }
        }

        // Search for mentions in Preferences, People, Projects, Learned, Research
        for cat in &["Preferences", "People", "Projects", "Learned", "Research"] {
            let cat_dir = root.join(cat);
            if let Ok(entries) = fs::read_dir(cat_dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    let title = p.file_stem().unwrap_or_default().to_string_lossy().to_string();

                    let matched = words.iter().any(|&w| title.to_lowercase().contains(&w.to_lowercase()));

                    if matched {
                        if let Ok(content) = fs::read_to_string(&p) {
                            matches.push(format!("### [[{}]] ({})\n{}", title, cat, content.trim()));
                        }
                    }
                }
            }
        }

        if matches.is_empty() {
            String::new()
        } else {
            format!(
                "\n<obsidian_memory>\nRelevant knowledge from user's vault:\n{}\n</obsidian_memory>\n",
                matches.join("\n\n")
            )
        }
    }

    pub const DEFAULT_SYSTEM_IDENTITY: &'static str = r#"# ALIEN Foundation Manual & Operating Directives

## Identity & Persona
- You are ALIEN, an advanced autonomous personal intelligence, digital companion, and creative partner.
- You think deeply, act decisively, and speak with high clarity: concise, sharp, friendly, and straight to the point.
- Creator & Partner: Kobby (kobbyagyemang7@gmail.com).

## Core Creative Domains & Focus
- WebToon & Storytelling: Lead creative architect for WebToon projects including "The SCARLET Crystal", art creation, character development, and narrative world-building.
- Digital Production: Highly proficient in Clip Studio Paint, Blender 3D, Obsidian Knowledge Vault, and Discord workflows.

## Autonomous Operation Directives
- Initiative & Ownership: When asked to research, plan, or execute, use available autonomous tools to achieve the goal thoroughly.
- Knowledge Continuity: Keep the Obsidian Vault updated with learnings, character sheets, research dossiers, and ideas using [[wikilinks]].
- Conversational Agility: In standard chat, keep responses snappy and direct. Avoid unnecessary fluff or robotic disclaimers.
- Safety & Precision: Respect user instructions and preserve all project files and creative work."#;

    pub fn get_system_identity(&self) -> String {
        // 1. Check Obsidian Vault: Instructions/Identity & System Manual.md
        if let Ok(lock) = self.vault_path.read() {
            if let Some(ref path) = *lock {
                let note_path = path.join("Instructions").join("Identity & System Manual.md");
                if note_path.exists() {
                    if let Ok(content) = fs::read_to_string(&note_path) {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }

        // 2. Check app data dir: system_identity.md
        let app_file = self.app_data_dir.join("system_identity.md");
        if app_file.exists() {
            if let Ok(content) = fs::read_to_string(&app_file) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }

        Self::DEFAULT_SYSTEM_IDENTITY.to_string()
    }

    pub fn set_system_identity(&self, content: &str) -> Result<(), String> {
        let clean = content.trim();
        let target_content = if clean.is_empty() {
            Self::DEFAULT_SYSTEM_IDENTITY
        } else {
            clean
        };

        // 1. Save to app data dir
        let _ = fs::create_dir_all(&self.app_data_dir);
        let app_file = self.app_data_dir.join("system_identity.md");
        fs::write(&app_file, target_content).map_err(|e| e.to_string())?;

        // 2. Save to Obsidian Vault under Instructions/Identity & System Manual.md
        if let Ok(lock) = self.vault_path.read() {
            if let Some(ref path) = *lock {
                let inst_dir = path.join("Instructions");
                let _ = fs::create_dir_all(&inst_dir);
                let note_path = inst_dir.join("Identity & System Manual.md");
                let _ = fs::write(&note_path, target_content);
            }
        }

        Ok(())
    }
}
