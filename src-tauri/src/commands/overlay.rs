// ── ALIEN Presence Overlay Commands ── Phase B ──────────────────────────────
//
// Commands to query and toggle the red edge glow and red cursor for Agent mode.

use tauri::{AppHandle, Manager};
use crate::services::overlay::OverlayService;

/// Toggle the visual presence overlay and red cursor.
#[tauri::command]
pub fn toggle_agent_presence(app: AppHandle, active: bool) -> Result<(), String> {
    let service = app.state::<OverlayService>();
    service.set_presence(&app, active)
}

/// Check whether the agent presence overlay and red cursor are currently active.
#[tauri::command]
pub fn get_agent_presence(app: AppHandle) -> bool {
    let service = app.state::<OverlayService>();
    service.is_active()
}
