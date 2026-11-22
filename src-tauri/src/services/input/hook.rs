// ── ALIEN User Takeover Hook ── Phase C: Hands ──────────────────────────────
//
// Installs low-level Windows hooks (WH_MOUSE_LL & WH_KEYBOARD_LL) on a background thread.
// Distinguishes between physical user actions and ALIEN-injected actions.
// If the user physically touches the mouse or keyboard while the Agent is acting,
// it triggers a takeover pause immediately.

#[cfg(windows)]
mod win32_hook {
    use std::ptr;
    use tauri::Manager;
    use super::super::injector::ALIEN_INPUT_SIGNATURE;
    use super::super::{is_agent_active, trigger_takeover};

    const WH_KEYBOARD_LL: i32 = 13;
    const WH_MOUSE_LL: i32 = 14;
    const LLMHF_INJECTED: u32 = 0x00000001;
    const LLKHF_INJECTED: u32 = 0x00000010;

    const WM_LBUTTONDOWN: usize = 0x0201;
    const WM_RBUTTONDOWN: usize = 0x0204;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_SYSKEYDOWN: usize = 0x0104;

    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12; // Alt
    const VK_SHIFT: i32 = 0x10;
    const VK_LWIN: i32 = 0x5B;

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct MSLLHOOKSTRUCT {
        pt: POINT,
        mouse_data: u32,
        flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    #[repr(C)]
    struct KBDLLHOOKSTRUCT {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    #[repr(C)]
    struct MSG {
        hwnd: *mut std::ffi::c_void,
        message: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        pt: POINT,
    }

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowsHookExW(
            id_hook: i32,
            lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
            h_mod: *mut std::ffi::c_void,
            dw_thread_id: u32,
        ) -> *mut std::ffi::c_void;
        fn UnhookWindowsHookEx(hhk: *mut std::ffi::c_void) -> i32;
        fn CallNextHookEx(
            hhk: *mut std::ffi::c_void,
            n_code: i32,
            w_param: usize,
            l_param: isize,
        ) -> isize;
        fn GetMessageW(
            lp_msg: *mut MSG,
            hwnd: *mut std::ffi::c_void,
            w_msg_filter_min: u32,
            w_msg_filter_max: u32,
        ) -> i32;
        fn TranslateMessage(lp_msg: *const MSG) -> i32;
        fn DispatchMessageW(lp_msg: *const MSG) -> isize;
        fn GetAsyncKeyState(v_key: i32) -> i16;
    }

    static mut MOUSE_HOOK: *mut std::ffi::c_void = ptr::null_mut();
    static mut KEYBOARD_HOOK: *mut std::ffi::c_void = ptr::null_mut();

    unsafe extern "system" fn mouse_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
        if n_code >= 0 {
            let ms = &*(l_param as *const MSLLHOOKSTRUCT);
            let is_injected = (ms.flags & LLMHF_INJECTED) != 0 || ms.dw_extra_info == ALIEN_INPUT_SIGNATURE;

            if !is_injected {
                if is_agent_active() {
                    // Physical mouse interaction while agent was active: Takeover!
                    trigger_takeover();
                } else if w_param == WM_LBUTTONDOWN || w_param == WM_RBUTTONDOWN {
                    // Check if Learn Mode is active
                    if let Some(app) = crate::services::input::get_global_app_handle() {
                        let (x, y) = (ms.pt.x, ms.pt.y);
                        tauri::async_runtime::spawn(async move {
                            if let Some(learn) = app.try_state::<crate::services::learning::LearnModeService>() {
                                if learn.is_active() {
                                    learn.record_click(x, y).await;
                                }
                            }
                        });
                    }
                }
            }
        }
        CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
    }

    unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
        if n_code >= 0 {
            let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
            let is_injected = (kb.flags & LLKHF_INJECTED) != 0 || kb.dw_extra_info == ALIEN_INPUT_SIGNATURE;

            if !is_injected {
                if is_agent_active() {
                    // Physical keyboard press while agent was active: Takeover!
                    trigger_takeover();
                } else if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
                    // Check if Learn Mode is active
                    let ctrl = (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0;
                    let alt = (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0;
                    let shift = (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0;
                    let win = (GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0;

                    // STRICT ANTI-KEYLOGGER: Only record if a modifier is held or navigation key!
                    let is_nav = kb.vk_code == 0x0D // Enter
                        || kb.vk_code == 0x1B // Esc
                        || kb.vk_code == 0x09 // Tab
                        || (kb.vk_code >= 0x70 && kb.vk_code <= 0x7B); // F1 - F12

                    if ctrl || alt || win || is_nav {
                        let mut combo = String::new();
                        if ctrl { combo.push_str("Ctrl+"); }
                        if alt { combo.push_str("Alt+"); }
                        if shift { combo.push_str("Shift+"); }
                        if win { combo.push_str("Win+"); }

                        let key_str = match kb.vk_code {
                            0x0D => "Enter".into(),
                            0x1B => "Esc".into(),
                            0x09 => "Tab".into(),
                            0x20 => "Space".into(),
                            0x70..=0x7B => format!("F{}", kb.vk_code - 0x6F),
                            code => {
                                if (0x41..=0x5A).contains(&code) {
                                    format!("{}", (code as u8) as char)
                                } else {
                                    format!("Key{code}")
                                }
                            }
                        };
                        combo.push_str(&key_str);

                        if let Some(app) = crate::services::input::get_global_app_handle() {
                            tauri::async_runtime::spawn(async move {
                                if let Some(learn) = app.try_state::<crate::services::learning::LearnModeService>() {
                                    if learn.is_active() {
                                        learn.record_shortcut(combo).await;
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
        CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
    }

    /// Run the low-level hook message loop on a dedicated thread.
    pub fn start_hook_thread() {
        std::thread::spawn(|| unsafe {
            MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, mouse_proc, ptr::null_mut(), 0);
            KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_proc, ptr::null_mut(), 0);

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if !MOUSE_HOOK.is_null() {
                UnhookWindowsHookEx(MOUSE_HOOK);
            }
            if !KEYBOARD_HOOK.is_null() {
                UnhookWindowsHookEx(KEYBOARD_HOOK);
            }
        });
    }
}

pub fn init_takeover_hooks() {
    #[cfg(windows)]
    {
        win32_hook::start_hook_thread();
    }
}
