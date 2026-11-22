// ── ALIEN Input Injector ── Phase C: Hands ──────────────────────────────────
//
// Direct Win32 input injection via SetCursorPos and SendInput with:
//  - Eased human-like movement curves (400–850 ms).
//  - Multi-monitor physical pixel coordinate mapping.
//  - Mouse clicks, drag, and scroll.
//  - Unicode text typing (KEYEVENTF_UNICODE) & key/hotkey combos.
//  - Tagged dwExtraInfo = 0x414C49454E so low-level hooks recognize ALIEN's own events.

use std::time::Duration;
use tokio::time::sleep;

pub const ALIEN_INPUT_SIGNATURE: usize = 0x414C49454E; // "ALIEN" in hex

#[cfg(windows)]
pub mod win32 {
    use super::ALIEN_INPUT_SIGNATURE;

    pub const INPUT_MOUSE: u32 = 0;
    pub const INPUT_KEYBOARD: u32 = 1;

    #[allow(dead_code)]
    pub const MOUSEEVENTF_MOVE: u32 = 0x0001;
    pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
    pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
    pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
    pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
    pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
    pub const MOUSEEVENTF_WHEEL: u32 = 0x0800;

    #[allow(dead_code)]
    pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
    pub const KEYEVENTF_KEYUP: u32 = 0x0002;
    pub const KEYEVENTF_UNICODE: u32 = 0x0004;

    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct MOUSEINPUT {
        pub dx: i32,
        pub dy: i32,
        pub mouse_data: u32,
        pub dw_flags: u32,
        pub time: u32,
        pub dw_extra_info: usize,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct KEYBDINPUT {
        pub w_vk: u16,
        pub w_scan: u16,
        pub dw_flags: u32,
        pub time: u32,
        pub dw_extra_info: usize,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct HARDWAREINPUT {
        pub u_msg: u32,
        pub w_param_l: u16,
        pub w_param_h: u16,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union INPUT_UNION {
        pub mi: MOUSEINPUT,
        pub ki: KEYBDINPUT,
        pub hi: HARDWAREINPUT,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct INPUT {
        pub r#type: u32,
        pub u: INPUT_UNION,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SendInput(c_inputs: u32, p_inputs: *const INPUT, cb_size: i32) -> u32;
        pub fn GetCursorPos(lp_point: *mut POINT) -> i32;
        pub fn SetCursorPos(x: i32, y: i32) -> i32;
    }

    /// Get current cursor position in physical screen coordinates.
    pub fn get_cursor_pos() -> (i32, i32) {
        let mut pt = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut pt);
        }
        (pt.x, pt.y)
    }

    /// Set cursor position instantly.
    pub fn set_cursor_pos(x: i32, y: i32) -> bool {
        unsafe { SetCursorPos(x, y) != 0 }
    }

    /// Send a single mouse event flagged with ALIEN's signature.
    pub fn send_mouse_event(flags: u32, data: u32) {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            u: INPUT_UNION {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouse_data: data,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: ALIEN_INPUT_SIGNATURE,
                },
            },
        };
        unsafe {
            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// Send a single keyboard event flagged with ALIEN's signature.
    pub fn send_key_event(vk: u16, scan: u16, flags: u32) {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            u: INPUT_UNION {
                ki: KEYBDINPUT {
                    w_vk: vk,
                    w_scan: scan,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: ALIEN_INPUT_SIGNATURE,
                },
            },
        };
        unsafe {
            SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

pub struct InputInjector;

impl InputInjector {
    /// Smoothly move the mouse from its current location to (target_x, target_y)
    /// along an eased curve over 350–800 ms.
    pub async fn move_cursor_smooth(target_x: i32, target_y: i32) -> Result<(), String> {
        #[cfg(windows)]
        {
            let (start_x, start_y) = win32::get_cursor_pos();
            let dx = (target_x - start_x) as f32;
            let dy = (target_y - start_y) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 2.0 {
                win32::set_cursor_pos(target_x, target_y);
                return Ok(());
            }

            // Duration scales naturally with distance: 350ms to 750ms
            let duration_ms = (350.0 + dist * 0.35).min(750.0);
            let steps = ((duration_ms / 8.0) as usize).clamp(25, 80);

            // Subtle perpendicular curve displacement for human-like arc
            let perp_x = -dy / dist;
            let perp_y = dx / dist;
            let arc_magnitude = (dist * 0.12).min(45.0);
            let ctrl_x = (start_x as f32 + target_x as f32) / 2.0 + perp_x * arc_magnitude;
            let ctrl_y = (start_y as f32 + target_y as f32) / 2.0 + perp_y * arc_magnitude;

            for i in 1..=steps {
                // Check if takeover or kill switch triggered
                if super::is_takeover_triggered() || super::is_kill_switch_active() {
                    return Err("Input aborted: user takeover or kill switch triggered.".into());
                }

                let t = i as f32 / steps as f32;
                // Ease In-Out Sine
                let eased_t = -( (std::f32::consts::PI * t).cos() - 1.0) / 2.0;

                // Quadratic Bezier formula
                let u = 1.0 - eased_t;
                let cur_x = u * u * (start_x as f32) + 2.0 * u * eased_t * ctrl_x + eased_t * eased_t * (target_x as f32);
                let cur_y = u * u * (start_y as f32) + 2.0 * u * eased_t * ctrl_y + eased_t * eased_t * (target_y as f32);

                win32::set_cursor_pos(cur_x as i32, cur_y as i32);
                sleep(Duration::from_millis(8)).await;
            }

            win32::set_cursor_pos(target_x, target_y);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Click mouse button ('left', 'right', 'middle', 'double') with natural pre-click pause.
    pub async fn click(button: &str) -> Result<(), String> {
        #[cfg(windows)]
        {
            if super::is_takeover_triggered() || super::is_kill_switch_active() {
                return Err("Action aborted: user takeover or kill switch active.".into());
            }

            // Small natural pause before clicking (35ms)
            sleep(Duration::from_millis(35)).await;

            match button {
                "left" => {
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0);
                    sleep(Duration::from_millis(45)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTUP, 0);
                }
                "right" => {
                    win32::send_mouse_event(win32::MOUSEEVENTF_RIGHTDOWN, 0);
                    sleep(Duration::from_millis(45)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_RIGHTUP, 0);
                }
                "middle" => {
                    win32::send_mouse_event(win32::MOUSEEVENTF_MIDDLEDOWN, 0);
                    sleep(Duration::from_millis(45)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_MIDDLEUP, 0);
                }
                "double" => {
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0);
                    sleep(Duration::from_millis(35)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTUP, 0);
                    sleep(Duration::from_millis(60)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0);
                    sleep(Duration::from_millis(35)).await;
                    win32::send_mouse_event(win32::MOUSEEVENTF_LEFTUP, 0);
                }
                _ => return Err(format!("Unknown button: {button}")),
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Drag from (x1, y1) to (x2, y2).
    pub async fn drag(x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), String> {
        #[cfg(windows)]
        {
            Self::move_cursor_smooth(x1, y1).await?;
            sleep(Duration::from_millis(50)).await;

            win32::send_mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0);
            sleep(Duration::from_millis(60)).await;

            Self::move_cursor_smooth(x2, y2).await?;
            sleep(Duration::from_millis(50)).await;

            win32::send_mouse_event(win32::MOUSEEVENTF_LEFTUP, 0);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Scroll wheel by delta (positive = up, negative = down).
    pub async fn scroll(delta_y: i32) -> Result<(), String> {
        #[cfg(windows)]
        {
            if super::is_takeover_triggered() || super::is_kill_switch_active() {
                return Err("Scroll aborted: user takeover or kill switch active.".into());
            }

            // WHEEL_DELTA is 120 in Windows
            let raw_data = (delta_y * 120) as u32;
            win32::send_mouse_event(win32::MOUSEEVENTF_WHEEL, raw_data);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Type arbitrary text with human typing cadences via Win32 Unicode injection.
    pub async fn type_text(text: &str) -> Result<(), String> {
        #[cfg(windows)]
        {
            for c in text.chars() {
                if super::is_takeover_triggered() || super::is_kill_switch_active() {
                    return Err("Typing aborted: user takeover or kill switch active.".into());
                }

                // Handle UTF-16 code units
                let mut buf = [0u16; 2];
                let encoded = c.encode_utf16(&mut buf);
                for &code in encoded.iter() {
                    win32::send_key_event(0, code, win32::KEYEVENTF_UNICODE);
                    sleep(Duration::from_millis(15)).await;
                    win32::send_key_event(0, code, win32::KEYEVENTF_UNICODE | win32::KEYEVENTF_KEYUP);
                }

                // Variable typing pause (25ms to 55ms) for natural human-like cadence
                let delay = 25 + (c as u64 % 30);
                sleep(Duration::from_millis(delay)).await;
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Press a special key (e.g. "enter", "tab", "esc", "backspace", "space").
    pub async fn press_key(key: &str) -> Result<(), String> {
        #[cfg(windows)]
        {
            let vk = match key.to_lowercase().as_str() {
                "enter" | "return" => 0x0D,
                "tab" => 0x09,
                "esc" | "escape" => 0x1B,
                "backspace" => 0x08,
                "space" => 0x20,
                "delete" => 0x2E,
                "up" => 0x26,
                "down" => 0x28,
                "left" => 0x25,
                "right" => 0x27,
                "home" => 0x24,
                "end" => 0x23,
                _ => return Err(format!("Unsupported special key: {key}")),
            };

            win32::send_key_event(vk, 0, 0);
            sleep(Duration::from_millis(40)).await;
            win32::send_key_event(vk, 0, win32::KEYEVENTF_KEYUP);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Press a hotkey combo (e.g. "ctrl+s", "ctrl+shift+esc", "alt+f4").
    pub async fn hotkey(combo: &str) -> Result<(), String> {
        #[cfg(windows)]
        {
            let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
            let mut vks: Vec<u16> = Vec::new();

            for part in parts {
                let vk: u16 = match part.to_lowercase().as_str() {
                    "ctrl" | "control" => 0x11,
                    "shift" => 0x10,
                    "alt" => 0x12,
                    "win" | "meta" => 0x5B,
                    "enter" => 0x0D,
                    "tab" => 0x09,
                    "esc" => 0x1B,
                    "f1" => 0x70,
                    "f2" => 0x71,
                    "f3" => 0x72,
                    "f4" => 0x73,
                    "f5" => 0x74,
                    "f6" => 0x75,
                    "f7" => 0x76,
                    "f8" => 0x77,
                    "f9" => 0x78,
                    "f10" => 0x79,
                    "f11" => 0x7A,
                    "f12" => 0x7B,
                    s if s.len() == 1 => {
                        let c = s.chars().next().unwrap().to_ascii_uppercase();
                        c as u16
                    }
                    _ => return Err(format!("Unsupported hotkey component: {part}")),
                };
                vks.push(vk);
            }

            // Press all modifiers down
            for &vk in &vks {
                win32::send_key_event(vk, 0, 0);
                sleep(Duration::from_millis(20)).await;
            }

            sleep(Duration::from_millis(40)).await;

            // Release in reverse order
            for &vk in vks.iter().rev() {
                win32::send_key_event(vk, 0, win32::KEYEVENTF_KEYUP);
                sleep(Duration::from_millis(15)).await;
            }

            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}
