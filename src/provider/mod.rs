use anyhow::Result;

use crate::config::{ApiType, ProviderConfig};

mod anthropic;
mod openai;

/// Dispatches to the correct API implementation based on `ProviderConfig.api_type`.
pub enum Provider {
    OpenAi(openai::OpenAiProvider),
    Anthropic(anthropic::AnthropicProvider),
}

impl Provider {
    pub fn from_config(cfg: ProviderConfig) -> Self {
        match cfg.api_type {
            ApiType::OpenAi => Provider::OpenAi(openai::OpenAiProvider::new(cfg)),
            ApiType::Anthropic => Provider::Anthropic(anthropic::AnthropicProvider::new(cfg)),
        }
    }

    pub async fn complete(&self, system: &str, messages: Vec<String>, max_tokens: u32) -> Result<String> {
        match self {
            Provider::OpenAi(p) => p.complete(system, messages, max_tokens).await,
            Provider::Anthropic(p) => p.complete(system, messages, max_tokens).await,
        }
    }
}
