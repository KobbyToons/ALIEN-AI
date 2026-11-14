// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod single_instance {
    use std::ffi::c_void;

    extern "system" {
        pub fn CreateMutexW(
            lpMutexAttributes: *mut c_void,
            bInitialOwner: i32,
            lpName: *const u16,
        ) -> *mut c_void;
        pub fn GetLastError() -> u32;
        pub fn CloseHandle(hObject: *mut c_void) -> i32;
        pub fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut c_void;
        pub fn ShowWindow(hWnd: *mut c_void, nCmdShow: i32) -> i32;
        pub fn SetForegroundWindow(hWnd: *mut c_void) -> i32;
    }

    const ERROR_ALREADY_EXISTS: u32 = 183;
    const SW_RESTORE: i32 = 9;

    static mut MUTEX_HANDLE: *mut c_void = std::ptr::null_mut();

    pub fn check_or_exit() -> bool {
        let name: Vec<u16> = "Local\\ALIEN_Desktop_Assistant_SingleInstance_Mutex\0"
            .encode_utf16()
            .collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr());
            if handle.is_null() {
                return true;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let title: Vec<u16> = "ALIEN\0".encode_utf16().collect();
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
                CloseHandle(handle);
                return false;
            }
            MUTEX_HANDLE = handle;
        }
        true
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let _ = std::fs::write("D:\\KOBbyTOOns\\ARt\\AL\\My Music\\ANTIGRAVITY\\crash.log", msg);
    }));

    #[cfg(windows)]
    {
        if !single_instance::check_or_exit() {
            return;
        }
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--use-fake-ui-for-media-stream",
        );
    }

    alien_lib::run();
}
