use std::sync::atomic::Ordering;
use tauri::State;
use tauri_plugin_shell::ShellExt;

use crate::permissions::{AuditEntry, PermissionDecision, PermissionGate, RiskLevel};

#[tauri::command]
#[allow(deprecated)]
pub async fn open_url(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    url: String,
) -> Result<(), String> {
    let desc = format!("Open URL: `{url}` in default browser");
    let audit_args = serde_json::json!({ "url": url });
    gate.check(&app, "open_url", &desc, RiskLevel::ReadOnly, audit_args).await?;

    app.shell()
        .open(&url, None)
        .map_err(|e| format!("Failed to open URL: {e}"))
}

#[tauri::command]
pub async fn respond_permission(
    gate: State<'_, PermissionGate>,
    id: String,
    decision: PermissionDecision,
) -> Result<(), String> {
    gate.respond(&id, decision).await
}

#[tauri::command]
pub fn toggle_kill_switch(
    gate: State<'_, PermissionGate>,
    enabled: bool,
) -> bool {
    gate.kill_switch.store(enabled, Ordering::SeqCst);
    enabled
}

#[tauri::command]
pub fn get_kill_switch(
    gate: State<'_, PermissionGate>,
) -> bool {
    gate.kill_switch.load(Ordering::SeqCst)
}

#[tauri::command]
pub async fn get_audit_log(
    gate: State<'_, PermissionGate>,
    limit: Option<usize>,
) -> Result<Vec<AuditEntry>, String> {
    gate.recent_entries(limit.unwrap_or(50)).await
}

#[tauri::command]
pub fn resize_window(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
        let _ = window.show();
        let _ = window.set_focus();
        Ok(())
    } else {
        Err("Main window not found".into())
    }
}
