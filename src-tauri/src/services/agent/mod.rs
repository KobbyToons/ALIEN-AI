use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use crate::services::ai::{AiProviders, ChatMessageInput, ModelProvider};
use crate::services::learning::LearningService;
use crate::services::mcp::{McpRegistry, McpToolCall};
use crate::services::obsidian::ObsidianVault;
use crate::services::CredentialService;

pub mod computer_use;
pub mod metrics;
pub mod playbooks;
pub mod safety;
pub mod ui_automation;

pub use computer_use::ComputerUseEngine;
pub use metrics::AgentMetricsService;
#[allow(unused_imports)]
pub use playbooks::PlaybookManager;
pub use safety::SafetyBlocklistService;
#[allow(unused_imports)]
pub use ui_automation::UiAutomationService;

const MAX_AGENT_STEPS: usize = 25;

pub struct AgentService;

impl AgentService {
    pub async fn run_task(
        app: AppHandle,
        task_id: String,
        provider: ModelProvider,
        model: String,
        user_prompt: String,
        custom_endpoint: Option<String>,
    ) -> Result<String, String> {
        let ai = app.state::<AiProviders>();
        let mcp = app.state::<McpRegistry>();
        let vault = app.state::<ObsidianVault>();
        let learning = app.state::<LearningService>();

        // Detect Computer Use / GUI autonomous agent tasks
        let p_lower = user_prompt.to_lowercase();
        let is_gui_task = p_lower.contains("open notepad")
            || p_lower.contains("open chrome")
            || p_lower.contains("open paint")
            || p_lower.contains("draw in paint")
            || p_lower.contains("type into")
            || p_lower.contains("type in notepad")
            || p_lower.contains("shopping list")
            || p_lower.contains("computer use")
            || p_lower.contains("agent mode")
            || p_lower.contains("clip studio")
            || p_lower.contains("photoshop")
            || p_lower.contains("blender")
            || p_lower.contains("unreal")
            || (p_lower.contains("open") && (p_lower.contains("app") || p_lower.contains("browser") || p_lower.contains("search for")))
            || (p_lower.contains("click") && (p_lower.contains("screen") || p_lower.contains("button")));

        if is_gui_task {
            return ComputerUseEngine::run_agent_loop(
                app,
                task_id,
                provider,
                model,
                user_prompt,
                custom_endpoint,
            )
            .await;
        }

        let api_key = if provider == ModelProvider::Ollama || provider == ModelProvider::LmStudio {
            None
        } else {
            match CredentialService::get_key(provider.as_str()) {
                Ok(k) => Some(k),
                Err(_) => {
                    if provider == ModelProvider::Gemini {
                        None
                    } else {
                        return Err(format!(
                            "API key missing for {}. Add it in Settings.",
                            provider.as_str()
                        ));
                    }
                }
            }
        };

        let mut all_tools = mcp.get_tools().to_vec();
        if let Some(loader) = app.try_state::<crate::skills::SkillLoader>() {
            for s in loader.get_skills() {
                for t in s.manifest.tools {
                    all_tools.push(crate::services::mcp::McpTool {
                        name: t.name,
                        description: format!("[Skill: {}] {}", s.manifest.name, t.description),
                        input_schema: t.input_schema,
                    });
                }
            }
        }
        let tools_schema = serde_json::to_string(&all_tools).unwrap_or_default();
        let memory_context = vault.retrieve_context(&user_prompt);
        let thinking_context = learning.format_prompt_block(&vault);
        let system_identity = vault.get_system_identity();

        if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
            cm.clear(&task_id);
            cm.reset_global();
        }

        let system_prompt = format!(
            "{}\n\n{}\n{}\nAvailable Tools Schema:\n{}\n\nWhen you want to call a tool, respond ONLY with a JSON object in this exact format:\n```json\n{{\n  \"tool\": \"tool_name\",\n  \"arguments\": {{ ... }}\n}}\n```\nDo not add conversational commentary around tool calls.\nWhen you have finished the task, or if you do not need to call a tool, reply in a warm, concise, witty conversational tone to the user.",
            system_identity, thinking_context, memory_context, tools_schema
        );

        let mut conversation: Vec<ChatMessageInput> = vec![
            ChatMessageInput::new("system", system_prompt),
            ChatMessageInput::new("user", user_prompt.clone()),
        ];

        let mut step = 0;
        let mut final_response = String::new();

        while step < MAX_AGENT_STEPS {
            // Check if cancelled before next step
            if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
                if cm.is_cancelled(&task_id) {
                    let _ = app.emit(
                        "buddy://agent-step",
                        json!({
                            "taskId": &task_id,
                            "step": step,
                            "status": "done",
                            "description": "Agent task stopped by user."
                        }),
                    );
                    return Ok("⚠️ Agent task stopped by user.".into());
                }
            }

            step += 1;

            let _ = app.emit(
                "buddy://agent-step",
                json!({
                    "taskId": &task_id,
                    "step": step,
                    "status": "thinking",
                    "description": format!("Step {step}: Reasoning on next action...")
                }),
            );

            // Ask the model for next decision
            let mut model_output = String::new();
            let res = ai
                .stream_chat(
                    &provider,
                    &model,
                    api_key.as_deref(),
                    custom_endpoint.as_deref(),
                    &conversation,
                    |chunk| {
                        model_output.push_str(chunk);
                    },
                )
                .await;

            if let Err(e) = res {
                let _ = app.emit(
                    "buddy://agent-step",
                    json!({
                        "taskId": &task_id,
                        "step": step,
                        "status": "error",
                        "description": format!("Error: {e}")
                    }),
                );
                return Err(e);
            }

            // Check if model returned a tool call JSON
            if let Some(tool_call) = Self::parse_tool_call(&model_output) {
                // Check if cancelled right before executing the tool!
                if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
                    if cm.is_cancelled(&task_id) {
                        let _ = app.emit(
                            "buddy://agent-step",
                            json!({
                                "taskId": &task_id,
                                "step": step,
                                "status": "done",
                                "description": "Prevented tool execution — stopped by user."
                            }),
                        );
                        return Ok("⚠️ Prevented tool execution — stopped by user.".into());
                    }
                }

                let _ = app.emit(
                    "buddy://agent-step",
                    json!({
                        "taskId": &task_id,
                        "step": step,
                        "status": "executing",
                        "description": format!("Executing `{}` with args: {}", tool_call.name, tool_call.arguments)
                    }),
                );

                // Execute tool via MCP host (and safe Permission Gate)
                let result = mcp.execute_tool(&app, tool_call).await;

                // Push tool call and tool result into context
                conversation.push(ChatMessageInput::new("assistant", model_output));
                conversation.push(ChatMessageInput::new("user", format!("Tool Output:\n{}", result.output)));
            } else {
                // Done! Assistant responded with final user-facing text
                final_response = model_output;
                break;
            }
        }

        // Auto-record task learnings and interaction into Obsidian Vault
        vault.record_interaction(&user_prompt, &final_response);

        let _ = app.emit(
            "buddy://agent-step",
            json!({
                "taskId": &task_id,
                "step": step,
                "status": "done",
                "description": "Task completed successfully!"
            }),
        );

        Ok(final_response)
    }

    fn parse_tool_call(text: &str) -> Option<McpToolCall> {
        let cleaned = if let Some(start) = text.find("```json") {
            let rest = &text[start + 7..];
            if let Some(end) = rest.find("```") {
                &rest[..end]
            } else {
                rest
            }
        } else if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                &text[start..=end]
            } else {
                text
            }
        } else {
            return None;
        };

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(cleaned.trim()) {
            if let Some(tool_name) = val["tool"].as_str() {
                return Some(McpToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: tool_name.to_string(),
                    arguments: val["arguments"].clone(),
                });
            }
        }
        None
    }
}
