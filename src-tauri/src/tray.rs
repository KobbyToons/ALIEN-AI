// ── System-tray icon ─────────────────────────────────────────────────

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

/// Build the system-tray icon with a right-click context menu.
pub fn create_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let icon = if let Some(default_icon) = app.default_window_icon() {
        default_icon.clone()
    } else {
        let icon_rgba = generate_orb_icon(32);
        tauri::image::Image::new_owned(icon_rgba, 32, 32)
    };

    let toggle_item = MenuItemBuilder::with_id("toggle", "Show / Hide ALIEN (Ctrl+Space)").build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;
    let exit_item = MenuItemBuilder::with_id("exit", "Exit ALIEN").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&toggle_item, &settings_item, &exit_item])
        .build()?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ALIEN — Ctrl+Space to summon")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "exit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn generate_orb_icon(size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    let radius = center - 1.5;
    let hl_x = center - radius * 0.35;
    let hl_y = center - radius * 0.35;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius + 0.5 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let hdx = x as f32 - hl_x + 0.5;
                let hdy = y as f32 - hl_y + 0.5;
                let hdist = (hdx * hdx + hdy * hdy).sqrt() / (radius * 1.4);
                let factor = (1.0 - hdist.clamp(0.0, 1.0)).powi(2);

                let r = (255.0 * factor + 220.0 * (1.0 - factor)).clamp(0.0, 255.0) as u8;
                let g = (130.0 * factor + 25.0 * (1.0 - factor)).clamp(0.0, 255.0) as u8;
                let b = (150.0 * factor + 35.0 * (1.0 - factor)).clamp(0.0, 255.0) as u8;

                let alpha = if dist <= radius - 0.5 {
                    255
                } else {
                    ((radius + 0.5 - dist) * 255.0) as u8
                };
                rgba.extend_from_slice(&[r, g, b, alpha]);
            }
        }
    }
    rgba
}
