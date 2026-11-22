// ── ALIEN Computer Use Telemetry & Metrics ── Phase H: Polish ──────────────
//
// Tracks real-time session statistics:
//  - Total autonomous actions and steps executed.
//  - Total screen frames analyzed.
//  - Estimated multimodal vision input tokens and API costs.
//  - Average step execution and reasoning latency.

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseMetrics {
    pub total_steps: u64,
    pub total_frames: u64,
    pub estimated_tokens: u64,
    pub estimated_cost_usd: f64,
    pub average_step_ms: u64,
    pub provider: String,
}

pub struct AgentMetricsService {
    total_steps: AtomicU64,
    total_frames: AtomicU64,
    total_tokens: AtomicU64,
    total_latency_ms: AtomicU64,
}

impl AgentMetricsService {
    pub fn new() -> Self {
        Self {
            total_steps: AtomicU64::new(0),
            total_frames: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    pub fn record_step(&self, latency_ms: u64, has_frame: bool, prompt_chars: usize) {
        self.total_steps.fetch_add(1, Ordering::SeqCst);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::SeqCst);

        let mut tokens = (prompt_chars as u64) / 4;
        if has_frame {
            self.total_frames.fetch_add(1, Ordering::SeqCst);
            tokens += 258; // Standard multimodal vision token weight for 1280px image
        }
        self.total_tokens.fetch_add(tokens, Ordering::SeqCst);
    }

    pub fn get_metrics(&self) -> ComputerUseMetrics {
        let steps = self.total_steps.load(Ordering::SeqCst);
        let frames = self.total_frames.load(Ordering::SeqCst);
        let tokens = self.total_tokens.load(Ordering::SeqCst);
        let total_ms = self.total_latency_ms.load(Ordering::SeqCst);

        let avg_latency = if steps > 0 { total_ms / steps } else { 0 };
        // Gemini Flash input price is ~$0.075 per 1,000,000 tokens
        let cost = (tokens as f64 / 1_000_000.0) * 0.075;

        ComputerUseMetrics {
            total_steps: steps,
            total_frames: frames,
            estimated_tokens: tokens,
            estimated_cost_usd: (cost * 10000.0).round() / 10000.0,
            average_step_ms: avg_latency,
            provider: "Google Gemini (3.8 Flash)".to_string(),
        }
    }

    pub fn reset(&self) {
        self.total_steps.store(0, Ordering::SeqCst);
        self.total_frames.store(0, Ordering::SeqCst);
        self.total_tokens.store(0, Ordering::SeqCst);
        self.total_latency_ms.store(0, Ordering::SeqCst);
    }
}
