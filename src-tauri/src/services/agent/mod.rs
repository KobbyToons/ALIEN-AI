use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use crate::services::ai::{AiProviders, ChatMessageInput, ModelProvider};
use crate::services::learning::LearningService;
use crate::services::mcp::{McpRegistry, McpToolCall};
use crate::services::obsidian::ObsidianVault;
use crate::services::CredentialService;

const MAX_AGENT_STEPS: usize = 8;

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

        let system_prompt = format!(
            "{}\n\n{}\n{}\nAvailable Tools Schema:\n{}\n\nWhen you want to call a tool, respond ONLY with a JSON object in this exact format:\n```json\n{{\n  \"tool\": \"tool_name\",\n  \"arguments\": {{ ... }}\n}}\n```\nDo not add conversational commentary around tool calls.\nWhen you have finished the task, or if you do not need to call a tool, reply in a warm, concise, witty conversational tone to the user.",
            system_identity, thinking_context, memory_context, tools_schema
        );

        let mut conversation: Vec<ChatMessageInput> = vec![
            ChatMessageInput {
                role: "system".into(),
                content: system_prompt,
            },
            ChatMessageInput {
                role: "user".into(),
                content: user_prompt.clone(),
            },
        ];

        let mut step = 0;
        let mut final_response = String::new();

        while step < MAX_AGENT_STEPS {
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
                conversation.push(ChatMessageInput {
                    role: "assistant".into(),
                    content: model_output,
                });
                conversation.push(ChatMessageInput {
                    role: "user".into(),
                    content: format!("Tool Output:\n{}", result.output),
                });
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
