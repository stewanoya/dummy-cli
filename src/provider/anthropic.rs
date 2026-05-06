use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ProviderConfig;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: Option<Usage>,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

pub struct AnthropicProvider {
    client: Client,
    config: ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { client: Client::new(), config }
    }

    pub async fn complete(&self, system: &str, messages: Vec<String>, max_tokens: u32) -> Result<String> {
        let base_url = self.config.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
        let url = format!("{}/messages", base_url.trim_end_matches('/'));

        // Anthropic requires alternating user/assistant turns, so multiple user
        // messages are joined rather than sent separately.
        let content = messages.join("\n\n");

        let req = MessagesRequest {
            model: self.config.model.clone(),
            max_tokens,
            system: system.into(),
            messages: vec![Message { role: "user".into(), content }],
        };

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&req)
            .send()
            .await
            .context("Network error — check provider config")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Provider error {status}: {body}");
        }

        let data: MessagesResponse =
            resp.json().await.context("Failed to parse Anthropic response")?;

        crate::ui::stats(
            &self.config.model,
            data.usage.as_ref().map_or(0, |u| u.input_tokens),
            data.usage.as_ref().map_or(0, |u| u.output_tokens),
            data.stop_reason.as_deref().unwrap_or("unknown"),
        );

        data.content
            .into_iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Model returned empty content. Try a higher --max-tokens value."
                )
            })
    }
}
