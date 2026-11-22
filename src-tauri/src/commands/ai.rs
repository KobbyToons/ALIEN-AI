use serde::Serialize;
use tauri::{Emitter, Manager, State};

use crate::services::ai::{AiProviders, ChatMessageInput, ModelProvider};
use crate::services::obsidian::ObsidianVault;
use crate::services::CredentialService;

#[derive(Serialize)]
pub struct GoogleAccountStatus {
    pub is_connected: bool,
    pub email: Option<String>,
    pub method: String,
}

#[tauri::command]
pub fn save_api_key(provider: String, api_key: String) -> Result<(), String> {
    CredentialService::set_key(&provider, &api_key)
}

#[tauri::command]
pub fn has_api_key(provider: String) -> bool {
    CredentialService::has_key(&provider)
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    CredentialService::delete_key(&provider)
}

#[tauri::command]
pub fn get_google_account_status() -> GoogleAccountStatus {
    let email = CredentialService::get_google_account_email();
    let has_key = CredentialService::has_key("gemini");

    GoogleAccountStatus {
        is_connected: has_key,
        email,
        method: if has_key {
            "Google Account (Active)".into()
        } else {
            "Not Connected".into()
        },
    }
}

#[tauri::command]
pub fn connect_google_account() -> Result<GoogleAccountStatus, String> {
    // Open Google AI Studio / Google Account login in the user's default browser
    #[cfg(windows)]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", "https://aistudio.google.com/apikey"])
        .spawn();

    let email = CredentialService::get_google_account_email()
        .or_else(|| Some("kobbyagyemang7@gmail.com".into()));

    Ok(GoogleAccountStatus {
        is_connected: true,
        email,
        method: "Google Account (Connected)".into(),
    })
}

#[tauri::command]
pub async fn model_stream_chat(
    app: tauri::AppHandle,
    ai_state: State<'_, AiProviders>,
    request_id: String,
    provider: ModelProvider,
    model: String,
    messages: Vec<ChatMessageInput>,
    custom_endpoint: Option<String>,
) -> Result<String, String> {
    let api_key = if provider == ModelProvider::Ollama || provider == ModelProvider::LmStudio {
        None
    } else {
        match CredentialService::get_key(provider.as_str()) {
            Ok(k) => Some(k),
            Err(_) => {
                if provider == ModelProvider::Gemini {
                    // Gemini can run via local Google Account (agy.exe)
                    None
                } else {
                    return Err(format!(
                        "No API key found for {}. Please enter your key in Settings.",
                        provider.as_str()
                    ));
                }
            }
        }
    };

    let app_clone = app.clone();
    let req_id_clone = request_id.clone();

    if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
        cm.clear(&request_id);
        cm.reset_global();
    }

    // Inject customizable Foundation Identity Manual & Directives into context
    let mut enriched_messages = messages.clone();
    if let Some(vault) = app.try_state::<ObsidianVault>() {
        let identity = vault.get_system_identity();
        if !identity.is_empty() {
            if let Some(sys_msg) = enriched_messages.iter_mut().find(|m| m.role == "system") {
                sys_msg.content = format!("{}\n\n{}", identity, sys_msg.content);
            } else {
                enriched_messages.insert(0, ChatMessageInput::new("system", identity));
            }
        }
    }

    let res = ai_state
        .stream_chat(
            &provider,
            &model,
            api_key.as_deref(),
            custom_endpoint.as_deref(),
            &enriched_messages,
            move |chunk| {
                if let Some(cm) = app_clone.try_state::<crate::services::CancellationManager>() {
                    if cm.is_cancelled(&req_id_clone) {
                        return;
                    }
                }
                let _ = app_clone.emit(
                    "buddy://chat-chunk",
                    serde_json::json!({
                        "requestId": &req_id_clone,
                        "chunk": chunk
                    }),
                );
            },
        )
        .await;

    if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
        if cm.is_cancelled(&request_id) {
            return Ok("[Generation stopped by user]".into());
        }
    }

    match res {
        Ok(full_text) => {
            // Auto-record interaction and learnings into Obsidian Vault
            if let Some(vault) = app.try_state::<ObsidianVault>() {
                let last_user_msg = messages
                    .iter()
                    .rfind(|m| m.role == "user")
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                vault.record_interaction(&last_user_msg, &full_text);
            }

            let _ = app.emit(
                "buddy://chat-end",
                serde_json::json!({
                    "requestId": &request_id,
                    "fullText": &full_text
                }),
            );
            Ok(full_text)
        }
        Err(e) => {
            let _ = app.emit(
                "buddy://chat-error",
                serde_json::json!({
                    "requestId": &request_id,
                    "error": &e
                }),
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub fn stop_generation(
    app: tauri::AppHandle,
    request_id: Option<String>,
) -> Result<(), String> {
    if let Some(cm) = app.try_state::<crate::services::CancellationManager>() {
        cm.cancel(request_id.clone());
    }
    if let Some(id) = request_id {
        let _ = app.emit(
            "buddy://chat-end",
            serde_json::json!({
                "requestId": &id,
                "fullText": "[Generation stopped by user]"
            }),
        );
    }
    Ok(())
}
