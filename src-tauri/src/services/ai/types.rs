use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Gemini,
    OpenAI,
    Anthropic,
    DeepSeek,
    LmStudio,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "ollama")]
    Ollama,
}

impl ModelProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::DeepSeek => "deepseek",
            Self::LmStudio => "lmstudio",
            Self::OpenRouter => "openrouter",
            Self::Ollama => "ollama",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageInput {
    pub role: String,
    pub content: String,
}
