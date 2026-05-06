use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Which API wire format to use when talking to a provider.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiType {
    /// OpenAI-compatible /v1/chat/completions (DeepSeek, Kimi, Ollama, Groq, Together, …)
    #[default]
    OpenAi,
    /// Anthropic /v1/messages (Claude models)
    Anthropic,
}

impl std::fmt::Display for ApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiType::OpenAi => write!(f, "openai"),
            ApiType::Anthropic => write!(f, "anthropic"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    /// API wire format. Defaults to "openai".
    #[serde(default)]
    pub api_type: ApiType,
    pub api_key: String,
    /// Base URL for the provider. Defaults to the canonical URL for the api_type.
    pub base_url: Option<String>,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub default_provider: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        toml::from_str(&src).context("Config file parse error")
    }

    /// Resolve a provider config. Priority: env vars > named provider > config defaults > built-in defaults.
    pub fn resolve_provider(&self, name: Option<&str>) -> Result<ProviderConfig> {
        let provider_name = name
            .map(str::to_owned)
            .or_else(|| std::env::var("DUMMY_PROVIDER").ok())
            .or_else(|| self.default_provider.clone())
            .unwrap_or_else(|| "deepseek".to_string());

        let mut cfg = self
            .providers
            .get(&provider_name)
            .cloned()
            .unwrap_or_else(|| ProviderConfig {
                api_type: ApiType::OpenAi,
                api_key: String::new(),
                base_url: None,
                model: "deepseek-chat".to_string(),
            });

        // Env vars always override named provider fields
        if let Ok(v) = std::env::var("DUMMY_API_KEY") {
            cfg.api_key = v;
        }
        if let Ok(v) = std::env::var("DUMMY_BASE_URL") {
            cfg.base_url = Some(v);
        }
        if let Ok(v) = std::env::var("DUMMY_MODEL") {
            cfg.model = v;
        }

        if cfg.api_key.is_empty() {
            anyhow::bail!(
                "No API key found for provider '{provider_name}'.\n\
                 Set DUMMY_API_KEY or add [providers.{provider_name}] to {}",
                config_path().display()
            );
        }

        Ok(cfg)
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dummy")
        .join("config.toml")
}
