// ── FilmCraft Service — main module ─────────────────────────────────────────
//
// FilmCraftService is a Tauri-managed state singleton that owns the MCP
// connection to filmcraft-cli.  All interaction with FilmCraft goes through
// this module.

pub mod mcp_client;
pub mod types;

use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use self::mcp_client::FilmCraftConnection;
use self::types::{
    EditingMode, EditingStep, FilmCraftConnectionState, FilmCraftMcpTool, FilmCraftStatus,
    McpLaunchMode,
};
use crate::services::mcp::McpTool;

// ─── State ───────────────────────────────────────────────────────────────────

pub struct FilmCraftService {
    inner: Mutex<FilmCraftInner>,
}

struct FilmCraftInner {
    connection: Option<FilmCraftConnection>,
    state: FilmCraftStatus,
    history: Vec<EditingStep>,
}

impl FilmCraftService {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(FilmCraftInner {
                connection: None,
                state: FilmCraftStatus::default(),
                history: Vec::new(),
            }),
        }
    }

    // ─── CLI discovery ───────────────────────────────────────────────────────

    /// Find filmcraft-cli.exe in known Windows install locations.
    pub fn find_cli() -> Option<PathBuf> {
        // 1. User-local MSI install
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(&local)
                .join("Programs")
                .join("ArtCraft")
                .join("FilmCraft")
                .join("filmcraft-cli.exe");
            if p.exists() { return Some(p); }
        }

        // 2. All-users MSI install
        for prog in &["C:\\Program Files\\ArtCraft\\FilmCraft",
                      "C:\\Program Files (x86)\\ArtCraft\\FilmCraft"] {
            let p = PathBuf::from(prog).join("filmcraft-cli.exe");
            if p.exists() { return Some(p); }
        }

        // 3. Portable install in project data dir
        let portable_dirs = [
            "d:\\KOBbyTOOns\\ARt\\AL\\filmcraft\\filmcraft-0.20.1-windows-x64-portable",
            "d:\\KOBbyTOOns\\ARt\\AL\\filmcraft",
            "d:\\KOBbyTOOns\\ARt\\AL\\filmcraft-portable",
        ];
        for dir in &portable_dirs {
            let p = PathBuf::from(dir).join("filmcraft-cli.exe");
            if p.exists() { return Some(p); }
        }

        // 4. PATH
        if let Ok(output) = std::process::Command::new("where").arg("filmcraft-cli").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let first_line = path.lines().next().unwrap_or("").trim().to_string();
                if !first_line.is_empty() {
                    return Some(PathBuf::from(first_line));
                }
            }
        }

        None
    }

    // ─── Connection management ────────────────────────────────────────────────

    pub fn connect(
        &self,
        app: &AppHandle,
        cli_path_override: Option<String>,
        launch_mode: McpLaunchMode,
    ) -> Result<FilmCraftStatus, String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;

        // Drop existing connection
        inner.connection = None;
        inner.state.connection = FilmCraftConnectionState::Connecting;
        let _ = app.emit("filmcraft://status", inner.state.clone());

        // Security: restrict Bridge mode to localhost / 127.0.0.1 only
        if let McpLaunchMode::Bridge(ref addr) = launch_mode {
            let trimmed = addr.trim();
            if !trimmed.starts_with("127.0.0.1:") && !trimmed.starts_with("localhost:") {
                let err = format!(
                    "Security policy: FilmCraft bridge mode is restricted to localhost (127.0.0.1:<port> or localhost:<port>), got '{trimmed}'"
                );
                inner.state.connection = FilmCraftConnectionState::Error(err.clone());
                inner.state.last_error = Some(err.clone());
                let _ = app.emit("filmcraft://status", inner.state.clone());
                return Err(err);
            }
        }

        // Resolve CLI path
        let cli_path = if let Some(p) = cli_path_override {
            PathBuf::from(p)
        } else {
            Self::find_cli().ok_or_else(|| {
                "filmcraft-cli.exe not found. Install FilmCraft from https://github.com/storytold/filmcraft/releases or specify path in settings.".to_string()
            })?
        };

        let cli_str = cli_path.to_string_lossy().to_string();
        inner.state.cli_path = Some(cli_str.clone());

        // Launch MCP process and handshake
        match FilmCraftConnection::connect(&cli_str, &launch_mode) {
            Ok(conn) => {
                inner.state.tools_count = conn.tools.len();
                inner.state.connection = FilmCraftConnectionState::Connected;
                inner.state.launch_mode = launch_mode;
                inner.state.last_error = None;
                inner.connection = Some(conn);
                let _ = app.emit("filmcraft://status", inner.state.clone());
                Ok(inner.state.clone())
            }
            Err(e) => {
                inner.state.connection = FilmCraftConnectionState::Error(e.clone());
                inner.state.last_error = Some(e.clone());
                let _ = app.emit("filmcraft://status", inner.state.clone());
                Err(e)
            }
        }
    }

    pub fn disconnect(&self, app: &AppHandle) -> FilmCraftStatus {
        let mut inner = self.inner.lock().unwrap();
        inner.connection = None; // Drop triggers filmcraft-cli shutdown
        inner.state.connection = FilmCraftConnectionState::Disconnected;
        inner.state.tools_count = 0;
        inner.state.current_operation = None;
        let _ = app.emit("filmcraft://status", inner.state.clone());
        inner.state.clone()
    }

    pub fn get_status(&self) -> FilmCraftStatus {
        let mut inner = self.inner.lock().unwrap();
        // Check if process is still alive
        if inner.state.connection == FilmCraftConnectionState::Connected {
            if let Some(conn) = inner.connection.as_mut() {
                if !conn.is_alive() {
                    inner.state.connection =
                        FilmCraftConnectionState::Error("filmcraft-cli exited unexpectedly".into());
                    inner.connection = None;
                }
            }
        }
        inner.state.clone()
    }

    pub fn set_mode(&self, mode: EditingMode) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.mode = mode;
    }

    // ─── Tool list for McpRegistry ────────────────────────────────────────────

    /// Return all FilmCraft MCP tools, prefixed with "filmcraft_" for ALIEN's
    /// MCP registry, so Gemini can discover and call them.
    pub fn get_mcp_tools_for_registry(&self) -> Vec<McpTool> {
        let inner = self.inner.lock().unwrap();
        let tools = if let Some(conn) = &inner.connection {
            conn.tools.clone()
        } else {
            // Provide minimal stub tools even when disconnected so user can see them
            vec![
                FilmCraftMcpTool {
                    name: "command_list".into(),
                    description: "List all FilmCraft engine commands.".into(),
                    input_schema: json!({"type":"object","properties":{"filter":{"type":"string"}}}),
                },
                FilmCraftMcpTool {
                    name: "command_run".into(),
                    description: "Execute a FilmCraft engine command by id.".into(),
                    input_schema: json!({"type":"object","properties":{"id":{"type":"string"},"params":{"type":"object"}},"required":["id"]}),
                },
                FilmCraftMcpTool {
                    name: "media_import".into(),
                    description: "Import media files into the FilmCraft project.".into(),
                    input_schema: json!({"type":"object","properties":{"paths":{"type":"array","items":{"type":"string"}},"bin":{"type":"string"}},"required":["paths"]}),
                },
                FilmCraftMcpTool {
                    name: "sequence_inspect".into(),
                    description: "Inspect the active FilmCraft sequence (tracks, clips, effects).".into(),
                    input_schema: json!({"type":"object","properties":{}}),
                },
                FilmCraftMcpTool {
                    name: "render_frame".into(),
                    description: "Render a preview frame as PNG at the given time (seconds).".into(),
                    input_schema: json!({"type":"object","properties":{"seconds":{"type":"number"}},"required":["seconds"]}),
                },
                FilmCraftMcpTool {
                    name: "project_inspect".into(),
                    description: "Inspect the FilmCraft project bins and media items.".into(),
                    input_schema: json!({"type":"object","properties":{}}),
                },
                FilmCraftMcpTool {
                    name: "command_batch".into(),
                    description: "Execute multiple FilmCraft commands in order.".into(),
                    input_schema: json!({"type":"object","properties":{"steps":{"type":"array","items":{"type":"object"}},"stop_on_error":{"type":"boolean"}},"required":["steps"]}),
                },
                FilmCraftMcpTool {
                    name: "doc_inspect".into(),
                    description: "Get project tree and active sequence in one call.".into(),
                    input_schema: json!({"type":"object","properties":{}}),
                },
            ]
        };

        let mut registry_tools: Vec<McpTool> = vec![
            McpTool {
                name: "filmcraft_connect".into(),
                description: "[FilmCraft Video Editor] Connect to the FilmCraft MCP server (headless, demo, project, or bridge mode).".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "mode": { "type": "string", "description": "Optional: 'headless' (default), 'demo', 'project', or 'bridge'" },
                        "project_path": { "type": "string", "description": "Path to .fcproj file when mode is 'project'" },
                        "bridge_addr": { "type": "string", "description": "Localhost address e.g. '127.0.0.1:9886' when mode is 'bridge'" }
                    }
                }),
            },
            McpTool {
                name: "filmcraft_disconnect".into(),
                description: "[FilmCraft Video Editor] Disconnect from the FilmCraft MCP server.".into(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            McpTool {
                name: "filmcraft_status".into(),
                description: "[FilmCraft Video Editor] Check FilmCraft MCP connection status, active project, and editing mode.".into(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
        ];

        registry_tools.extend(tools.into_iter().map(|t| McpTool {
            name: format!("filmcraft_{}", t.name),
            description: format!("[FilmCraft Video Editor] {}", t.description),
            input_schema: t.input_schema,
        }));

        registry_tools
    }

    // ─── Tool execution ───────────────────────────────────────────────────────

    /// Execute a filmcraft_* tool call forwarded from McpRegistry.
    /// `name` has the "filmcraft_" prefix stripped already.
    pub async fn execute_tool(
        &self,
        app: &AppHandle,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<String, String> {
        // Safety check in supervised mode
        {
            let inner = self.inner.lock().unwrap();
            if inner.state.connection != FilmCraftConnectionState::Connected {
                return Err(
                    "FilmCraft MCP is not connected. Use filmcraft_connect first.".into(),
                );
            }

            if inner.state.mode == EditingMode::Planning {
                // Block any state-mutating tools in planning mode
                let mutating = ["command_run", "command_batch", "media_import"];
                if mutating.contains(&tool_name) {
                    return Err(format!(
                        "Planning mode: '{tool_name}' is blocked. Switch to Supervised or Autonomous mode to execute edits."
                    ));
                }
            }
        }

        // Emit progress event
        let _ = app.emit(
            "filmcraft://tool-call",
            json!({
                "tool": tool_name,
                "arguments": arguments
            }),
        );

        // Execute the tool
        let result = {
            let mut inner = self.inner.lock().unwrap();
            inner.state.current_operation = Some(format!("filmcraft_{tool_name}"));
            if let Some(conn) = inner.connection.as_mut() {
                let r = conn.call_tool(tool_name, arguments);
                inner.state.steps_taken += 1;
                r
            } else {
                Err("No active FilmCraft connection".into())
            }
        };

        // Record in history and emit completion
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state.current_operation = None;
            let step = inner.state.steps_taken;
            inner.history.push(EditingStep {
                step,
                tool: tool_name.to_string(),
                description: format!("Called filmcraft_{tool_name}"),
                success: result.is_ok(),
            });

            // Update project info if we saved/opened something
            if tool_name == "command_run" {
                if let Some(id) = arguments.get("id").and_then(|v| v.as_str()) {
                    if id == "file.save" || id == "file.saveAs" || id == "file.open" {
                        // Try to get current project info
                        if let Some(conn) = inner.connection.as_mut() {
                            if let Ok(info) = conn.call_tool("project_inspect", &json!({})) {
                                // Parse project name from result if possible
                                if let Ok(parsed) = serde_json::from_str::<Value>(&info) {
                                    inner.state.project_name = parsed["name"]
                                        .as_str()
                                        .map(|s| s.to_string());
                                    inner.state.project_path = parsed["path"]
                                        .as_str()
                                        .map(|s| s.to_string());
                                }
                            }
                        }
                    }
                }
            }

            let _ = app.emit("filmcraft://status", inner.state.clone());
        }

        result
    }

    pub fn get_history(&self) -> Vec<EditingStep> {
        self.inner.lock().unwrap().history.clone()
    }

    pub fn clear_history(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.history.clear();
        inner.state.steps_taken = 0;
    }
}

impl Default for FilmCraftService {
    fn default() -> Self {
        Self::new()
    }
}
