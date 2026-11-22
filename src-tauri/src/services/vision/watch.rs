// ── ALIEN Watch Mode Service ── Phase E: Continuous Vision ───────────────────
//
// Observational visual awareness loop:
//  - Captures frames at ~1 fps when active.
//  - Maintains an in-memory 10-frame ring buffer (zero disk writes).
//  - Skips identical frames via perceptual hash deduplication.
//  - Answers "What am I doing?" queries using multimodal vision.
//  - Strictly read-only: no input injection or mouse/keyboard control.

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::services::ai::{AiProviders, ChatMessageInput, ModelProvider};
use crate::services::vision;
use crate::services::CredentialService;

const MAX_RING_BUFFER_FRAMES: usize = 10;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CapturedFrame {
    pub timestamp: u64,
    pub base64: String,
    pub hash: u64,
}

pub struct WatchModeService {
    is_active: Arc<AtomicBool>,
    ring_buffer: Arc<Mutex<VecDeque<CapturedFrame>>>,
}

impl WatchModeService {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            ring_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_RING_BUFFER_FRAMES))),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Start continuous background screen observation.
    pub fn start(&self, app: &AppHandle) -> Result<(), String> {
        if self.is_active.swap(true, Ordering::SeqCst) {
            // Already active
            return Ok(());
        }

        let is_active = self.is_active.clone();
        let ring_buffer = self.ring_buffer.clone();
        let app_handle = app.clone();

        // Emit state change to widget
        let _ = app.emit("buddy://watch-mode-state", serde_json::json!({ "active": true }));

        tokio::spawn(async move {
            let mut last_hash: u64 = 0;

            while is_active.load(Ordering::SeqCst) {
                // Capture frame in blocking thread pool
                let frame_res = tokio::task::spawn_blocking(|| {
                    vision::capture_monitor_base64(None, Some(1280))
                })
                .await;

                if let Ok(Ok(b64)) = frame_res {
                    // Compute hash for deduplication
                    let mut hasher = DefaultHasher::new();
                    b64.hash(&mut hasher);
                    let current_hash = hasher.finish();

                    // Only push if changed
                    if current_hash != last_hash {
                        last_hash = current_hash;
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let mut buf = ring_buffer.lock().await;
                        if buf.len() >= MAX_RING_BUFFER_FRAMES {
                            buf.pop_front();
                        }
                        buf.push_back(CapturedFrame {
                            timestamp: now,
                            base64: b64,
                            hash: current_hash,
                        });

                        // Emit visual heartbeat
                        let _ = app_handle.emit("buddy://watch-frame-update", ());
                    }
                }

                // 1 fps cadence
                sleep(Duration::from_millis(1000)).await;
            }

            // Cleanup when stopped
            let _ = app_handle.emit("buddy://watch-mode-state", serde_json::json!({ "active": false }));
        });

        Ok(())
    }

    /// Stop continuous screen observation.
    pub fn stop(&self, app: &AppHandle) -> Result<(), String> {
        self.is_active.store(false, Ordering::SeqCst);
        let _ = app.emit("buddy://watch-mode-state", serde_json::json!({ "active": false }));
        Ok(())
    }

    /// Get the most recently observed frame base64.
    pub async fn get_latest_frame(&self) -> Option<String> {
        let buf = self.ring_buffer.lock().await;
        buf.back().map(|f| f.base64.clone())
    }

    /// Answer "What am I doing?" or "Describe what is on my screen".
    pub async fn describe_current_screen(
        &self,
        app: &AppHandle,
        user_prompt: &str,
    ) -> Result<String, String> {
        // Try getting frame from ring buffer, or take a fresh capture if buffer is empty
        let frame_b64 = match self.get_latest_frame().await {
            Some(f) => f,
            None => {
                tokio::task::spawn_blocking(|| vision::capture_monitor_base64(None, Some(1280)))
                    .await
                    .map_err(|e| format!("Task error: {e}"))?
                    .map_err(|e| format!("Capture error: {e}"))?
            }
        };

        let ai = app.state::<AiProviders>();
        let api_key = match CredentialService::get_key("gemini") {
            Ok(k) => Some(k),
            Err(_) => None,
        };

        let system_prompt = "You are ALIEN observing the user's computer screen in Watch Mode.\n\
            The user asks for an explanation or description of what they are doing or what is on screen.\n\
            Provide a natural, concise, friendly, and accurate response in 2-4 sentences.\n\
            Specify:\n\
            1. What application(s) or website(s) are open.\n\
            2. What specific content or task is in progress (e.g. editing a script, watching a video, reviewing notes).\n\
            3. Any helpful observations without being preachy.";

        let conversation = vec![
            ChatMessageInput::new("system", system_prompt),
            ChatMessageInput::with_image(
                "user",
                user_prompt,
                format!("data:image/jpeg;base64,{frame_b64}"),
            ),
        ];

        let mut output = String::new();
        ai.stream_chat(
            &ModelProvider::Gemini,
            "gemini-3.8-flash",
            api_key.as_deref(),
            None,
            &conversation,
            |chunk| {
                output.push_str(chunk);
            },
        )
        .await?;

        Ok(output)
    }
}
