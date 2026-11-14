// ── Permission Gate ──────────────────────────────────────────────────
//
// Central gatekeeper.  Every system-affecting operation calls
// `gate.check(…)` before executing.
//
// Flow:
//   1. Kill switch ON  →  block immediately
//   2. ReadOnly         →  auto-approve, log silently
//   3. Write + always_allow set  →  auto-approve, log
//   4. Write / Destructive  →  emit event → wait for user → log
//   5. Timeout (30 s)  →  deny

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};

use super::{AuditEntry, AuditLog, PermissionDecision, PermissionRequest, RiskLevel};

const APPROVAL_TIMEOUT_SECS: u64 = 60;

pub struct PermissionGate {
    /// Emergency stop — blocks ALL tool execution when `true`.
    pub kill_switch: AtomicBool,

    /// In-flight approval requests: id → response channel.
    pending: Mutex<HashMap<String, oneshot::Sender<PermissionDecision>>>,

    /// Tools the user has permanently approved (never includes destructive).
    always_allow: Mutex<HashSet<String>>,

    /// Append-only action log.
    audit: Mutex<AuditLog>,
}

impl PermissionGate {
    /// Create a new gate.  `app_data_dir` is where the audit log lives.
    pub fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        let audit = AuditLog::new(app_data_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            kill_switch: AtomicBool::new(false),
            pending: Mutex::new(HashMap::new()),
            always_allow: Mutex::new(HashSet::new()),
            audit: Mutex::new(audit),
        })
    }

    // ── Public API ───────────────────────────────────────────────────

    /// Check permission.  Returns `Ok(())` if approved, `Err(reason)` if denied.
    pub async fn check(
        &self,
        app: &tauri::AppHandle,
        tool: &str,
        description: &str,
        risk: RiskLevel,
        args: serde_json::Value,
    ) -> Result<(), String> {
        // 1 ─ Kill switch
        if self.kill_switch.load(Ordering::SeqCst) {
            self.log(tool, &args, risk, PermissionDecision::KillSwitchBlocked, false, Some("Kill switch active"))
                .await;
            return Err("🛑 Kill switch is active — all operations blocked.".into());
        }

        // 2 ─ Auto-approve read-only
        if risk == RiskLevel::ReadOnly {
            self.log(tool, &args, risk, PermissionDecision::AutoApproved, true, None)
                .await;
            return Ok(());
        }

        // 3 ─ Permanent allow (never for destructive)
        if risk != RiskLevel::Destructive && self.always_allow.lock().await.contains(tool) {
            self.log(tool, &args, risk, PermissionDecision::AlwaysAllow, true, None)
                .await;
            return Ok(());
        }

        // 4 ─ Ask the user
        self.ask_user(app, tool, description, risk, args).await
    }

    /// Deliver the user's decision for a pending request.
    pub async fn respond(&self, id: &str, decision: PermissionDecision) -> Result<(), String> {
        let tx = self
            .pending
            .lock()
            .await
            .remove(id)
            .ok_or_else(|| format!("No pending request with id: {id}"))?;
        tx.send(decision)
            .map_err(|_| "Response channel already closed.".to_string())
    }

    /// Read the most recent `limit` audit entries.
    pub async fn recent_entries(&self, limit: usize) -> Result<Vec<AuditEntry>, String> {
        self.audit
            .lock()
            .await
            .read_recent(limit)
            .map_err(|e| e.to_string())
    }

    // ── Internals ────────────────────────────────────────────────────

    async fn ask_user(
        &self,
        app: &tauri::AppHandle,
        tool: &str,
        description: &str,
        risk: RiskLevel,
        args: serde_json::Value,
    ) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        // Store the channel so `respond()` can find it
        self.pending.lock().await.insert(id.clone(), tx);

        // Tell the frontend to show an approval card
        let request = PermissionRequest {
            id: id.clone(),
            tool: tool.to_string(),
            description: description.to_string(),
            risk,
            args: Some(args.clone()),
        };
        app.emit("buddy://permission-request", &request)
            .map_err(|e| format!("Failed to emit permission event: {e}"))?;

        // Wait up to APPROVAL_TIMEOUT_SECS for the user's answer
        let result = timeout(Duration::from_secs(APPROVAL_TIMEOUT_SECS), rx).await;

        // Always clean up the pending entry
        self.pending.lock().await.remove(&id);

        match result {
            // User responded in time
            Ok(Ok(decision)) => match decision {
                PermissionDecision::Approved => {
                    self.log(tool, &args, risk, decision, true, None).await;
                    Ok(())
                }
                PermissionDecision::AlwaysAllow => {
                    self.always_allow.lock().await.insert(tool.to_string());
                    self.log(tool, &args, risk, decision, true, None).await;
                    Ok(())
                }
                _ => {
                    self.log(tool, &args, risk, PermissionDecision::Denied, false, None)
                        .await;
                    Err("Permission denied by user.".into())
                }
            },
            // Channel dropped before response
            Ok(Err(_)) => {
                self.log(
                    tool, &args, risk, PermissionDecision::Denied, false,
                    Some("Channel closed"),
                ).await;
                Err("Permission request was cancelled.".into())
            }
            // Timeout
            Err(_) => {
                self.log(
                    tool, &args, risk, PermissionDecision::Denied, false,
                    Some("Timeout"),
                ).await;
                Err(format!(
                    "Permission request timed out ({APPROVAL_TIMEOUT_SECS}s)."
                ))
            }
        }
    }

    async fn log(
        &self,
        tool: &str,
        args: &serde_json::Value,
        risk: RiskLevel,
        decision: PermissionDecision,
        success: bool,
        error: Option<&str>,
    ) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            id: uuid::Uuid::new_v4().to_string(),
            tool: tool.to_string(),
            args: args.clone(),
            risk,
            decision,
            success,
            error: error.map(String::from),
        };
        if let Err(e) = self.audit.lock().await.append(&entry) {
            eprintln!("[buddy] audit log write failed: {e}");
        }
    }
}

