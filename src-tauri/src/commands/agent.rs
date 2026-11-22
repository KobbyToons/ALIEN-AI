use tauri::{AppHandle, Manager};

use crate::services::agent::{AgentMetricsService, AgentService, SafetyBlocklistService};
use crate::services::agent::metrics::ComputerUseMetrics;
use crate::services::ai::ModelProvider;

#[tauri::command]
pub async fn run_agent_task(
    app: AppHandle,
    task_id: String,
    provider: ModelProvider,
    model: String,
    prompt: String,
    custom_endpoint: Option<String>,
) -> Result<String, String> {
    AgentService::run_task(app, task_id, provider, model, prompt, custom_endpoint).await
}

#[tauri::command]
pub fn get_safety_blocklist(app: AppHandle) -> Vec<String> {
    if let Some(safety) = app.try_state::<SafetyBlocklistService>() {
        safety.get_keywords()
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn add_safety_blocklist_keyword(app: AppHandle, keyword: String) -> Vec<String> {
    if let Some(safety) = app.try_state::<SafetyBlocklistService>() {
        safety.add_keyword(keyword)
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn remove_safety_blocklist_keyword(app: AppHandle, keyword: String) -> Vec<String> {
    if let Some(safety) = app.try_state::<SafetyBlocklistService>() {
        safety.remove_keyword(keyword)
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn toggle_safety_blocklist(app: AppHandle, enabled: bool) -> bool {
    if let Some(safety) = app.try_state::<SafetyBlocklistService>() {
        safety.set_enabled(enabled);
        safety.is_enabled()
    } else {
        false
    }
}

#[tauri::command]
pub fn get_computer_use_metrics(app: AppHandle) -> Option<ComputerUseMetrics> {
    app.try_state::<AgentMetricsService>().map(|m| m.get_metrics())
}

#[tauri::command]
pub fn reset_computer_use_metrics(app: AppHandle) -> Result<(), String> {
    if let Some(m) = app.try_state::<AgentMetricsService>() {
        m.reset();
    }
    Ok(())
}


