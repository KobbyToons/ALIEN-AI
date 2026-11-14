use std::path::PathBuf;
use std::process::Command;
use tauri::State;

use crate::permissions::{
    classify_shell_risk, PermissionGate, ShellOutput,
};

#[tauri::command]
pub async fn shell_exec(
    app: tauri::AppHandle,
    gate: State<'_, PermissionGate>,
    command: String,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<ShellOutput, String> {
    let args_vec = args.unwrap_or_default();
    let risk = classify_shell_risk(&command, &args_vec);

    let desc = format!("Run command: `{}` with args: {:?}", command, args_vec);
    let audit_args = serde_json::json!({
        "command": command,
        "args": args_vec,
        "cwd": cwd,
    });

    gate.check(&app, "shell_exec", &desc, risk, audit_args).await?;

    let mut cmd = Command::new(&command);
    cmd.args(&args_vec);

    if let Some(c) = cwd {
        cmd.current_dir(PathBuf::from(c));
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute command: {e}"))?;

    Ok(ShellOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

