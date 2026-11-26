// ── FilmCraft MCP stdio client ───────────────────────────────────────────────
//
// Implements the MCP 2024-11-05 JSON-RPC protocol over stdio.
// Manages the filmcraft-cli child process lifetime.

use std::io::Write;
use std::process::{Child, ChildStdin, ChildStdout};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

use super::types::{FilmCraftMcpTool, McpLaunchMode};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

/// Low-level synchronous stdio MCP connection to filmcraft-cli.
/// Wrapped in Mutex<Option<FilmCraftConnection>> by FilmCraftService.
pub struct FilmCraftConnection {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    pub tools: Vec<FilmCraftMcpTool>,
}

impl FilmCraftConnection {
    /// Spawn filmcraft-cli mcp and perform the MCP initialize handshake.
    pub fn connect(
        cli_path: &str,
        launch_mode: &McpLaunchMode,
        control_token: Option<&str>,
    ) -> Result<Self, String> {
        use std::process::{Command, Stdio};
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;

        let mut cmd = Command::new(cli_path);
        cmd.arg("mcp");

        match launch_mode {
            McpLaunchMode::Headless => {}
            McpLaunchMode::HeadlessDemo => { cmd.arg("--demo"); }
            McpLaunchMode::HeadlessProject(p) => { cmd.args(["--project", p]); }
            McpLaunchMode::Bridge(addr) => {
                cmd.args(["--bridge", addr]);
                if let Some(tok) = control_token {
                    cmd.env("FILMCRAFT_CONTROL_TOKEN", tok);
                }
            }
        }

        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()); // suppress FilmCraft log noise

        let mut child = cmd.spawn().map_err(|e| {
            format!("Failed to launch filmcraft-cli: {e}. Path: {cli_path}")
        })?;

        let stdin = child.stdin.take().ok_or("Could not get stdin")?;
        let stdout = child.stdout.take().ok_or("Could not get stdout")?;
        let reader = BufReader::new(stdout);

        let mut conn = FilmCraftConnection {
            child,
            stdin,
            reader,
            tools: Vec::new(),
        };

        // MCP initialize handshake
        conn.initialize()?;

        // Discover tools
        conn.tools = conn.list_tools()?;

        Ok(conn)
    }

    /// Send a JSON-RPC request and read the response line.
    fn send_request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = next_id();
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let mut line = serde_json::to_string(&request)
            .map_err(|e| format!("Serialize error: {e}"))?;
        line.push('\n');

        self.stdin
            .write_all(line.as_bytes())
            .map_err(|e| format!("Write to filmcraft-cli failed: {e}"))?;
        self.stdin.flush().map_err(|e| format!("Flush failed: {e}"))?;

        // Read response (may block — filmcraft-cli responds on the same line)
        let mut response_line = String::new();
        self.reader
            .read_line(&mut response_line)
            .map_err(|e| format!("Read from filmcraft-cli failed: {e}"))?;

        if response_line.is_empty() {
            return Err("filmcraft-cli closed its stdout (process may have exited)".into());
        }

        let response: Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Invalid JSON from filmcraft-cli: {e}\nRaw: {response_line}"))?;

        // Check for JSON-RPC error
        if let Some(err) = response.get("error") {
            return Err(format!(
                "MCP error {}: {}",
                err["code"].as_i64().unwrap_or(0),
                err["message"].as_str().unwrap_or("unknown")
            ));
        }

        Ok(response["result"].clone())
    }

    /// Send a notification (no response expected).
    fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let mut line = serde_json::to_string(&request)
            .map_err(|e| format!("Serialize error: {e}"))?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .map_err(|e| format!("Write notification failed: {e}"))?;
        self.stdin.flush().map_err(|e| format!("Flush failed: {e}"))?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), String> {
        let result = self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "ALIEN",
                    "version": "1.0"
                }
            }),
        )?;

        // Log the server info for debugging
        let server_name = result["serverInfo"]["name"].as_str().unwrap_or("unknown");
        let server_version = result["serverInfo"]["version"].as_str().unwrap_or("?");
        eprintln!("[FilmCraft MCP] Connected to {server_name} {server_version}");

        // Send initialized notification
        self.send_notification("notifications/initialized", json!({}))?;

        Ok(())
    }

    fn list_tools(&mut self) -> Result<Vec<FilmCraftMcpTool>, String> {
        let result = self.send_request("tools/list", json!({}))?;
        let tools_arr = result["tools"].as_array().ok_or("tools/list returned no array")?;

        let tools = tools_arr
            .iter()
            .map(|t| FilmCraftMcpTool {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
            })
            .collect();

        Ok(tools)
    }

    /// Call a named MCP tool with the given arguments JSON.
    pub fn call_tool(&mut self, name: &str, arguments: &Value) -> Result<String, String> {
        let result = self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )?;

        // MCP result: {"content":[{"type":"text","text":"..."}],"isError":false}
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let content = result["content"].as_array();

        let text = if let Some(arr) = content {
            arr.iter()
                .filter_map(|c| {
                    if c["type"].as_str() == Some("text") {
                        c["text"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            serde_json::to_string(&result).unwrap_or_default()
        };

        if is_error {
            Err(format!("FilmCraft tool error: {text}"))
        } else {
            Ok(text)
        }
    }

    /// Check if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,       // still running
            Ok(Some(_)) => false,   // exited
            Err(_) => false,
        }
    }
}

impl Drop for FilmCraftConnection {
    fn drop(&mut self) {
        // Send MCP shutdown gracefully, then kill if still alive
        let _ = self.send_request("shutdown", json!({}));
        let _ = self.send_notification("exit", json!({}));
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::filmcraft::FilmCraftService;

    #[test]
    fn test_filmcraft_mcp_end_to_end() {
        let cli_path = FilmCraftService::find_cli()
            .expect("filmcraft-cli.exe must be installed for integration test");
        let cli_str = cli_path.to_string_lossy().to_string();

        // 1. Connect in HeadlessDemo mode & verify initialize + tools/list
        let mut conn = FilmCraftConnection::connect(&cli_str, &McpLaunchMode::HeadlessDemo, None)
            .expect("Failed to connect to FilmCraft MCP server");

        assert!(conn.is_alive(), "filmcraft-cli process should be alive");
        assert!(!conn.tools.is_empty(), "tools/list should return FilmCraft MCP tools");

        let tool_names: Vec<&str> = conn.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"command_list"), "Missing command_list tool");
        assert!(tool_names.contains(&"command_run"), "Missing command_run tool");
        assert!(tool_names.contains(&"doc_inspect"), "Missing doc_inspect tool");
        assert!(tool_names.contains(&"sequence_inspect"), "Missing sequence_inspect tool");
        assert!(tool_names.contains(&"render_frame"), "Missing render_frame tool");
        assert!(tool_names.contains(&"media_import"), "Missing media_import tool");

        // 2. Call doc_inspect
        let doc = conn.call_tool("doc_inspect", &json!({}))
            .expect("doc_inspect should succeed");
        assert!(!doc.is_empty(), "doc_inspect returned empty output");

        // 3. Call command_list
        let cmds = conn.call_tool("command_list", &json!({"filter": "file."}))
            .expect("command_list should succeed");
        assert!(cmds.contains("file.save"), "command_list should include file.save");

        // 4. Render a preview frame from the demo project
        let frame = conn.call_tool("render_frame", &json!({"seconds": 0.5}))
            .expect("render_frame should succeed");
        assert!(!frame.is_empty(), "render_frame returned empty output");

        // 5. Save native .fcproj project to a temp path and verify it is valid JSON
        let temp_proj = std::env::temp_dir().join("alien_filmcraft_test.fcproj");
        let _ = std::fs::remove_file(&temp_proj);
        let save_res = conn.call_tool(
            "command_run",
            &json!({
                "id": "file.saveAs",
                "params": { "path": temp_proj.to_string_lossy() }
            }),
        );
        assert!(save_res.is_ok(), "file.saveAs failed: {:?}", save_res);
        assert!(temp_proj.exists(), "Saved .fcproj file should exist on disk");

        let proj_text = std::fs::read_to_string(&temp_proj).expect("Read saved .fcproj");
        let proj_json: Value = serde_json::from_str(&proj_text).expect(".fcproj must be valid JSON");
        assert_eq!(proj_json["format"].as_str(), Some("filmcraft.project"));

        let _ = std::fs::remove_file(&temp_proj);
    }
}
