// ── ALIEN Capture Commands ── Phase A ─────────────────────────────────────────
//
// IPC commands exposed to the frontend via Tauri's invoke handler.

use serde::Serialize;
use crate::services::vision;

/// Serialisable monitor metadata returned to the frontend.
#[derive(Debug, Serialize)]
pub struct MonitorInfoDto {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f32,
    pub is_primary: bool,
}

/// List all connected monitors.
#[tauri::command]
pub fn list_monitors() -> Result<Vec<MonitorInfoDto>, String> {
    let monitors = vision::list_monitors()?;
    Ok(monitors
        .into_iter()
        .map(|m| MonitorInfoDto {
            name: m.name,
            width: m.width,
            height: m.height,
            x: m.x,
            y: m.y,
            scale_factor: m.scale_factor,
            is_primary: m.is_primary,
        })
        .collect())
}

/// Capture a monitor and return it as a base64-encoded JPEG string.
///
/// `monitor_index` – 0-based index; None → primary monitor.
/// `max_long_edge` – maximum long-edge pixels after downscaling (default 1280).
#[tauri::command]
pub async fn capture_screen(
    monitor_index: Option<usize>,
    max_long_edge: Option<u32>,
) -> Result<String, String> {
    // Run the blocking capture on a thread-pool so we don't stall the async runtime.
    tokio::task::spawn_blocking(move || {
        vision::capture_monitor_base64(monitor_index, max_long_edge)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))
    .and_then(|r| r)
}
