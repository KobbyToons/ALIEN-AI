// ── ALIEN Watch Mode Commands ── Phase E ─────────────────────────────────────
//
// Commands to control continuous observational screen vision.

use tauri::{AppHandle, Manager};
use crate::services::vision::WatchModeService;

#[tauri::command]
pub fn start_watch_mode(app: AppHandle) -> Result<(), String> {
    let watch = app.state::<WatchModeService>();
    watch.start(&app)
}

#[tauri::command]
pub fn stop_watch_mode(app: AppHandle) -> Result<(), String> {
    let watch = app.state::<WatchModeService>();
    watch.stop(&app)
}

#[tauri::command]
pub fn get_watch_mode_status(app: AppHandle) -> bool {
    let watch = app.state::<WatchModeService>();
    watch.is_active()
}

#[tauri::command]
pub async fn describe_screen(app: AppHandle, prompt: Option<String>) -> Result<String, String> {
    let watch = app.state::<WatchModeService>();
    let p = prompt.unwrap_or_else(|| "What am I doing on my screen right now?".into());
    watch.describe_current_screen(&app, &p).await
}
