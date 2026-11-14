// ── Permission Gate ──────────────────────────────────────────────────

pub mod audit;
pub mod gate;
pub mod rules;
pub mod types;

pub use audit::AuditLog;
pub use gate::PermissionGate;
pub use rules::classify_shell_risk;
pub use types::*;
