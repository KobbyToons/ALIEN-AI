use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub tools: Vec<SkillToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolDefinition {
    pub name: String,
    pub description: String,
    pub risk: Option<String>,
    pub input_schema: serde_json::Value,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedSkill {
    pub manifest: SkillManifest,
    pub directory: String,
}

