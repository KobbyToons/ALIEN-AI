use tauri::State;

use crate::services::learning::{LearningConfig, LearningService, ScenarioQuestion, ThinkingProfile};
use crate::services::obsidian::ObsidianVault;

#[tauri::command]
pub fn get_learning_config(learning: State<'_, LearningService>) -> LearningConfig {
    learning.get_config()
}

#[tauri::command]
pub fn update_learning_config(learning: State<'_, LearningService>, config: LearningConfig) {
    learning.update_config(config);
}

#[tauri::command]
pub fn get_next_question(learning: State<'_, LearningService>) -> Option<ScenarioQuestion> {
    learning.get_next_question()
}

#[tauri::command]
pub fn answer_question(
    learning: State<'_, LearningService>,
    vault: State<'_, ObsidianVault>,
    question_id: String,
    answer: String,
) -> Result<(), String> {
    learning.answer_question(&vault, &question_id, &answer)
}

#[tauri::command]
pub fn get_thinking_profile(
    learning: State<'_, LearningService>,
    vault: State<'_, ObsidianVault>,
) -> ThinkingProfile {
    learning.get_thinking_profile(&vault)
}

