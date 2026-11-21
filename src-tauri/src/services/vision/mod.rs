// ── ALIEN Vision Service ── Phase A: Monitor Mapper + Screen Capture ──────────
//
// Provides:
//  - MonitorInfo: physical bounds + DPI scale for each connected display.
//  - capture_monitor: grab one (or primary) monitor as a downscaled JPEG.
//  - All OS-specific work is behind #[cfg(windows)] but xcap itself is
//    cross-platform, so the module compiles anywhere.

use xcap::Monitor;

/// Metadata about a single physical display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorInfo {
    /// Human-readable name (e.g. "\\.\DISPLAY1").
    pub name: String,
    /// Physical pixel dimensions.
    pub width: u32,
    pub height: u32,
    /// Top-left origin in virtual-desktop physical coordinates.
    pub x: i32,
    pub y: i32,
    /// DPI scale factor (1.0 = 96 dpi, 2.0 = 192 dpi / 200%).
    pub scale_factor: f32,
    /// True when this is the primary (main) monitor.
    pub is_primary: bool,
}

/// List every connected monitor with its physical geometry and DPI info.
pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
    let infos = monitors
        .into_iter()
        .map(|m| MonitorInfo {
            name: m.name().to_string(),
            width: m.width(),
            height: m.height(),
            x: m.x(),
            y: m.y(),
            scale_factor: m.scale_factor(),
            is_primary: m.is_primary(),
        })
        .collect();
    Ok(infos)
}

/// Convert logical (DPI-scaled) coordinates to physical pixels for a given monitor.
#[allow(dead_code)]
pub fn logical_to_physical(logical_x: f32, logical_y: f32, scale: f32) -> (i32, i32) {
    ((logical_x * scale) as i32, (logical_y * scale) as i32)
}

/// Convert a 0–1000 normalised grid coordinate to physical pixels on a monitor.
#[allow(dead_code)]
pub fn normalised_to_physical(norm_x: f32, norm_y: f32, info: &MonitorInfo) -> (i32, i32) {
    let px = info.x + ((norm_x / 1000.0) * info.width as f32) as i32;
    let py = info.y + ((norm_y / 1000.0) * info.height as f32) as i32;
    (px, py)
}

/// Capture a monitor (by index; 0 = primary when None) and return a JPEG encoded
/// as a Base64 string.  The image is downscaled so the long edge is at most
/// `max_long_edge` pixels (default 1280) before encoding.
pub fn capture_monitor_base64(
    monitor_index: Option<usize>,
    max_long_edge: Option<u32>,
) -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {e}"))?;
    if monitors.is_empty() {
        return Err("No monitors found.".into());
    }

    // Pick the requested monitor (or the primary one).
    let monitor = if let Some(idx) = monitor_index {
        monitors
            .into_iter()
            .nth(idx)
            .ok_or_else(|| format!("Monitor index {idx} out of range."))?
    } else {
        monitors
            .into_iter()
            .find(|m| m.is_primary())
            .or_else(|| {
                // Fallback: re-enumerate and take the first
                Monitor::all().ok().and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            })
            .ok_or("No primary monitor found.")?
    };

    // Capture as an xcap RgbaImage (DynamicImage under the hood).
    let image = monitor
        .capture_image()
        .map_err(|e| format!("Capture failed: {e}"))?;

    // Convert to image::DynamicImage for resizing + JPEG encoding.
    let dyn_image = image::DynamicImage::ImageRgba8(image);

    // Downscale so the long edge ≤ max_long_edge.
    let max_edge = max_long_edge.unwrap_or(1280);
    let (orig_w, orig_h) = (dyn_image.width(), dyn_image.height());
    let long_edge = orig_w.max(orig_h);
    let resized = if long_edge > max_edge {
        let scale = max_edge as f32 / long_edge as f32;
        let new_w = (orig_w as f32 * scale) as u32;
        let new_h = (orig_h as f32 * scale) as u32;
        dyn_image.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        dyn_image
    };

    // Encode as JPEG (quality 80).
    let mut jpeg_buf: Vec<u8> = Vec::new();
    resized
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg_buf),
            image::ImageFormat::Jpeg,
        )
        .map_err(|e| format!("JPEG encode failed: {e}"))?;

    // Base64-encode and return.
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&jpeg_buf))
}
