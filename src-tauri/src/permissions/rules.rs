// ── Risk Classification Rules ────────────────────────────────────────
//
// Determines the RiskLevel for shell commands based on the executable name.
// Filesystem / process / clipboard commands have fixed risk levels
// and don't need this module.

use super::RiskLevel;

/// Classify a shell command by its executable name and arguments.
pub fn classify_shell_risk(command: &str, _args: &[String]) -> RiskLevel {
    let cmd = command.to_lowercase();

    // Strip path and common extensions to get the bare command name
    let base = cmd
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(&cmd)
        .trim_end_matches(".exe")
        .trim_end_matches(".cmd")
        .trim_end_matches(".bat")
        .trim_end_matches(".ps1")
        .trim();

    if is_read_only(base) {
        RiskLevel::ReadOnly
    } else if is_destructive(base) {
        RiskLevel::Destructive
    } else {
        RiskLevel::Write
    }
}

fn is_read_only(base: &str) -> bool {
    matches!(
        base,
        // Windows builtins
        "dir" | "type" | "more" | "find" | "findstr" | "where"
        | "whoami" | "hostname" | "ipconfig" | "systeminfo"
        | "tasklist" | "tree" | "ver" | "echo" | "set" | "path"
        | "netstat" | "nslookup" | "ping" | "tracert"
        | "date" | "time"   // note: date/time without /s are read-only
        // Unix-like (Git Bash, WSL, etc.)
        | "ls" | "cat" | "head" | "tail" | "grep" | "wc" | "pwd"
        | "env" | "which" | "file" | "stat" | "df" | "du" | "uname"
        // Git (read operations)
        | "git" // heuristic: most git commands are safe reads
        // Node / Python / Cargo (inspections)
        | "node" | "python" | "python3" | "cargo" | "rustc" | "npm" | "pnpm"
    )
}

fn is_destructive(base: &str) -> bool {
    matches!(
        base,
        // Windows
        "del" | "erase" | "rd" | "rmdir" | "format" | "diskpart"
        | "shutdown" | "taskkill"
        // Unix-like
        | "rm" | "kill" | "killall" | "pkill"
        // PowerShell
        | "remove-item" | "clear-content" | "stop-process"
    )
}

