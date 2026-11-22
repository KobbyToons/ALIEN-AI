// ── ALIEN Visible Presence ── Crash-Safe Red Cursor ─────────────────────────
//
// Replaces the standard system cursor with a custom red cursor during Agent mode,
// and ensures 100% crash-safe restoration of standard cursors on stop, exit,
// panic, or next application start.

#[cfg(windows)]
mod win32 {
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};

    static IS_RED_ACTIVE: AtomicBool = AtomicBool::new(false);

    // Win32 constants
    const OCR_NORMAL: u32 = 32512;
    const SPI_SETCURSORS: u32 = 0x0057;
    const BI_RGB: u32 = 0;
    const DIB_RGB_COLORS: u32 = 0;

    #[repr(C)]
    struct BITMAPINFOHEADER {
        bi_size: u32,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: u32,
        bi_size_image: u32,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: u32,
        bi_clr_important: u32,
    }

    #[repr(C)]
    struct ICONINFO {
        f_icon: i32,
        x_hotspot: u32,
        y_hotspot: u32,
        hbm_mask: *mut std::ffi::c_void,
        hbm_color: *mut std::ffi::c_void,
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetDC(hwnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn ReleaseDC(hwnd: *mut std::ffi::c_void, hdc: *mut std::ffi::c_void) -> i32;
        fn SetSystemCursor(hcur: *mut std::ffi::c_void, id: u32) -> i32;
        fn SystemParametersInfoW(
            ui_action: u32,
            ui_param: u32,
            pv_param: *mut std::ffi::c_void,
            f_win_ini: u32,
        ) -> i32;
        fn CreateIconIndirect(piconinfo: *mut ICONINFO) -> *mut std::ffi::c_void;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateBitmap(
            n_width: i32,
            n_height: i32,
            n_planes: u32,
            n_bit_count: u32,
            lp_bits: *const std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn CreateDIBSection(
            hdc: *mut std::ffi::c_void,
            pbmi: *const BITMAPINFOHEADER,
            usage: u32,
            ppv_bits: *mut *mut std::ffi::c_void,
            h_section: *mut std::ffi::c_void,
            offset: u32,
        ) -> *mut std::ffi::c_void;
        fn DeleteObject(ho: *mut std::ffi::c_void) -> i32;
    }

    /// Restore standard Windows cursors from the active system theme.
    pub fn restore_default_cursors() {
        unsafe {
            SystemParametersInfoW(SPI_SETCURSORS, 0, ptr::null_mut(), 0);
        }
        IS_RED_ACTIVE.store(false, Ordering::SeqCst);
    }

    /// Whether the custom red cursor is currently applied.
    #[allow(dead_code)]
    #[allow(dead_code)]
pub fn is_red_cursor_active() -> bool {
        IS_RED_ACTIVE.load(Ordering::SeqCst)
    }

    /// Create a 32x32 RGBA red arrow cursor with black outline and hot-spot at (0, 0).
    unsafe fn create_red_arrow_cursor() -> Option<*mut std::ffi::c_void> {
        let width: i32 = 32;
        let height: i32 = 32;

        let bih = BITMAPINFOHEADER {
            bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            bi_width: width,
            bi_height: -height, // Negative = top-down
            bi_planes: 1,
            bi_bit_count: 32,
            bi_compression: BI_RGB,
            bi_size_image: 0,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        };

        let hdc_screen = GetDC(ptr::null_mut());
        let mut pv_bits: *mut std::ffi::c_void = ptr::null_mut();

        let hbm_color = CreateDIBSection(
            hdc_screen,
            &bih,
            DIB_RGB_COLORS,
            &mut pv_bits,
            ptr::null_mut(),
            0,
        );

        if hbm_color.is_null() || pv_bits.is_null() {
            ReleaseDC(ptr::null_mut(), hdc_screen);
            return None;
        }

        // Fill 32x32 pixel buffer with transparent ARGB: 0x00000000
        let pixels = std::slice::from_raw_parts_mut(pv_bits as *mut u32, (width * height) as usize);
        pixels.fill(0x00000000);

        // Draw a crisp red arrow with black border
        // Arrow shape up to 24px tall
        for y in 0..24 {
            for x in 0..=y {
                let idx = (y * width + x) as usize;
                if idx < pixels.len() {
                    if x == 0 || x == y || y == 23 || (y > 14 && x == y / 2) {
                        // Black border: Alpha 255, R: 0, G: 0, B: 0
                        pixels[idx] = 0xFF000000;
                    } else {
                        // Glowing Red fill: Alpha 255, R: 255, G: 30, B: 30 (ARGB)
                        pixels[idx] = 0xFFFF1E1E;
                    }
                }
            }
        }

        // Dummy 1-bpp monochrome mask
        let hbm_mask = CreateBitmap(width, height, 1, 1, ptr::null());

        let mut ii = ICONINFO {
            f_icon: 0, // 0 = cursor
            x_hotspot: 0,
            y_hotspot: 0,
            hbm_mask,
            hbm_color,
        };

        let h_cursor = CreateIconIndirect(&mut ii);

        // Delete temporary bitmaps (CreateIconIndirect makes its own internal copy)
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        ReleaseDC(ptr::null_mut(), hdc_screen);

        if h_cursor.is_null() {
            None
        } else {
            Some(h_cursor)
        }
    }

    /// Set system cursor to the red agent cursor.
    pub fn set_red_system_cursor() -> Result<(), String> {
        unsafe {
            if let Some(h_cursor) = create_red_arrow_cursor() {
                // SetSystemCursor destroys the passed handle and takes ownership
                let ok = SetSystemCursor(h_cursor, OCR_NORMAL);
                if ok != 0 {
                    IS_RED_ACTIVE.store(true, Ordering::SeqCst);
                    Ok(())
                } else {
                    Err("SetSystemCursor failed".into())
                }
            } else {
                Err("Failed to create red arrow cursor".into())
            }
        }
    }
}

/// Public cross-platform interface
pub fn set_red_cursor() -> Result<(), String> {
    #[cfg(windows)]
    {
        win32::set_red_system_cursor()
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

pub fn restore_default_cursors() {
    #[cfg(windows)]
    {
        win32::restore_default_cursors();
    }
}

#[allow(dead_code)]
pub fn is_red_cursor_active() -> bool {
    #[cfg(windows)]
    {
        win32::is_red_cursor_active()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Initialize crash safety: registers panic hook and resets cursor on app launch.
pub fn init_cursor_safety() {
    restore_default_cursors();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_default_cursors();
        prev_hook(info);
    }));
}
