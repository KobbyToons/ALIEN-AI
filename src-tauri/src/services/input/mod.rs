// ── ALIEN Input Subsystem ── Phase C: Hands ────────────────────────────────
//
// Coordinates:
//  - InputInjector (mouse movement, clicks, typing, hotkeys).
//  - User Takeover detector (low-level hooks).
//  - Global kill switch (Ctrl+Shift+F12).

pub mod hook;
pub mod injector;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, Manager};

static IS_AGENT_ACTIVE: AtomicBool = AtomicBool::new(false);
static IS_TAKEOVER_TRIGGERED: AtomicBool = AtomicBool::new(false);
static IS_KILL_SWITCH_ACTIVE: AtomicBool = AtomicBool::new(false);
static GLOBAL_APP_HANDLE: RwLock<Option<AppHandle>> = RwLock::new(None);

pub fn set_global_app_handle(app: AppHandle) {
    if let Ok(mut g) = GLOBAL_APP_HANDLE.write() {
        *g = Some(app);
    }
}

pub fn get_global_app_handle() -> Option<AppHandle> {
    GLOBAL_APP_HANDLE.read().ok().and_then(|g| g.clone())
}

/// Mark whether ALIEN is actively controlling mouse/keyboard.
pub fn set_agent_active(active: bool) {
    IS_AGENT_ACTIVE.store(active, Ordering::SeqCst);
    if active {
        // Reset takeover state when a new action begins
        IS_TAKEOVER_TRIGGERED.store(false, Ordering::SeqCst);
    }
}

pub fn is_agent_active() -> bool {
    IS_AGENT_ACTIVE.load(Ordering::SeqCst)
}

/// Triggered when the low-level hook detects real physical human mouse/keyboard input.
pub fn trigger_takeover() {
    IS_TAKEOVER_TRIGGERED.store(true, Ordering::SeqCst);
    set_agent_active(false);
}

pub fn is_takeover_triggered() -> bool {
    IS_TAKEOVER_TRIGGERED.load(Ordering::SeqCst)
}

pub fn reset_takeover() {
    IS_TAKEOVER_TRIGGERED.store(false, Ordering::SeqCst);
}

/// Global kill switch (Ctrl+Shift+F12 or user emergency stop).
pub fn trigger_kill_switch(app: Option<&AppHandle>) {
    IS_KILL_SWITCH_ACTIVE.store(true, Ordering::SeqCst);
    set_agent_active(false);

    if let Some(app) = app {
        // Clear presence glow and restore default cursors immediately
        if let Some(overlay) = app.try_state::<crate::services::OverlayService>() {
            let _ = overlay.set_presence(app, false);
        }
        // Emit global kill event to frontend
        let _ = app.emit("buddy://agent-kill-switch", ());
    }
}

pub fn is_kill_switch_active() -> bool {
    IS_KILL_SWITCH_ACTIVE.load(Ordering::SeqCst)
}

pub fn reset_kill_switch() {
    IS_KILL_SWITCH_ACTIVE.store(false, Ordering::SeqCst);
}

pub struct InputService;

impl InputService {
    pub fn new() -> Self {
        hook::init_takeover_hooks();
        Self
    }
}
