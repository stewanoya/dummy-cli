use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ProviderConfig;

const DEFAULT_BASE_URL: &str = "https://api.deepseek.com/v1";

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

pub struct OpenAiProvider {
    client: Client,
    config: ProviderConfig,
}

impl OpenAiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { client: Client::new(), config }
    }

    pub async fn complete(&self, system: &str, messages: Vec<String>, max_tokens: u32) -> Result<String> {
        let base_url = self.config.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        // System message first, then each element as a separate user message.
        // Keeping corpus and question in separate messages creates a stable prefix
        // for provider-side KV cache hits on repeated reads of the same files.
        let mut msgs = vec![Message { role: "system".into(), content: system.into() }];
        for content in messages {
            msgs.push(Message { role: "user".into(), content });
        }

        let req = ChatRequest {
            model: self.config.model.clone(),
            messages: msgs,
            max_tokens,
            stream: false,
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&req)
            .send()
            .await
            .context("Network error — check DUMMY_BASE_URL")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Provider error {status}: {body}");
        }

        let mut data: ChatResponse =
            resp.json().await.context("Failed to parse provider response")?;

        let choice = data
            .choices
            .drain(..)
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty response from provider"))?;

        crate::ui::stats(
            &self.config.model,
            data.usage.as_ref().map_or(0, |u| u.prompt_tokens),
            data.usage.as_ref().map_or(0, |u| u.completion_tokens),
            choice.finish_reason.as_deref().unwrap_or("unknown"),
        );

        choice.message.content.ok_or_else(|| {
            anyhow::anyhow!(
                "Model returned empty content — possibly hit token limit during reasoning. \
                 Try a higher --max-tokens value."
            )
        })
    }
}
