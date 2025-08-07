// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::load_aws_config;
use crate::common::{CubConfig, Error};
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConversationRole, InferenceConfiguration, Message, PerformanceConfigLatency,
    PerformanceConfiguration, SystemContentBlock,
};
use aws_sdk_bedrockruntime::Client;

/// A convenient alias for translate client so consuming code doesn't need to add it to `Cargo.toml`
pub type LlmClient = aws_sdk_bedrockruntime::Client;

/// Creates a LLM client.
pub async fn new_llm_client(cub_config: &CubConfig) -> LlmClient {
    let aws_config = load_aws_config(cub_config).await;
    Client::new(&aws_config)
}

/// Optional options for prompting an LLM.
#[derive(Debug, Default, Copy, Clone)]
pub struct LlmOptions<'a> {
    temperature: Option<f32>,
    top_p: Option<f32>,
    optimize_latency: bool,
    system_prompt: Option<&'a str>,
}

impl<'a> LlmOptions<'a> {
    /// 0 (more conservative) to 1 (more variety).
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Only choose between the most probable tokens
    /// that sum to this probability. Lower numbers
    /// reduce the probability of rare mistakes.
    pub fn top_p(mut self, max_p: f32) -> Self {
        self.top_p = Some(max_p);
        self
    }

    /// Costs more but reduces latency.
    pub fn optimize_latency(mut self) -> Self {
        self.optimize_latency = true;
        self
    }

    /// Set the system prompt.
    pub fn system_prompt(mut self, system_prompt: &'a str) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }
}

/// Run an LLM on a prompt.
pub async fn prompt_llm(
    client: &LlmClient,
    model_id: &str,
    prompt: &str,
    options: &LlmOptions<'_>,
) -> Result<String, Error> {
    let response = client
        .converse()
        .model_id(model_id)
        .inference_config(
            InferenceConfiguration::builder()
                .set_temperature(options.temperature)
                .set_top_p(options.top_p)
                .build(),
        )
        .performance_config(
            PerformanceConfiguration::builder()
                .latency(if options.optimize_latency {
                    PerformanceConfigLatency::Optimized
                } else {
                    PerformanceConfigLatency::Standard
                })
                .build(),
        )
        .set_system(
            options
                .system_prompt
                .map(|system_prompt| vec![SystemContentBlock::Text(system_prompt.to_owned())]),
        )
        .messages(
            Message::builder()
                .role(ConversationRole::User)
                .content(ContentBlock::Text(prompt.to_owned()))
                .build()
                .unwrap(),
        )
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("prompt_llm({prompt})")))?;

    let output = response
        .output
        .ok_or_else(|| Error::String("llm returned no response".to_owned()))?;

    let output = output
        .as_message()
        .map_err(|_| Error::String("llm returned non-message".to_owned()))?;

    let output = output
        .content
        .first()
        .ok_or_else(|| Error::String("llm returned non content in message".to_owned()))?;

    let output = output
        .as_text()
        .map_err(|_| Error::String("llm returned non-text content".to_owned()))?;

    Ok(output.clone())
}
