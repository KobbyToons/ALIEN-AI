pub mod agent;
pub mod ai;
pub mod clipboard;
pub mod filesystem;
pub mod learning;
pub mod obsidian;
pub mod process;
pub mod shell;
pub mod skills;
pub mod system;

pub use agent::*;
pub use ai::*;
pub use clipboard::*;
pub use filesystem::*;
pub use learning::*;
pub use obsidian::*;
pub use process::*;
pub use shell::*;
pub use skills::*;
pub use system::*;

/// Simple ping command — confirms the Rust backend is alive.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hey {name}! Buddy's backend is up and running 🚀")
}
