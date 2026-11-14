use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::State;
use tokio::sync::Mutex;

use crate::permissions::{PermissionGate, ProcessInfo, RiskLevel};

pub struct ProcessState {
    pub sys: Mutex<System>,
}

#[tauri::command]
pub async fn process_list(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    proc_state: State<'_, ProcessState>,
) -> Result<Vec<ProcessInfo>, String> {
    let desc = "List running processes";
    gate.check(&app, "process_list", desc, RiskLevel::ReadOnly, serde_json::json!({})).await?;

    let mut sys = proc_state.sys.lock().await;
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut list = Vec::new();
    for (pid, process) in sys.processes() {
        list.push(ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory_bytes: process.memory(),
            cpu_percent: process.cpu_usage(),
        });
    }

    Ok(list)
}

#[tauri::command]
pub async fn process_kill(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    proc_state: State<'_, ProcessState>,
    pid: u32,
) -> Result<(), String> {
    let desc = format!("Kill process with PID: {pid}");
    let audit_args = serde_json::json!({ "pid": pid });
    gate.check(&app, "process_kill", &desc, RiskLevel::Destructive, audit_args).await?;

    let sys = proc_state.sys.lock().await;
    let s_pid = Pid::from_u32(pid);

    if let Some(process) = sys.process(s_pid) {
        if process.kill() {
            Ok(())
        } else {
            Err(format!("Failed to kill process {pid}"))
        }
    } else {
        Err(format!("Process {pid} not found"))
    }
}

