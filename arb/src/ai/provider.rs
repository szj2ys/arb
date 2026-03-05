//! LLM Provider abstraction for Arb AI Terminal
//!
//! Supports OpenAI-compatible APIs including:
//! - OpenAI
//! - Anthropic (via compat layer)
//! - DashScope (阿里云)
//! - Ollama (local)
//! - Custom endpoints

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// LLM request
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// LLM response
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming response chunk
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChunk {
    pub id: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct Delta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

/// LLM Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: "openai".to_string(),
            api_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4".to_string(),
            timeout_seconds: 60,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            headers: vec![],
        }
    }
}

/// Trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a chat completion request
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// Send a streaming chat completion request
    async fn chat_stream(&self, request: ChatRequest) -> Result<BoxStream<'static, Result<StreamChunk>>>;

    /// Test the connection to the provider
    async fn test_connection(&self) -> Result<()>;

    /// Get provider configuration
    fn config(&self) -> &ProviderConfig;
}

/// OpenAI-compatible provider implementation
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { config, client })
    }

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key)).unwrap(),
        );

        // Add custom headers
        for (key, value) in &self.config.headers {
            if let Ok(key) = key.parse::<reqwest::header::HeaderName>() {
                if let Ok(val) = HeaderValue::from_str(value) {
                    headers.insert(key, val);
                }
            }
        }

        headers
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.config.api_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("API error ({}): {}", status, text));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    async fn chat_stream(&self, mut request: ChatRequest) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        request.stream = Some(true);

        let url = format!("{}/chat/completions", self.config.api_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("API error ({}): {}", status, text));
        }

        let stream = response.bytes_stream();
        let mapped = stream.map(|result| {
            result.map_err(|e| anyhow!("Stream error: {}", e))
                .and_then(|bytes| {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse SSE format
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                continue;
                            }
                            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                return Ok(chunk);
                            }
                        }
                    }
                    Err(anyhow!("Failed to parse stream chunk"))
                })
        });

        Ok(Box::pin(mapped))
    }

    async fn test_connection(&self) -> Result<()> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(0.0),
            max_tokens: Some(10),
            stream: Some(false),
            tools: None,
        };

        let _ = self.chat(request).await?;
        Ok(())
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

/// Factory for creating providers
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: ProviderConfig) -> Result<Box<dyn LLMProvider>> {
        match config.name.as_str() {
            "openai" | "dashscope" | "custom" => {
                Ok(Box::new(OpenAIProvider::new(config)?))
            }
            _ => Err(anyhow!("Unknown provider: {}", config.name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ProviderConfig {
        ProviderConfig {
            name: "dashscope".to_string(),
            api_url: "https://coding.dashscope.aliyuncs.com/v1".to_string(),
            api_key: "sk-sp-3a9cf8cb9a714f67bec0f464a13bcb35".to_string(),
            model: "kimi-k2.5".to_string(),
            timeout_seconds: 60,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            headers: vec![],
        }
    }

    #[test]
    fn test_provider_factory() {
        let config = test_config();
        let provider = ProviderFactory::create(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "kimi-k2.5".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: Some(false),
            tools: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("kimi-k2.5"));
        assert!(json.contains("Hello"));
    }
}
