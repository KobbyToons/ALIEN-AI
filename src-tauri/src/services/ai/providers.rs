use std::path::PathBuf;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio::io::AsyncBufReadExt;

use super::types::{ChatMessageInput, ModelProvider};

pub struct AiProviders {
    client: Client,
}

impl AiProviders {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
        }
    }


    pub async fn stream_chat<F>(
        &self,
        provider: &ModelProvider,
        model: &str,
        api_key: Option<&str>,
        custom_endpoint: Option<&str>,
        messages: &[ChatMessageInput],
        mut on_chunk: F,
    ) -> Result<String, String>
    where
        F: FnMut(&str) + Send,
    {
        match provider {
            ModelProvider::OpenAI | ModelProvider::OpenRouter | ModelProvider::LmStudio | ModelProvider::DeepSeek => {
                let base_url = match provider {
                    ModelProvider::OpenRouter => "https://openrouter.ai/api/v1/chat/completions".to_string(),
                    ModelProvider::OpenAI => "https://api.openai.com/v1/chat/completions".to_string(),
                    ModelProvider::DeepSeek => "https://api.deepseek.com/chat/completions".to_string(),
                    ModelProvider::LmStudio => {
                        let endpoint = custom_endpoint
                            .unwrap_or("http://localhost:1234/v1")
                            .trim()
                            .trim_end_matches('/');
                        if endpoint.ends_with("/chat/completions") {
                            endpoint.to_string()
                        } else {
                            format!("{endpoint}/chat/completions")
                        }
                    }
                    _ => unreachable!(),
                };

                let mut req = self
                    .client
                    .post(&base_url)
                    .header("Content-Type", "application/json");

                if let Some(key) = api_key {
                    req = req.header("Authorization", format!("Bearer {key}"));
                } else if *provider == ModelProvider::LmStudio {
                    req = req.header("Authorization", "Bearer lm-studio");
                } else {
                    return Err(format!("API Key required for {}", provider.as_str()));
                }

                if *provider == ModelProvider::OpenRouter {
                    req = req
                        .header("HTTP-Referer", "https://github.com/alien-ai")
                        .header("X-Title", "ALIEN Assistant");
                }

                let payload = json!({
                    "model": if model.is_empty() { "default" } else { model },
                    "messages": messages,
                    "stream": true,
                });

                let res = req
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| format!("Request failed: {e}"))?;

                let status = res.status();
                if !status.is_success() {
                    let err_txt = res.text().await.unwrap_or_default();
                    return Err(format!("Provider returned error ({status}): {err_txt}"));
                }

                let mut stream = res.bytes_stream();
                let mut full_text = String::new();

                while let Some(chunk_res) = stream.next().await {
                    let chunk = chunk_res.map_err(|e| format!("Stream error: {e}"))?;
                    let text = String::from_utf8_lossy(&chunk);

                    for line in text.lines() {
                        let line = line.trim();
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                break;
                            }
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(delta) = val["choices"][0]["delta"]["content"].as_str() {
                                    full_text.push_str(delta);
                                    on_chunk(delta);
                                }
                            }
                        }
                    }
                }
                Ok(full_text)
            }

            ModelProvider::Anthropic => {
                let key = api_key.ok_or("API Key required for Anthropic")?;
                let anthropic_messages: Vec<serde_json::Value> = messages
                    .iter()
                    .filter(|m| m.role != "system")
                    .map(|m| json!({ "role": m.role, "content": m.content }))
                    .collect();

                let system_prompt = messages
                    .iter()
                    .find(|m| m.role == "system")
                    .map(|m| m.content.as_str())
                    .unwrap_or("You are ALIEN, an intelligent, advanced personal assistant.");

                let payload = json!({
                    "model": model,
                    "max_tokens": 4096,
                    "system": system_prompt,
                    "messages": anthropic_messages,
                    "stream": true
                });

                let res = self
                    .client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| format!("Anthropic request failed: {e}"))?;

                let status = res.status();
                if !status.is_success() {
                    let err = res.text().await.unwrap_or_default();
                    return Err(format!("Anthropic error ({status}): {err}"));
                }

                let mut stream = res.bytes_stream();
                let mut full_text = String::new();

                while let Some(chunk_res) = stream.next().await {
                    let chunk = chunk_res.map_err(|e| format!("Stream error: {e}"))?;
                    let text = String::from_utf8_lossy(&chunk);

                    for line in text.lines() {
                        let line = line.trim();
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                if val["type"] == "content_block_delta" {
                                    if let Some(delta) = val["delta"]["text"].as_str() {
                                        full_text.push_str(delta);
                                        on_chunk(delta);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(full_text)
            }

            ModelProvider::Gemini => {
                let clean_key = api_key
                    .map(|k| crate::services::CredentialService::extract_token_if_json(k))
                    .unwrap_or_default();

                // 1. If an explicit AIza API key was provided, use Google AI Studio REST API
                if clean_key.starts_with("AIza") {
                    let rest_model = if model.contains("3.8") || model.is_empty() || model == "default" {
                        "gemini-2.0-flash"
                    } else {
                        model
                    };

                    let system_prompt = messages
                        .iter()
                        .filter(|m| m.role == "system")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>()
                        .join("\n\n");

                    let contents: Vec<serde_json::Value> = messages
                        .iter()
                        .filter(|m| m.role != "system")
                        .map(|m| {
                            let role = if m.role == "assistant" { "model" } else { "user" };
                            json!({
                                "role": role,
                                "parts": [{ "text": m.content }]
                            })
                        })
                        .collect();

                    let mut payload_map = serde_json::Map::new();
                    payload_map.insert("contents".into(), json!(contents));

                    if !system_prompt.is_empty() {
                        payload_map.insert(
                            "system_instruction".into(),
                            json!({
                                "parts": [{ "text": system_prompt }]
                            }),
                        );
                    }

                    let payload = serde_json::Value::Object(payload_map);
                    let url = format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                        rest_model, clean_key
                    );

                    let res = self
                        .client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .header("x-goog-api-key", &clean_key)
                        .json(&payload)
                        .send()
                        .await
                        .map_err(|e| format!("Gemini API request failed: {e}"))?;

                    let status = res.status();
                    if !status.is_success() {
                        let err = res.text().await.unwrap_or_default();
                        return Err(format!("Gemini API returned ({status}): {err}"));
                    }

                    let mut stream = res.bytes_stream();
                    let mut full_text = String::new();

                    while let Some(chunk_res) = stream.next().await {
                        let chunk = chunk_res.map_err(|e| format!("Stream error: {e}"))?;
                        let text = String::from_utf8_lossy(&chunk);

                        for line in text.lines() {
                            let line = line.trim();
                            if let Some(data) = line.strip_prefix("data: ") {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(candidates) = val["candidates"].as_array() {
                                        if let Some(first) = candidates.first() {
                                            if let Some(parts) = first["content"]["parts"].as_array() {
                                                for part in parts {
                                                    if let Some(delta) = part["text"].as_str() {
                                                        full_text.push_str(delta);
                                                        on_chunk(delta);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    return Ok(full_text);
                }

                // 2. Default & Antigravity Google Account integration: Run via agy.exe
                let agy_path = dirs::home_dir()
                    .map(|h| h.join(".gemini").join("bin").join("agy.exe"))
                    .filter(|p| p.exists())
                    .unwrap_or_else(|| PathBuf::from("agy.exe"));

                // Map model name for agy
                let mapped_model = if model.contains("3.8") || model == "default" || model.is_empty() {
                    "gemini-3.8-flash-high"
                } else if model.contains("3.5") {
                    "gemini-3.8-flash-medium"
                } else if model.contains("3.1") {
                    "gemini-3.8-flash-low"
                } else if model.contains("claude") {
                    "claude-sonnet-4-6"
                } else {
                    "gemini-3.8-flash-high"
                };

                let system_prompt = messages
                    .iter()
                    .filter(|m| m.role == "system")
                    .map(|m| m.content.clone())
                    .collect::<Vec<String>>()
                    .join("\n\n");

                let mut prompt_builder = String::new();
                if !system_prompt.is_empty() {
                    prompt_builder.push_str(&format!("[System Instructions: {}]\n\n", system_prompt));
                }

                let non_sys_messages: Vec<&ChatMessageInput> = messages
                    .iter()
                    .filter(|m| m.role != "system")
                    .collect();

                if non_sys_messages.len() == 1 {
                    prompt_builder.push_str(&non_sys_messages[0].content);
                } else {
                    for m in &non_sys_messages {
                        let speaker = if m.role == "assistant" { "ALIEN" } else { "User" };
                        prompt_builder.push_str(&format!("{}: {}\n", speaker, m.content));
                    }
                    prompt_builder.push_str("ALIEN: ");
                }

                let prompt_arg = if prompt_builder.len() > 15000 {
                    prompt_builder[prompt_builder.len() - 15000..].to_string()
                } else {
                    prompt_builder
                };

                // Sanitize quotes to prevent Go's Windows command-line parser from splitting
                // argument strings on internal quotes (e.g. "The SCARLET Crystal").
                let sanitized_prompt = prompt_arg.replace('"', "'");

                let mut cmd = tokio::process::Command::new(&agy_path);
                #[cfg(windows)]
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                cmd.stdin(std::process::Stdio::null());
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());
                cmd.args([
                    "--model",
                    mapped_model,
                    "--disable-slash-commands",
                    "--dangerously-skip-permissions",
                    "--print",
                    &sanitized_prompt,
                ]);

                let mut child = cmd.spawn().map_err(|e| {
                    format!(
                        "Failed to invoke Antigravity Gemini engine: {e}. Ensure ~/.gemini/bin/agy.exe is available or provide an API key."
                    )
                })?;

                let stderr = child.stderr.take();
                let stderr_handle = tokio::spawn(async move {
                    if let Some(stderr) = stderr {
                        let mut err_reader = tokio::io::BufReader::new(stderr).lines();
                        let mut err_str = String::new();
                        while let Ok(Some(l)) = err_reader.next_line().await {
                            err_str.push_str(&l);
                            err_str.push('\n');
                        }
                        err_str
                    } else {
                        String::new()
                    }
                });

                let stdout = child.stdout.take().ok_or("Failed to capture stdout from agy")?;
                let mut reader = tokio::io::BufReader::new(stdout).lines();
                let mut full_text = String::new();

                while let Ok(Some(line)) = reader.next_line().await {
                    let chunk = format!("{line}\n");
                    full_text.push_str(&chunk);
                    on_chunk(&chunk);
                }

                let status = child.wait().await.map_err(|e| e.to_string())?;
                let stderr_output = stderr_handle.await.unwrap_or_default();

                if !status.success() {
                    let trimmed_err = stderr_output.trim();
                    if !trimmed_err.is_empty() {
                        return Err(format!("Gemini Engine: {trimmed_err}"));
                    }
                    return Err(format!("Gemini Engine exited with status: {status}"));
                }

                let trimmed = full_text.trim().to_string();
                if trimmed.is_empty() {
                    let trimmed_err = stderr_output.trim();
                    if !trimmed_err.is_empty() {
                        return Err(format!("Gemini Engine: {trimmed_err}"));
                    }
                    return Err("Gemini returned an empty response. Please check your internet connection.".into());
                } else {
                    Ok(trimmed)
                }
            }

            ModelProvider::Ollama => {
                let host = custom_endpoint.unwrap_or("http://localhost:11434");
                let url = format!("{host}/api/chat");

                let payload = json!({
                    "model": model,
                    "messages": messages,
                    "stream": true
                });

                let res = self
                    .client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| format!("Ollama request failed: {e}. Is Ollama running?"))?;

                let status = res.status();
                if !status.is_success() {
                    let err = res.text().await.unwrap_or_default();
                    return Err(format!("Ollama error ({status}): {err}"));
                }

                let mut stream = res.bytes_stream();
                let mut full_text = String::new();

                while let Some(chunk_res) = stream.next().await {
                    let chunk = chunk_res.map_err(|e| format!("Stream error: {e}"))?;
                    let text = String::from_utf8_lossy(&chunk);

                    for line in text.lines() {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(delta) = val["message"]["content"].as_str() {
                                full_text.push_str(delta);
                                on_chunk(delta);
                            }
                        }
                    }
                }
                Ok(full_text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_stream_chat() {
        let providers = AiProviders::new();
        let msgs = vec![ChatMessageInput {
            role: "user".into(),
            content: "Respond with the word \"ALIEN_ONLINE\" in one line.".into(),
        }];
        let mut chunks = Vec::new();
        let res = providers
            .stream_chat(
                &ModelProvider::Gemini,
                "gemini-3.8-flash",
                None,
                None,
                &msgs,
                |chunk| chunks.push(chunk.to_string()),
            )
            .await;
        println!("Gemini test result: {:?}", res);
        assert!(res.is_ok(), "Gemini stream_chat failed: {:?}", res.err());
        let reply = res.unwrap();
        assert!(reply.contains("ALIEN_ONLINE") || !reply.is_empty());
    }
}
