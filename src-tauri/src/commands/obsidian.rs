use std::path::PathBuf;
use tauri::State;

use crate::services::obsidian::{MemoryNote, ObsidianVault, VaultInfo};

#[tauri::command]
pub fn set_vault_path(vault: State<'_, ObsidianVault>, path: String) -> Result<VaultInfo, String> {
    vault.set_path(PathBuf::from(path))
}

#[tauri::command]
pub fn get_vault_info(vault: State<'_, ObsidianVault>) -> Result<VaultInfo, String> {
    vault.get_info()
}

#[tauri::command]
pub fn search_vault(vault: State<'_, ObsidianVault>, query: String) -> Result<Vec<MemoryNote>, String> {
    vault.search(&query)
}

#[tauri::command]
pub fn create_vault_test_note(vault: State<'_, ObsidianVault>) -> Result<String, String> {
    vault.create_test_note()
}

#[tauri::command]
pub fn rewrite_vault_note(
    vault: State<'_, ObsidianVault>,
    category: String,
    title: String,
    content: String,
) -> Result<String, String> {
    vault.rewrite_note(&category, &title, &content)
}

#[tauri::command]
pub fn list_vault_notes(vault: State<'_, ObsidianVault>) -> Result<Vec<MemoryNote>, String> {
    vault.list_all_notes()
}

#[tauri::command]
pub fn open_vault_in_explorer(vault: State<'_, ObsidianVault>) -> Result<(), String> {
    vault.open_in_explorer()
}

#[tauri::command]
pub fn get_system_identity(vault: State<'_, ObsidianVault>) -> String {
    vault.get_system_identity()
}

#[tauri::command]
pub fn save_system_identity(vault: State<'_, ObsidianVault>, content: String) -> Result<(), String> {
    vault.set_system_identity(&content)
}

#[tauri::command]
pub fn reset_system_identity(vault: State<'_, ObsidianVault>) -> Result<String, String> {
    vault.set_system_identity("")?;
    Ok(vault.get_system_identity())
}

