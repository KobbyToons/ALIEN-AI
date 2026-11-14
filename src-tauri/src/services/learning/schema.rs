use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub passive_learning: bool,
    pub scenario_questions: bool,
    pub last_question_time: Option<u64>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            passive_learning: true,
            scenario_questions: true,
            last_question_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioQuestion {
    pub id: String,
    pub category: String,
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingProfile {
    pub summary: String,
    pub last_updated: String,
}

