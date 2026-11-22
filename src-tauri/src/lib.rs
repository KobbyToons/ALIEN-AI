// ── Buddy — Application entry point ──────────────────────────────────

mod commands;
mod permissions;
mod services;
mod skills;
mod tray;

use std::path::PathBuf;
use sysinfo::System;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};
use tokio::sync::Mutex;

use commands::clipboard::ClipboardState;
use commands::process::ProcessState;
use permissions::PermissionGate;
use services::ai::AiProviders;
use services::learning::LearningService;
use services::mcp::McpRegistry;
use services::obsidian::ObsidianVault;
use skills::SkillLoader;

/// Toggle the main widget window: show ↔ hide.
fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Plugins ──────────────────────────────────────────────
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let voice_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Backquote);
                        let kill_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F12);

                        if shortcut == &kill_shortcut {
                            // Global Emergency Kill Switch (Ctrl+Shift+F12)
                            services::input::trigger_kill_switch(Some(app));
                        } else if shortcut == &voice_shortcut {
                            // Instant voice shortcut: unhide window and start voice transcribing
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            let _ = app.emit("buddy://start-voice", ());
                        } else {
                            toggle_main_window(app);
                        }
                    }
                })
                .build(),
        )
        // ── Setup ────────────────────────────────────────────────
        .setup(|app| {
            // Register Ctrl+Space as the global summon shortcut
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
            let _ = app.global_shortcut().register(shortcut);

            // Register Ctrl+` (Ctrl+Backquote) as the instant voice shortcut
            let voice_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Backquote);
            let _ = app.global_shortcut().register(voice_shortcut);

            // Register Ctrl+Shift+F12 as the global emergency kill switch
            let kill_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F12);
            let _ = app.global_shortcut().register(kill_shortcut);

            // Create the system-tray icon with right-click menu
            tray::create_tray(app.handle())?;

            // App Data Directory for audit logs & storage
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./.buddy_data"));

            // Initialize PermissionGate state
            let gate = PermissionGate::new(app_data.clone())
                .expect("Failed to initialize permission gate");
            app.manage(gate);

            // Initialize Process, Clipboard, AI, MCP, Obsidian, Learning, and Skills services
            app.manage(ProcessState {
                sys: Mutex::new(System::new_all()),
            });
            app.manage(ClipboardState {
                clip: Mutex::new(None),
            });
            app.manage(AiProviders::new());
            app.manage(McpRegistry::new());
            app.manage(ObsidianVault::new(app_data.clone()));
            app.manage(LearningService::new());
            app.manage(services::CancellationManager::new());
            app.manage(services::OverlayService::new());
            app.manage(services::input::InputService::new());
            app.manage(services::vision::WatchModeService::new());

            let skill_loader = SkillLoader::new(app_data);
            let _ = skill_loader.reload();
            app.manage(skill_loader);

            // Show the floating orb widget on summon/launch
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        // ── IPC commands ─────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::shell_exec,
            commands::fs_read,
            commands::fs_write,
            commands::fs_list,
            commands::fs_remove,
            commands::fs_copy,
            commands::fs_move,
            commands::process_list,
            commands::process_kill,
            commands::clipboard_read,
            commands::clipboard_write,
            commands::open_url,
            commands::respond_permission,
            commands::toggle_kill_switch,
            commands::get_kill_switch,
            commands::get_audit_log,
            commands::resize_window,
            commands::save_api_key,
            commands::has_api_key,
            commands::delete_api_key,
            commands::get_google_account_status,
            commands::connect_google_account,
            commands::model_stream_chat,
            commands::stop_generation,
            commands::run_agent_task,
            commands::set_vault_path,
            commands::get_vault_info,
            commands::search_vault,
            commands::rewrite_vault_note,
            commands::list_vault_notes,
            commands::create_vault_test_note,
            commands::open_vault_in_explorer,
            commands::get_system_identity,
            commands::save_system_identity,
            commands::reset_system_identity,
            commands::get_learning_config,
            commands::update_learning_config,
            commands::get_next_question,
            commands::answer_question,
            commands::get_thinking_profile,
            commands::list_skills,
            commands::reload_skills,
            commands::list_monitors,
            commands::capture_screen,
            commands::toggle_agent_presence,
            commands::get_agent_presence,
            commands::input_move,
            commands::input_click,
            commands::input_drag,
            commands::input_scroll,
            commands::input_type,
            commands::input_press_key,
            commands::input_hotkey,
            commands::check_takeover_status,
            commands::clear_takeover_status,
            commands::emergency_kill_switch,
            commands::reset_emergency_kill_switch,
            commands::start_watch_mode,
            commands::stop_watch_mode,
            commands::get_watch_mode_status,
            commands::describe_screen,
        ])
        // ── Launch ───────────────────────────────────────────────
        .run(tauri::generate_context!())
        .expect("failed to start Buddy");
}
