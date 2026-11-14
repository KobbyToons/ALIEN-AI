use tauri::AppHandle;

use crate::services::agent::AgentService;
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

