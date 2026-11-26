// ── FilmCraft Types ──────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// Connection and operational state reported to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilmCraftConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for FilmCraftConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Autonomous editing safety mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EditingMode {
    /// Analyse footage and plan only. No modifications.
    #[default]
    Planning,
    /// Execute edits; require user approval before destructive ops.
    Supervised,
    /// Full autonomy with checkpoints; no source-file deletion ever.
    Autonomous,
}

/// Which MCP operating mode to use when launching filmcraft-cli.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpLaunchMode {
    /// No window; empty project.
    Headless,
    /// No window; demo project.
    HeadlessDemo,
    /// No window; specific project file.
    HeadlessProject(String),
    /// Drives a running desktop instance on the given address.
    Bridge(String),
}

impl Default for McpLaunchMode {
    fn default() -> Self {
        Self::Headless
    }
}

/// Snapshot of FilmCraft state sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilmCraftStatus {
    pub connection: FilmCraftConnectionState,
    pub mode: EditingMode,
    pub launch_mode: McpLaunchMode,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub cli_path: Option<String>,
    pub tools_count: usize,
    pub current_operation: Option<String>,
    pub steps_taken: usize,
    pub last_error: Option<String>,
}

/// One MCP tool as returned by tools/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmCraftMcpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// A single step recorded in the editing history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingStep {
    pub step: usize,
    pub tool: String,
    pub description: String,
    pub success: bool,
}
