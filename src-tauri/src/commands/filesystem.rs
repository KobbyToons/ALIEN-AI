use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::permissions::{FileEntry, PermissionGate, RiskLevel};

#[tauri::command]
pub async fn fs_read(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    path: String,
) -> Result<String, String> {
    let desc = format!("Read file: `{path}`");
    let audit_args = serde_json::json!({ "path": path });
    gate.check(&app, "fs_read", &desc, RiskLevel::ReadOnly, audit_args).await?;

    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {e}"))
}

#[tauri::command]
pub async fn fs_write(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    path: String,
    content: String,
) -> Result<(), String> {
    let desc = format!("Write to file: `{path}`");
    let audit_args = serde_json::json!({ "path": path, "byteCount": content.len() });
    gate.check(&app, "fs_write", &desc, RiskLevel::Write, audit_args).await?;

    if let Some(parent) = PathBuf::from(&path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {e}"))
}

#[tauri::command]
pub async fn fs_list(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    let desc = format!("List directory: `{path}`");
    let audit_args = serde_json::json!({ "path": path });
    gate.check(&app, "fs_list", &desc, RiskLevel::ReadOnly, audit_args).await?;

    let entries = fs::read_dir(&path).map_err(|e| format!("Failed to read dir: {e}"))?;
    let mut result = Vec::new();

    for entry in entries.flatten() {
        let meta = entry.metadata().ok();
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        result.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn fs_remove(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    path: String,
    recursive: Option<bool>,
) -> Result<(), String> {
    let is_rec = recursive.unwrap_or(false);
    let desc = format!("Permanently delete: `{path}` (recursive: {is_rec})");
    let audit_args = serde_json::json!({ "path": path, "recursive": is_rec });
    gate.check(&app, "fs_remove", &desc, RiskLevel::Destructive, audit_args).await?;

    let p = PathBuf::from(&path);
    if p.is_dir() {
        if is_rec {
            fs::remove_dir_all(&p).map_err(|e| format!("Failed to remove dir: {e}"))
        } else {
            fs::remove_dir(&p).map_err(|e| format!("Failed to remove empty dir: {e}"))
        }
    } else {
        fs::remove_file(&p).map_err(|e| format!("Failed to remove file: {e}"))
    }
}

#[tauri::command]
pub async fn fs_copy(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    src: String,
    dest: String,
) -> Result<(), String> {
    let desc = format!("Copy `{src}` to `{dest}`");
    let audit_args = serde_json::json!({ "src": src, "dest": dest });
    gate.check(&app, "fs_copy", &desc, RiskLevel::Write, audit_args).await?;

    fs::copy(&src, &dest)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy: {e}"))
}

#[tauri::command]
pub async fn fs_move(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    src: String,
    dest: String,
) -> Result<(), String> {
    let desc = format!("Move `{src}` to `{dest}`");
    let audit_args = serde_json::json!({ "src": src, "dest": dest });
    gate.check(&app, "fs_move", &desc, RiskLevel::Write, audit_args).await?;

    fs::rename(&src, &dest).map_err(|e| format!("Failed to move: {e}"))
}

