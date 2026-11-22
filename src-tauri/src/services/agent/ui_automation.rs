// ── ALIEN UI Automation Hints ── Phase G: Hardening ─────────────────────────
//
// Extracts accessible Win32 / UWP element names and bounds from the active window.
// Provides normalized (0-1000) hints to the vision model for pinpoint accuracy
// when interacting with dialog boxes, toolbars, buttons, and menus.

use serde::{Deserialize, Serialize};
use crate::services::vision::MonitorInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElementHint {
    pub name: String,
    pub class_name: String,
    pub norm_box: [u32; 4], // [x1, y1, x2, y2] in 0-1000 scale
}

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type LPARAM = isize;

    pub const TRUE: BOOL = 1;
    pub const FALSE: BOOL = 0;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, Default)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

    #[link(name = "user32")]
    extern "system" {
        pub fn GetForegroundWindow() -> HWND;
        pub fn GetWindowRect(hwnd: HWND, lp_rect: *mut RECT) -> BOOL;
        pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
        pub fn GetWindowTextW(hwnd: HWND, lp_string: *mut u16, n_max_count: i32) -> i32;
        pub fn GetClassNameW(hwnd: HWND, lp_class_name: *mut u16, n_max_count: i32) -> i32;
        pub fn EnumChildWindows(hwnd_parent: HWND, lp_enum_func: WNDENUMPROC, l_param: LPARAM) -> BOOL;
    }
}

pub struct UiAutomationService;

impl UiAutomationService {
    /// Inspect visible controls in the foreground window and map them to normalized coordinates.
    pub fn get_active_window_hints(monitor: &MonitorInfo) -> Vec<UiElementHint> {
        #[cfg(windows)]
        {
            use win32::*;

            struct EnumContext {
                elements: Vec<UiElementHint>,
                mon_x: i32,
                mon_y: i32,
                mon_w: u32,
                mon_h: u32,
            }

            unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
                let ctx = &mut *(lparam as *mut EnumContext);

                if ctx.elements.len() >= 30 {
                    return FALSE; // Limit to 30 elements to keep model prompt concise
                }

                if IsWindowVisible(hwnd) == 0 {
                    return TRUE;
                }

                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect) == 0 {
                    return TRUE;
                }

                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;

                // Ignore tiny invisible spacers or gigantic full-screen wrappers
                if width < 8 || height < 8 || (width > (ctx.mon_w as i32 * 9 / 10) && height > (ctx.mon_h as i32 * 9 / 10)) {
                    return TRUE;
                }

                // Get text
                let mut text_buf = [0u16; 128];
                let text_len = GetWindowTextW(hwnd, text_buf.as_mut_ptr(), 128);
                let text = if text_len > 0 {
                    String::from_utf16_lossy(&text_buf[..text_len as usize]).trim().to_string()
                } else {
                    String::new()
                };

                // Get class name
                let mut class_buf = [0u16; 128];
                let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 128);
                let class_name = if class_len > 0 {
                    String::from_utf16_lossy(&class_buf[..class_len as usize]).trim().to_string()
                } else {
                    String::new()
                };

                // Only include elements with readable text or known clickable/editable classes
                let is_interactive = class_name.contains("Button")
                    || class_name.contains("Edit")
                    || class_name.contains("ComboBox")
                    || class_name.contains("Tab")
                    || class_name.contains("Toolbar")
                    || class_name.contains("MenuItem")
                    || !text.is_empty();

                if is_interactive {
                    // Normalize to 0-1000 scale based on monitor
                    let norm_x1 = (((rect.left - ctx.mon_x).max(0) as f32 / ctx.mon_w as f32) * 1000.0).clamp(0.0, 1000.0) as u32;
                    let norm_y1 = (((rect.top - ctx.mon_y).max(0) as f32 / ctx.mon_h as f32) * 1000.0).clamp(0.0, 1000.0) as u32;
                    let norm_x2 = (((rect.right - ctx.mon_x).max(0) as f32 / ctx.mon_w as f32) * 1000.0).clamp(0.0, 1000.0) as u32;
                    let norm_y2 = (((rect.bottom - ctx.mon_y).max(0) as f32 / ctx.mon_h as f32) * 1000.0).clamp(0.0, 1000.0) as u32;

                    let display_name = if text.is_empty() {
                        class_name.clone()
                    } else {
                        text
                    };

                    ctx.elements.push(UiElementHint {
                        name: display_name,
                        class_name,
                        norm_box: [norm_x1, norm_y1, norm_x2, norm_y2],
                    });
                }

                TRUE
            }

            unsafe {
                let fg_hwnd = GetForegroundWindow();
                if fg_hwnd.is_null() {
                    return Vec::new();
                }

                let mut ctx = EnumContext {
                    elements: Vec::new(),
                    mon_x: monitor.x,
                    mon_y: monitor.y,
                    mon_w: monitor.width,
                    mon_h: monitor.height,
                };

                EnumChildWindows(fg_hwnd, enum_child_proc, &mut ctx as *mut _ as LPARAM);
                ctx.elements
            }
        }

        #[cfg(not(windows))]
        {
            let _ = monitor;
            Vec::new()
        }
    }

    /// Format hints as a concise text block for the multimodal model.
    pub fn format_hints_for_prompt(hints: &[UiElementHint]) -> String {
        if hints.is_empty() {
            return String::new();
        }

        let mut out = String::from("### Active Window UI Automation Hints (Standard Controls):\n");
        for h in hints {
            let center_x = (h.norm_box[0] + h.norm_box[2]) / 2;
            let center_y = (h.norm_box[1] + h.norm_box[3]) / 2;
            out.push_str(&format!(
                "- [{}] \"{}\" -> Center Click: ({}, {}) | Box: [{}, {}, {}, {}]\n",
                h.class_name, h.name, center_x, center_y, h.norm_box[0], h.norm_box[1], h.norm_box[2], h.norm_box[3]
            ));
        }
        out.push('\n');
        out
    }
}
