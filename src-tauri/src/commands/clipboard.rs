use arboard::Clipboard;
use tauri::State;
use tokio::sync::Mutex;

use crate::permissions::{PermissionGate, RiskLevel};

pub struct ClipboardState {
    pub clip: Mutex<Option<Clipboard>>,
}

#[tauri::command]
pub async fn clipboard_read(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    clip_state: State<'_, ClipboardState>,
) -> Result<String, String> {
    gate.check(&app, "clipboard_read", "Read clipboard content", RiskLevel::ReadOnly, serde_json::json!({})).await?;

    let mut lock = clip_state.clip.lock().await;
    if lock.is_none() {
        *lock = Clipboard::new().ok();
    }

    if let Some(ref mut c) = *lock {
        c.get_text().map_err(|e| format!("Failed to read clipboard: {e}"))
    } else {
        Err("Clipboard unavailable".to_string())
    }
}

#[tauri::command]
pub async fn clipboard_write(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    clip_state: State<'_, ClipboardState>,
    text: String,
) -> Result<(), String> {
    let audit_args = serde_json::json!({ "length": text.len() });
    gate.check(&app, "clipboard_write", "Write to clipboard", RiskLevel::Write, audit_args).await?;

    let mut lock = clip_state.clip.lock().await;
    if lock.is_none() {
        *lock = Clipboard::new().ok();
    }

    if let Some(ref mut c) = *lock {
        c.set_text(text).map_err(|e| format!("Failed to write clipboard: {e}"))
    } else {
        Err("Clipboard unavailable".to_string())
    }
}

