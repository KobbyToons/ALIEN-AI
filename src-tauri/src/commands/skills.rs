use tauri::State;

use crate::skills::{LoadedSkill, SkillLoader};

#[tauri::command]
pub fn list_skills(loader: State<'_, SkillLoader>) -> Vec<LoadedSkill> {
    loader.get_skills()
}

#[tauri::command]
pub fn reload_skills(loader: State<'_, SkillLoader>) -> Result<Vec<LoadedSkill>, String> {
    loader.reload()
}

