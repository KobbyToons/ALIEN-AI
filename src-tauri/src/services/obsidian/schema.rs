use serde::{Deserialize, Serialize};

pub const CATEGORIES: &[&str] = &[
    "Profile",
    "Preferences",
    "People",
    "Projects",
    "Daily",
    "Learned",
    "Questions",
    "Instructions",
    "Research",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub path: String,
    pub is_connected: bool,
    pub note_count: usize,
    pub categories: Vec<CategoryStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub category: String,
    pub title: String,
    pub content: String,
    pub path: String,
}

