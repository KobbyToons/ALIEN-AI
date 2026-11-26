// ── FilmCraft Tauri Commands ─────────────────────────────────────────────────

use serde_json::Value;
use tauri::{command, AppHandle, State};

use crate::services::filmcraft::{
    types::{EditingMode, FilmCraftStatus, McpLaunchMode},
    FilmCraftService,
};

/// Detect and return the filmcraft-cli path (or None if not installed).
#[command]
pub async fn filmcraft_detect_cli() -> Option<String> {
    FilmCraftService::find_cli().map(|p| p.to_string_lossy().to_string())
}

/// Connect to FilmCraft MCP in headless mode (default).
#[command]
pub async fn filmcraft_connect(
    app: AppHandle,
    fc: State<'_, FilmCraftService>,
    cli_path: Option<String>,
    mode: Option<String>,
    project_path: Option<String>,
    bridge_addr: Option<String>,
) -> Result<FilmCraftStatus, String> {
    let launch_mode = match mode.as_deref() {
        Some("demo") => McpLaunchMode::HeadlessDemo,
        Some("project") => McpLaunchMode::HeadlessProject(
            project_path.ok_or("project_path required for 'project' mode")?,
        ),
        Some("bridge") => McpLaunchMode::Bridge(
            bridge_addr.unwrap_or_else(|| "127.0.0.1:9886".to_string()),
        ),
        _ => McpLaunchMode::Headless,
    };

    fc.connect(&app, cli_path, launch_mode)
}

/// Launch FilmCraft Desktop with a secure localhost control token and connect Bridge MCP.
#[command]
pub async fn filmcraft_launch_desktop(
    app: AppHandle,
    fc: State<'_, FilmCraftService>,
    port: Option<u16>,
) -> Result<FilmCraftStatus, String> {
    fc.launch_desktop_and_connect(&app, port.unwrap_or(9886))
}

/// Disconnect from FilmCraft MCP.
#[command]
pub async fn filmcraft_disconnect(
    app: AppHandle,
    fc: State<'_, FilmCraftService>,
) -> Result<FilmCraftStatus, String> {
    Ok(fc.disconnect(&app))
}

/// Get current FilmCraft connection status.
#[command]
pub async fn filmcraft_get_status(fc: State<'_, FilmCraftService>) -> Result<FilmCraftStatus, String> {
    Ok(fc.get_status())
}

/// Set the editing safety mode.
#[command]
pub async fn filmcraft_set_mode(
    fc: State<'_, FilmCraftService>,
    mode: String,
) -> Result<(), String> {
    let m = match mode.as_str() {
        "Planning" | "planning" => EditingMode::Planning,
        "Supervised" | "supervised" => EditingMode::Supervised,
        "Autonomous" | "autonomous" => EditingMode::Autonomous,
        other => return Err(format!("Unknown editing mode: {other}")),
    };
    fc.set_mode(m);
    Ok(())
}

/// Directly execute a FilmCraft MCP tool (for testing / advanced use).
#[command]
pub async fn filmcraft_call_tool(
    app: AppHandle,
    fc: State<'_, FilmCraftService>,
    tool_name: String,
    arguments: Value,
) -> Result<String, String> {
    fc.execute_tool(&app, &tool_name, &arguments).await
}

/// Get list of editing step history.
#[command]
pub async fn filmcraft_get_history(fc: State<'_, FilmCraftService>) -> Result<Vec<serde_json::Value>, String> {
    Ok(fc
        .get_history()
        .into_iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect())
}

/// Clear editing history and step counter.
#[command]
pub async fn filmcraft_clear_history(fc: State<'_, FilmCraftService>) -> Result<(), String> {
    fc.clear_history();
    Ok(())
}

/// Run an autonomous video editing task via the agent loop.
/// This is the high-level entry point for "edit my video" requests.
#[command]
pub async fn filmcraft_edit_task(
    app: AppHandle,
    fc: State<'_, FilmCraftService>,
    task_description: String,
    footage_paths: Vec<String>,
    mode: Option<String>,
) -> Result<String, String> {
    // Set mode if provided
    if let Some(ref m) = mode {
        let _ = filmcraft_set_mode(fc.clone(), m.clone()).await;
    }

    // Verify connected
    let status = fc.get_status();
    if !matches!(
        status.connection,
        crate::services::filmcraft::types::FilmCraftConnectionState::Connected
    ) {
        return Err(
            "FilmCraft MCP not connected. Call filmcraft_connect first.".to_string(),
        );
    }

    // For the autonomous editing task, we build a prompt and feed it to the
    // main agent loop which has the filmcraft_* tools in its registry.
    // The agent will call filmcraft_* tools iteratively to fulfil the task.
    let footage_list = if footage_paths.is_empty() {
        "No specific footage files provided — inspect project bins.".to_string()
    } else {
        footage_paths.join("\n- ")
    };

    let agent_prompt = format!(
        "You are ALIEN, an autonomous video editor using FilmCraft.\n\
         \n\
         ## Editing Task\n\
         {task_description}\n\
         \n\
         ## Footage / Assets\n\
         - {footage_list}\n\
         \n\
         ## Instructions\n\
         1. Inspect the current project using `filmcraft_project_inspect` and `filmcraft_sequence_inspect`.\n\
         2. Import any supplied footage using `filmcraft_media_import`.\n\
         3. Inspect available commands with `filmcraft_command_list` if needed.\n\
         4. Build a timeline using `filmcraft_command_run` and `filmcraft_command_batch`.\n\
         5. Render a preview frame using `filmcraft_render_frame`.\n\
         6. Save the project using `filmcraft_command_run` with id `file.save`.\n\
         7. Report what you did, the project path, and any issues encountered.\n\
         \n\
         Do not delete original footage. Do not overwrite source files.\n\
         Use `filmcraft_command_list` to discover available commands before using `filmcraft_command_run`.\n\
         Work step by step and report your progress."
    );

    // Delegate to the main AgentService with this prompt
    // The agent will discover filmcraft_* tools from the registry and use them
    let task_id = uuid::Uuid::new_v4().to_string();
    crate::services::agent::AgentService::run_task(
        app,
        task_id,
        crate::services::ai::ModelProvider::Gemini,
        "gemini-3.8-flash".to_string(),
        agent_prompt,
        None,
    )
    .await
}
