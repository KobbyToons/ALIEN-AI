// ── ALIEN Visible Presence ── Overlay Service ──────────────────────────────
//
// Manages the red edge-glow overlays across all monitors.
// Ensures:
//  1. Each overlay is transparent, frameless, and always-on-top.
//  2. Excluded from capture on OS level via SetWindowDisplayAffinity (WDA_EXCLUDEFROMCAPTURE).
//  3. Click-through via set_ignore_cursor_events(true) and WS_EX_TRANSPARENT.
//  4. Non-focusable via WS_EX_NOACTIVATE so user typing is never stolen.

pub mod cursor;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::services::vision;

#[cfg(windows)]
mod win32 {
    pub const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const WS_EX_NOACTIVATE: isize = 0x08000000;
    pub const WS_EX_TRANSPARENT: isize = 0x00000020;
    pub const WS_EX_LAYERED: isize = 0x00080000;

    #[link(name = "user32")]
    extern "system" {
        pub fn SetWindowDisplayAffinity(hwnd: *mut std::ffi::c_void, dw_affinity: u32) -> i32;
        pub fn GetWindowLongPtrW(hwnd: *mut std::ffi::c_void, n_index: i32) -> isize;
        pub fn SetWindowLongPtrW(
            hwnd: *mut std::ffi::c_void,
            n_index: i32,
            dw_new_long: isize,
        ) -> isize;
    }

    /// Configure HWND for click-through, non-activate, and exclusion from capture.
    pub unsafe fn configure_overlay_hwnd(hwnd: *mut std::ffi::c_void) {
        if hwnd.is_null() {
            return;
        }

        // 1. Exclude from screen capture (BitBlt, Windows Graphics Capture, etc.)
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);

        // 2. Add WS_EX_TRANSPARENT, WS_EX_LAYERED, and WS_EX_NOACTIVATE
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_style = ex_style | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE;
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
    }
}

pub struct OverlayService {
    is_active: Arc<AtomicBool>,
}

impl OverlayService {
    pub fn new() -> Self {
        cursor::init_cursor_safety();
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Toggle agent presence overlay and red cursor.
    pub fn set_presence(&self, app: &AppHandle, active: bool) -> Result<(), String> {
        let monitors = vision::list_monitors()?;

        if active {
            // Spawn or show overlay on each monitor
            for (idx, mon) in monitors.iter().enumerate() {
                let label = format!("alien-overlay-{}", idx);
                if let Some(win) = app.get_webview_window(&label) {
                    let _ = win.show();
                    let _ = win.unminimize();
                } else {
                    // Create overlay window sized and positioned to this monitor
                    let win = WebviewWindowBuilder::new(
                        app,
                        &label,
                        WebviewUrl::App(format!("/?mode=overlay&monitor={}", idx).into()),
                    )
                    .title("ALIEN Presence Overlay")
                    .transparent(true)
                    .decorations(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .resizable(false)
                    .shadow(false)
                    .position(mon.x as f64, mon.y as f64)
                    .inner_size(mon.width as f64, mon.height as f64)
                    .build()
                    .map_err(|e| format!("Failed to create overlay window {label}: {e}"))?;

                    // Configure click-through in Tauri
                    let _ = win.set_ignore_cursor_events(true);

                    // Win32 HWND customizations: exclusion from capture & style bits
                    #[cfg(windows)]
                    {
                        if let Ok(hwnd) = win.hwnd() {
                            unsafe {
                                win32::configure_overlay_hwnd(hwnd.0 as *mut std::ffi::c_void);
                            }
                        }
                    }

                    let _ = win.show();
                }
            }

            // Set red cursor
            let _ = cursor::set_red_cursor();
            self.is_active.store(true, Ordering::SeqCst);
        } else {
            // Hide all active overlays
            for idx in 0..monitors.len() {
                let label = format!("alien-overlay-{}", idx);
                if let Some(win) = app.get_webview_window(&label) {
                    let _ = win.hide();
                }
            }

            // Restore standard cursors
            cursor::restore_default_cursors();
            self.is_active.store(false, Ordering::SeqCst);
        }

        // Notify UI of state change
        let _ = app.emit(
            "buddy://agent-presence",
            serde_json::json!({ "active": active }),
        );

        Ok(())
    }
}
