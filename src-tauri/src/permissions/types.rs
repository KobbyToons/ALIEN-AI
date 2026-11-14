// ── Permission types ─────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// How dangerous is this operation?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Safe to run without asking (e.g. reading a file, listing processes).
    ReadOnly,
    /// Modifies something — ask once, user can choose "always allow".
    Write,
    /// Irreversible — always ask, no "always allow" option.
    Destructive,
}

/// How was the permission decision made?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    /// Read-only operation, approved automatically.
    AutoApproved,
    /// User clicked "Allow once".
    Approved,
    /// User clicked "Always for this tool".
    AlwaysAllow,
    /// User clicked "Deny" or request timed out.
    Denied,
    /// Global kill switch was active.
    KillSwitchBlocked,
}

/// Sent to the frontend to show an approval card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub id: String,
    pub tool: String,
    pub description: String,
    pub risk: RiskLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

/// One row in the append-only audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub risk: RiskLevel,
    pub decision: PermissionDecision,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result from a shell command.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// One entry from `fs_list`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    /// Unix timestamp in seconds, if available.
    pub modified: Option<u64>,
}

/// Summary of a running process.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
}

