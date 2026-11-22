// ── ALIEN Hands Commands ── Phase C ──────────────────────────────────────────
//
// IPC commands for mouse movement, clicking, dragging, typing, scrolling, and hotkeys.

use crate::services::input::injector::InputInjector;
use crate::services::input::{
    is_takeover_triggered, reset_kill_switch, reset_takeover,
    set_agent_active, trigger_kill_switch,
};
use tauri::AppHandle;

#[tauri::command]
pub async fn input_move(x: i32, y: i32) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::move_cursor_smooth(x, y).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_click(button: Option<String>) -> Result<(), String> {
    set_agent_active(true);
    let btn = button.unwrap_or_else(|| "left".to_string());
    let res = InputInjector::click(&btn).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_drag(x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::drag(x1, y1, x2, y2).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_scroll(delta_y: i32) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::scroll(delta_y).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_type(text: String) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::type_text(&text).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_press_key(key: String) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::press_key(&key).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub async fn input_hotkey(combo: String) -> Result<(), String> {
    set_agent_active(true);
    let res = InputInjector::hotkey(&combo).await;
    set_agent_active(false);
    res
}

#[tauri::command]
pub fn check_takeover_status() -> bool {
    is_takeover_triggered()
}

#[tauri::command]
pub fn clear_takeover_status() {
    reset_takeover();
}

#[tauri::command]
pub fn emergency_kill_switch(app: AppHandle) {
    trigger_kill_switch(Some(&app));
}

#[tauri::command]
pub fn reset_emergency_kill_switch() {
    reset_kill_switch();
}
