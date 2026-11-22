pub mod agent;
pub mod ai;
pub mod capture;
pub mod clipboard;
pub mod filesystem;
pub mod input;
pub mod learning;
pub mod obsidian;
pub mod overlay;
pub mod process;
pub mod shell;
pub mod skills;
pub mod system;
pub mod watch;

pub use agent::*;
pub use ai::*;
pub use capture::*;
pub use clipboard::*;
pub use filesystem::*;
pub use input::*;
pub use learning::*;
pub use obsidian::*;
pub use overlay::*;
pub use process::*;
pub use shell::*;
pub use skills::*;
pub use system::*;
pub use watch::*;

/// Simple ping command — confirms the Rust backend is alive.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hey {name}! Buddy's backend is up and running 🚀")
}
