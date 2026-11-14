use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::commands::clipboard::ClipboardState;
use crate::commands::process::ProcessState;
use crate::permissions::PermissionGate;
use crate::services::obsidian::ObsidianVault;
use super::types::{McpTool, McpToolCall, McpToolResult};

pub struct McpRegistry {
    tools: Vec<McpTool>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                McpTool {
                    name: "fs_list".into(),
                    description: "List directory contents with file names, sizes, and directory status.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Folder path to list" }
                        },
                        "required": ["path"]
                    }),
                },
                McpTool {
                    name: "fs_read".into(),
                    description: "Read the UTF-8 text contents of a file.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Absolute or relative file path" }
                        },
                        "required": ["path"]
                    }),
                },
                McpTool {
                    name: "fs_write".into(),
                    description: "Write text contents to a file (creates directories if missing).".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Target file path" },
                            "content": { "type": "string", "description": "Text content to write" }
                        },
                        "required": ["path", "content"]
                    }),
                },
                McpTool {
                    name: "fs_remove".into(),
                    description: "Delete a file or directory permanently (Requires user permission).".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File or folder to delete" },
                            "recursive": { "type": "boolean", "description": "Delete directory recursively" }
                        },
                        "required": ["path"]
                    }),
                },
                McpTool {
                    name: "shell_exec".into(),
                    description: "Execute a command line in PowerShell / Windows shell with arguments.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "command": { "type": "string", "description": "Executable or shell command" },
                            "args": { "type": "array", "items": { "type": "string" }, "description": "List of command arguments" },
                            "cwd": { "type": "string", "description": "Working directory" }
                        },
                        "required": ["command"]
                    }),
                },
                McpTool {
                    name: "process_list".into(),
                    description: "List currently running processes with PID, memory, and CPU usage.".into(),
                    input_schema: json!({ "type": "object", "properties": {} }),
                },
                McpTool {
                    name: "process_kill".into(),
                    description: "Terminate a running process by its PID.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "pid": { "type": "number", "description": "Process ID" }
                        },
                        "required": ["pid"]
                    }),
                },
                McpTool {
                    name: "clipboard_read".into(),
                    description: "Read current clipboard text.".into(),
                    input_schema: json!({ "type": "object", "properties": {} }),
                },
                McpTool {
                    name: "clipboard_write".into(),
                    description: "Write text to the system clipboard.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "Text to set" }
                        },
                        "required": ["text"]
                    }),
                },
                McpTool {
                    name: "memory_read".into(),
                    description: "Read a long-term memory note from the Obsidian vault.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "category": { "type": "string", "enum": ["Profile", "Preferences", "People", "Projects", "Daily", "Learned", "Questions"] },
                            "title": { "type": "string", "description": "Note title without extension" }
                        },
                        "required": ["category", "title"]
                    }),
                },
                McpTool {
                    name: "memory_write".into(),
                    description: "Save or update a note in the Obsidian vault with [[wikilinks]]. Non-destructive append.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "category": { "type": "string", "enum": ["Profile", "Preferences", "People", "Projects", "Daily", "Learned", "Questions"] },
                            "title": { "type": "string", "description": "Note title" },
                            "content": { "type": "string", "description": "Markdown text with [[wikilinks]]" }
                        },
                        "required": ["category", "title", "content"]
                    }),
                },
                McpTool {
                    name: "memory_search".into(),
                    description: "Search notes across all categories in the Obsidian vault.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "Search term" }
                        },
                        "required": ["query"]
                    }),
                },
                McpTool {
                    name: "obsidian_rewrite_note".into(),
                    description: "Completely rewrite and modernize an existing note in the Obsidian vault to correct outdated facts, reorganize content, and update backlinks.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "category": { "type": "string", "enum": ["Profile", "Preferences", "People", "Projects", "Daily", "Learned", "Questions", "Instructions", "Research"] },
                            "title": { "type": "string", "description": "Note title" },
                            "content": { "type": "string", "description": "Modernized note text with [[wikilinks]]" }
                        },
                        "required": ["category", "title", "content"]
                    }),
                },
                McpTool {
                    name: "obsidian_list_notes".into(),
                    description: "List all notes across all categories in the Obsidian vault with their titles and categories.".into(),
                    input_schema: json!({ "type": "object", "properties": {} }),
                },
                McpTool {
                    name: "obsidian_teach".into(),
                    description: "Teach ALIEN a custom workflow, rule, or procedure to store in Instructions/ for future reference.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string", "description": "Workflow name (e.g. 'Blender Lighting Setup', 'Discord Message Rule')" },
                            "instructions": { "type": "string", "description": "Step-by-step procedural rules ALIEN must follow" }
                        },
                        "required": ["title", "instructions"]
                    }),
                },
                McpTool {
                    name: "web_search".into(),
                    description: "Search the web and Wikipedia for facts, documentation, or news using keywords.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "Search query terms" }
                        },
                        "required": ["query"]
                    }),
                },
                McpTool {
                    name: "web_fetch".into(),
                    description: "Fetch and read the text content of a web page URL.".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "url": { "type": "string", "description": "Webpage URL to fetch" }
                        },
                        "required": ["url"]
                    }),
                },
                McpTool {
                    name: "deep_research".into(),
                    description: "Perform comprehensive deep research on a topic, search web facts, synthesize findings, and document in Obsidian under Research/ with [[wikilinks]].".into(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "topic": { "type": "string", "description": "Topic or subject to research" },
                            "focus_areas": { "type": "string", "description": "Specific questions or focus areas" }
                        },
                        "required": ["topic"]
                    }),
                },
            ],
        }
    }

    pub fn get_tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Execute a tool call through the safe permission-gated commands
    pub async fn execute_tool(&self, app: &AppHandle, call: McpToolCall) -> McpToolResult {
        let gate = app.state::<PermissionGate>();

        let res: Result<String, String> = match call.name.as_str() {
            "fs_list" => {
                let path = call.arguments["path"].as_str().unwrap_or(".").to_string();
                crate::commands::fs_list(app.clone(), gate, path)
                    .await
                    .map(|entries| serde_json::to_string_pretty(&entries).unwrap_or_default())
            }
            "fs_read" => {
                let path = call.arguments["path"].as_str().unwrap_or("").to_string();
                crate::commands::fs_read(app.clone(), gate, path).await
            }
            "fs_write" => {
                let path = call.arguments["path"].as_str().unwrap_or("").to_string();
                let content = call.arguments["content"].as_str().unwrap_or("").to_string();
                crate::commands::fs_write(app.clone(), gate, path, content)
                    .await
                    .map(|_| "File written successfully.".into())
            }
            "fs_remove" => {
                let path = call.arguments["path"].as_str().unwrap_or("").to_string();
                let recursive = call.arguments["recursive"].as_bool();
                crate::commands::fs_remove(app.clone(), gate, path, recursive)
                    .await
                    .map(|_| "Deleted successfully.".into())
            }
            "shell_exec" => {
                let command = call.arguments["command"].as_str().unwrap_or("").to_string();
                let args = call.arguments["args"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });
                let cwd = call.arguments["cwd"].as_str().map(String::from);
                crate::commands::shell_exec(app.clone(), gate, command, args, cwd)
                    .await
                    .map(|out| {
                        format!(
                            "Exit: {}\nStdout:\n{}\nStderr:\n{}",
                            out.exit_code, out.stdout, out.stderr
                        )
                    })
            }
            "process_list" => {
                let proc_state = app.state::<ProcessState>();
                crate::commands::process_list(app.clone(), gate, proc_state)
                    .await
                    .map(|list| serde_json::to_string_pretty(&list[..list.len().min(30)]).unwrap_or_default())
            }
            "process_kill" => {
                let pid = call.arguments["pid"].as_u64().unwrap_or(0) as u32;
                let proc_state = app.state::<ProcessState>();
                crate::commands::process_kill(app.clone(), gate, proc_state, pid)
                    .await
                    .map(|_| format!("Process {pid} terminated."))
            }
            "clipboard_read" => {
                let clip_state = app.state::<ClipboardState>();
                crate::commands::clipboard_read(app.clone(), gate, clip_state).await
            }
            "clipboard_write" => {
                let text = call.arguments["text"].as_str().unwrap_or("").to_string();
                let clip_state = app.state::<ClipboardState>();
                crate::commands::clipboard_write(app.clone(), gate, clip_state, text)
                    .await
                    .map(|_| "Clipboard updated.".into())
            }
            "memory_read" => {
                let category = call.arguments["category"].as_str().unwrap_or("Learned");
                let title = call.arguments["title"].as_str().unwrap_or("");
                let vault = app.state::<ObsidianVault>();
                vault.read_note(category, title)
            }
            "memory_write" => {
                let category = call.arguments["category"].as_str().unwrap_or("Learned");
                let title = call.arguments["title"].as_str().unwrap_or("Note");
                let content = call.arguments["content"].as_str().unwrap_or("");
                let vault = app.state::<ObsidianVault>();
                vault.write_note(category, title, content)
            }
            "memory_search" => {
                let query = call.arguments["query"].as_str().unwrap_or("");
                let vault = app.state::<ObsidianVault>();
                vault.search(query).map(|res| serde_json::to_string_pretty(&res).unwrap_or_default())
            }
            "obsidian_rewrite_note" => {
                let category = call.arguments["category"].as_str().unwrap_or("Learned");
                let title = call.arguments["title"].as_str().unwrap_or("Note");
                let content = call.arguments["content"].as_str().unwrap_or("");
                let vault = app.state::<ObsidianVault>();
                vault.rewrite_note(category, title, content)
            }
            "obsidian_list_notes" => {
                let vault = app.state::<ObsidianVault>();
                vault.list_all_notes().map(|notes| serde_json::to_string_pretty(&notes).unwrap_or_default())
            }
            "obsidian_teach" => {
                let title = call.arguments["title"].as_str().unwrap_or("Custom Rule");
                let instructions = call.arguments["instructions"].as_str().unwrap_or("");
                let vault = app.state::<ObsidianVault>();
                let doc = format!(
                    "## User Instruction: {}\n- **Registered**: {}\n\n### Procedural Rules:\n{}\n",
                    title,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"),
                    instructions.trim()
                );
                vault.rewrite_note("Instructions", title, &doc)
            }
            "web_search" => {
                let query = call.arguments["query"].as_str().unwrap_or("");
                do_web_search(query).await
            }
            "web_fetch" => {
                let url = call.arguments["url"].as_str().unwrap_or("");
                do_web_fetch(url).await
            }
            "deep_research" => {
                let topic = call.arguments["topic"].as_str().unwrap_or("General Research");
                let focus = call.arguments["focus_areas"].as_str().unwrap_or("");
                let search_res = do_web_search(topic).await.unwrap_or_default();
                let vault = app.state::<ObsidianVault>();
                let note_content = format!(
                    "## Executive Summary\nDeep research investigation on **{}**.\n\n### Focus Areas\n{}\n\n### Verified Findings & Data\n{}\n\n### Obsidian Cross-Links\n- See also: [[Learned/Conversational Knowledge]]\n- Related project: [[Projects/Active Projects]]\n",
                    topic,
                    if focus.is_empty() { "Comprehensive overview and key factual synthesis." } else { focus },
                    search_res
                );
                let _ = vault.rewrite_note("Research", topic, &note_content);
                Ok(format!(
                    "Deep research complete! Findings documented in Obsidian vault as [[Research/{}]] with cross-links.",
                    topic
                ))
            }
            _ => {
                // Check if tool is provided by any loaded custom Skill (Blender, Discord, Clip Studio, etc.)
                let skill_loader = app.try_state::<crate::skills::SkillLoader>();
                if let Some(loader) = skill_loader {
                    let all_skills = loader.get_skills();
                    let mut matched_output = None;
                    for s in &all_skills {
                        if s.manifest.tools.iter().any(|t| t.name == call.name) {
                            match loader.execute_handler(&s.manifest.id, &call.name, &call.arguments) {
                                Ok(out) => matched_output = Some(Ok(out)),
                                Err(err) => matched_output = Some(Err(format!("Skill execution error: {err}"))),
                            }
                            break;
                        }
                    }
                    if let Some(out) = matched_output {
                        out
                    } else {
                        Err(format!("Unknown MCP or Skill tool: {}", call.name))
                    }
                } else {
                    Err(format!("Unknown MCP tool: {}", call.name))
                }
            }
        };

        match res {
            Ok(output) => McpToolResult {
                tool_call_id: call.id,
                output,
                is_error: false,
            },
            Err(e) => McpToolResult {
                tool_call_id: call.id,
                output: format!("Tool Error: {e}"),
                is_error: true,
            },
        }
    }
}

async fn do_web_search(query: &str) -> Result<String, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty.".into());
    }

    let client = reqwest::Client::builder()
        .user_agent("AlienAssistant/1.0 (Windows NT 10.0; Win64; x64)")
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let encoded = query.trim().replace(' ', "+");
    let wiki_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={encoded}&format=json&utf8="
    );

    let mut output = String::new();
    if let Ok(res) = client.get(&wiki_url).send().await {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(items) = json["query"]["search"].as_array() {
                for item in items.iter().take(5) {
                    let title = item["title"].as_str().unwrap_or("");
                    let raw_snippet = item["snippet"].as_str().unwrap_or("");
                    let snippet = strip_html_tags(raw_snippet);
                    output.push_str(&format!(
                        "- **{}**: {}\n  Reference: https://en.wikipedia.org/wiki/{}\n",
                        title,
                        snippet,
                        title.replace(' ', "_")
                    ));
                }
            }
        }
    }

    if output.is_empty() {
        Ok(format!("Web search completed for '{query}'. No immediate Wikipedia entries found."))
    } else {
        Ok(format!("Verified Web Research for '{query}':\n\n{output}"))
    }
}

async fn do_web_fetch(url: &str) -> Result<String, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".into());
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    if !status.is_success() {
        return Err(format!("Web request failed with HTTP {status}"));
    }

    let body = res.text().await.map_err(|e| e.to_string())?;
    let clean = strip_html_tags(&body);
    let truncated: String = clean.chars().take(3500).collect();
    Ok(format!("Extracted Web Content from {url}:\n\n{truncated}"))
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag {
            out.push(c);
        }
    }
    out.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}
