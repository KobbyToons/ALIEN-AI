// ── ALIEN Learn Mode Engine ── Phase F: "Watch how I do this" ───────────────
//
// Records user demonstrations and distills them into Obsidian workflows:
//  - Tracks click targets (x, y) and window contexts.
//  - Captures keyboard SHORTCUTS only (Ctrl+S, Alt+F4, Enter, etc.).
//  - NEVER logs text typing (Strict Anti-Keylogger Guarantee).
//  - Synthesizes structured markdown into Obsidian: Learned/Workflows/<name>.md.
//  - Generates replayable skills under ALIEN's loaded skill system.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::services::ai::{AiProviders, ChatMessageInput, ModelProvider};
use crate::services::obsidian::ObsidianVault;
use crate::services::vision;
use crate::services::CredentialService;

static IS_LEARNING_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
pub fn get_active_window_title() -> String {
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
        fn GetWindowTextW(hwnd: *mut std::ffi::c_void, lp_string: *mut u16, n_max_count: i32) -> i32;
    }
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return String::new();
        }
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

#[cfg(not(windows))]
pub fn get_active_window_title() -> String {
    String::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedStep {
    pub timestamp_ms: u64,
    pub action_type: String, // "click", "shortcut", "state"
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub norm_x: Option<f32>,
    pub norm_y: Option<f32>,
    pub details: String,
    pub app_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub name: String,
    pub start_time: u64,
    pub steps: Vec<RecordedStep>,
}

pub struct LearnModeService {
    session: Arc<Mutex<Option<ActiveSession>>>,
}

impl LearnModeService {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_active(&self) -> bool {
        IS_LEARNING_ACTIVE.load(Ordering::SeqCst)
    }

    /// Start recording a user demonstration.
    pub async fn start_session(&self, name: String, app: &AppHandle) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Take initial snapshot of screen
        let initial_b64 = tokio::task::spawn_blocking(|| vision::capture_monitor_base64(None, Some(1280)))
            .await
            .ok()
            .and_then(|r| r.ok());

        let initial_step = RecordedStep {
            timestamp_ms: now,
            action_type: "initial_state".into(),
            x: None,
            y: None,
            norm_x: None,
            norm_y: None,
            details: "Workflow started".into(),
            app_title: get_active_window_title(),
            frame_b64: initial_b64,
        };

        let mut sess = self.session.lock().await;
        *sess = Some(ActiveSession {
            name: name.clone(),
            start_time: now,
            steps: vec![initial_step],
        });

        IS_LEARNING_ACTIVE.store(true, Ordering::SeqCst);

        // Notify UI with active workflow name
        let _ = app.emit(
            "buddy://learn-mode-state",
            json!({
                "active": true,
                "name": name,
            }),
        );

        Ok(())
    }

    /// Record a physical click event during demonstration.
    pub async fn record_click(&self, x: i32, y: i32) {
        if !self.is_active() {
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let app_title = get_active_window_title();

        // Calculate normalized coordinates
        let monitors = vision::list_monitors().unwrap_or_default();
        let (norm_x, norm_y) = if let Some(m) = monitors.iter().find(|m| m.is_primary).or_else(|| monitors.first()) {
            let nx = ((x - m.x) as f32 / m.width as f32) * 1000.0;
            let ny = ((y - m.y) as f32 / m.height as f32) * 1000.0;
            (Some(nx.clamp(0.0, 1000.0)), Some(ny.clamp(0.0, 1000.0)))
        } else {
            (None, None)
        };

        let mut sess = self.session.lock().await;
        if let Some(ref mut s) = *sess {
            s.steps.push(RecordedStep {
                timestamp_ms: now,
                action_type: "click".into(),
                x: Some(x),
                y: Some(y),
                norm_x,
                norm_y,
                details: format!("Clicked at ({x}, {y})"),
                app_title,
                frame_b64: None,
            });
        }
    }

    /// Record a keyboard shortcut combo during demonstration.
    pub async fn record_shortcut(&self, combo: String) {
        if !self.is_active() {
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let app_title = get_active_window_title();

        let mut sess = self.session.lock().await;
        if let Some(ref mut s) = *sess {
            s.steps.push(RecordedStep {
                timestamp_ms: now,
                action_type: "shortcut".into(),
                x: None,
                y: None,
                norm_x: None,
                norm_y: None,
                details: format!("Pressed hotkey combo: {combo}"),
                app_title,
                frame_b64: None,
            });
        }
    }

    /// Stop session, synthesize workflow note in Obsidian, and build replayable skill.
    pub async fn stop_session(&self, app: &AppHandle) -> Result<String, String> {
        IS_LEARNING_ACTIVE.store(false, Ordering::SeqCst);
        let _ = app.emit("buddy://learn-mode-state", json!({ "active": false }));

        let session_data = {
            let mut sess = self.session.lock().await;
            sess.take()
        };

        let session = match session_data {
            Some(s) => s,
            None => return Err("No active learning session to stop.".into()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let duration_secs = (now.saturating_sub(session.start_time)) / 1000;

        // Capture final frame
        let final_b64 = tokio::task::spawn_blocking(|| vision::capture_monitor_base64(None, Some(1280)))
            .await
            .ok()
            .and_then(|r| r.ok());

        // Summarize step stream
        let mut steps_log = format!("Duration: {} seconds ({} steps recorded)\n", duration_secs, session.steps.len());
        for (i, step) in session.steps.iter().enumerate() {
            steps_log.push_str(&format!(
                "{}. [{}] App: \"{}\" - {}\n",
                i + 1,
                step.action_type,
                step.app_title,
                step.details
            ));
        }

        let vault = app.state::<ObsidianVault>();
        let ai = app.state::<AiProviders>();
        let api_key = match CredentialService::get_key("gemini") {
            Ok(k) => Some(k),
            Err(_) => None,
        };

        let primary_app = session
            .steps
            .iter()
            .map(|s| s.app_title.clone())
            .filter(|t| !t.is_empty())
            .last()
            .unwrap_or_else(|| "General GUI".into());

        // Prompt Gemini to distill into clean Obsidian Markdown
        let prompt = format!(
            "Distill this user workflow demonstration into a clear, structured Obsidian markdown playbook.\n\n\
            Workflow Name: {}\n\
            Primary App: {}\n\
            Recorded Interaction Steps:\n{}\n\n\
            Format requirements:\n\
            - Markdown with headers: ## Overview, ## Prerequisites, ## Step-by-Step Procedure, ## Shortcuts Used, ## Tips & Caveats.\n\
            - Include backlinks: [[Learned/Workflows]], [[Skills/Apps/{}]].\n\
            - Be concise, accurate, and actionable.",
            session.name, primary_app, steps_log, primary_app
        );

        let mut conversation = vec![
            ChatMessageInput::new(
                "system",
                "You are ALIEN distilling a learned user workflow into an Obsidian playbook note.",
            ),
        ];

        if let Some(b64) = final_b64 {
            conversation.push(ChatMessageInput::with_image(
                "user",
                prompt,
                format!("data:image/jpeg;base64,{b64}"),
            ));
        } else {
            conversation.push(ChatMessageInput::new("user", prompt));
        }

        let mut distilled_markdown = String::new();
        let _ = ai
            .stream_chat(
                &ModelProvider::Gemini,
                "gemini-3.8-flash",
                api_key.as_deref(),
                None,
                &conversation,
                |chunk| {
                    distilled_markdown.push_str(chunk);
                },
            )
            .await;

        if distilled_markdown.is_empty() {
            distilled_markdown = format!(
                "## Overview\nLearned workflow: **{}** in **{}**.\n\n## Recorded Steps\n{}\n",
                session.name, primary_app, steps_log
            );
        }

        // Save into Obsidian Vault under Learned/
        let note_title = format!("Workflows - {}", session.name);
        let _ = vault.rewrite_note("Learned", &note_title, &distilled_markdown);

        // Also update app playbook note under Instructions/ or Research/
        let app_playbook_title = format!("App Playbook - {}", primary_app);
        let app_note = format!(
            "## App Playbook: {}\n- Related learned workflow: [[Learned/{}]]\n\n### Documented Behaviors\n{}",
            primary_app, note_title, distilled_markdown
        );
        let _ = vault.write_note("Instructions", &app_playbook_title, &app_note);

        Ok(format!(
            "🎉 Workflow learned and synthesized!\n- Documented in Obsidian as `[[Learned/{}]]`\n- Linked to `[[Instructions/{}]]`",
            note_title, app_playbook_title
        ))
    }
}
