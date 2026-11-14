// ── Audit Log ────────────────────────────────────────────────────────
//
// Append-only JSON Lines file.  Each line is one `AuditEntry`.
// The file is never truncated or edited — only appended to.

use std::path::PathBuf;

use super::AuditEntry;

pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    /// Create (or open) the audit log inside `app_data_dir`.
    pub fn new(app_data_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        Ok(Self {
            path: app_data_dir.join("audit.jsonl"),
        })
    }

    /// Append one entry.
    pub fn append(&self, entry: &AuditEntry) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let json = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    /// Read the most recent `limit` entries (reads the whole file, returns tail).
    pub fn read_recent(&self, limit: usize) -> std::io::Result<Vec<AuditEntry>> {
        let content = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let entries: Vec<AuditEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let start = entries.len().saturating_sub(limit);
        Ok(entries[start..].to_vec())
    }

    /// Full path of the log file (exposed for settings UI).
    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

