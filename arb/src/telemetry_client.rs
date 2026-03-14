//! Telemetry backend client for sending events to the arb telemetry service
//!
//! This module provides HTTP client functionality to send telemetry events
//! to the remote backend for aggregation and analysis.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::telemetry::{Event, EventType};

const DEFAULT_ENDPOINT: &str = "https://arb-telemetry.fly.dev";
const DEFAULT_BATCH_SIZE: usize = 10;
const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 100;

/// Configuration for telemetry backend client
#[derive(Debug, Clone)]
pub struct TelemetryClientConfig {
    /// Backend endpoint URL
    pub endpoint: String,
    /// Enable remote sending
    pub enabled: bool,
    /// Offline mode (only local storage)
    pub offline_mode: bool,
    /// Batch size before sending
    pub batch_size: usize,
    /// Flush interval in seconds
    pub flush_interval_secs: u64,
}

impl Default for TelemetryClientConfig {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.to_string(),
            enabled: !std::env::var("ARB_DISABLE_TELEMETRY").is_ok(),
            offline_mode: std::env::var("ARB_TELEMETRY_OFFLINE").is_ok(),
            batch_size: DEFAULT_BATCH_SIZE,
            flush_interval_secs: DEFAULT_FLUSH_INTERVAL_SECS,
        }
    }
}

impl TelemetryClientConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(endpoint) = std::env::var("ARB_TELEMETRY_ENDPOINT") {
            config.endpoint = endpoint;
        }

        config
    }
}

/// HTTP client for sending telemetry events to backend
#[derive(Debug, Clone)]
pub struct TelemetryClient {
    config: TelemetryClientConfig,
    device_id: String,
    http_client: reqwest::Client,
    batch_sender: Option<mpsc::UnboundedSender<Event>>,
}

/// Batch of events to send to backend
#[derive(Debug, Serialize)]
struct EventBatch {
    device_id: String,
    events: Vec<BatchEvent>,
}

/// Event in batch format
#[derive(Debug, Serialize)]
struct BatchEvent {
    timestamp: u64,
    #[serde(flatten)]
    event_type: EventType,
}

impl From<Event> for BatchEvent {
    fn from(event: Event) -> Self {
        Self {
            timestamp: event.timestamp,
            event_type: event.event_type,
        }
    }
}

/// Backend response for batch submission
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BatchResponse {
    received: usize,
}

impl TelemetryClient {
    /// Create a new telemetry client
    pub fn new(device_id: String) -> Result<Self> {
        let config = TelemetryClientConfig::from_env();
        Self::with_config(device_id, config)
    }

    /// Create client with specific config
    pub fn with_config(device_id: String, config: TelemetryClientConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            config,
            device_id,
            http_client,
            batch_sender: None,
        })
    }

    /// Start background batch sender
    pub fn start_batch_sender(&mut self) {
        if self.config.offline_mode || !self.config.enabled {
            log::debug!("Telemetry batch sender not started (offline or disabled)");
            return;
        }

        // Check if we're running in a Tokio runtime context
        if tokio::runtime::Handle::try_current().is_err() {
            log::debug!("Telemetry batch sender not started (no Tokio runtime)");
            return;
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        self.batch_sender = Some(tx);

        let client = self.http_client.clone();
        let config = self.config.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            let mut batch: Vec<Event> = Vec::with_capacity(config.batch_size);
            let mut flush_interval = interval(Duration::from_secs(config.flush_interval_secs));

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= config.batch_size {
                            let events_to_send = std::mem::replace(
                                &mut batch,
                                Vec::with_capacity(config.batch_size)
                            );
                            let client = client.clone();
                            let config = config.clone();
                            let device_id = device_id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Self::send_batch(&client, &config, &device_id, events_to_send).await {
                                    log::debug!("Failed to send telemetry batch: {}", e);
                                }
                            });
                        }
                    }
                    _ = flush_interval.tick() => {
                        if !batch.is_empty() {
                            let events_to_send = std::mem::replace(
                                &mut batch,
                                Vec::with_capacity(config.batch_size)
                            );
                            let client = client.clone();
                            let config = config.clone();
                            let device_id = device_id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Self::send_batch(&client, &config, &device_id, events_to_send).await {
                                    log::debug!("Failed to send telemetry batch: {}", e);
                                }
                            });
                        }
                    }
                    else => break,
                }
            }
        });
    }

    /// Send a single event (adds to batch queue)
    pub fn send_event(&self, event: Event) -> Result<()> {
        if self.config.offline_mode || !self.config.enabled {
            log::debug!("Telemetry event not sent (offline or disabled)");
            return Ok(());
        }

        if let Some(ref sender) = self.batch_sender {
            sender
                .send(event)
                .map_err(|_| anyhow::anyhow!("Batch sender channel closed"))?;
        } else {
            // Fallback: send immediately if batch sender not started
            let client = self.http_client.clone();
            let config = self.config.clone();
            let device_id = self.device_id.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::send_batch(&client, &config, &device_id, vec![event]).await {
                    log::debug!("Failed to send telemetry event: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Send batch of events to backend with retry logic
    async fn send_batch(
        client: &reqwest::Client,
        config: &TelemetryClientConfig,
        device_id: &str,
        events: Vec<Event>,
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let batch = EventBatch {
            device_id: device_id.to_string(),
            events: events.into_iter().map(BatchEvent::from).collect(),
        };

        let url = format!("{}/v1/events/batch", config.endpoint);
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&batch)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        log::debug!(
                            "Telemetry batch sent successfully ({} events)",
                            batch.events.len()
                        );
                        return Ok(());
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        last_error = Some(format!("HTTP {}: {}", status, body));
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }

            if attempt < MAX_RETRIES - 1 {
                let delay = RETRY_BASE_DELAY_MS * 2_u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        Err(anyhow::anyhow!(
            "Failed to send telemetry batch after {} retries: {}",
            MAX_RETRIES,
            last_error.unwrap_or_default()
        ))
    }

    /// Check if remote sending is enabled
    pub fn is_remote_enabled(&self) -> bool {
        self.config.enabled && !self.config.offline_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::InstallMethod;

    #[test]
    fn test_config_from_env_defaults() {
        let config = TelemetryClientConfig::from_env();
        assert_eq!(config.endpoint, DEFAULT_ENDPOINT);
        assert!(config.batch_size > 0);
        assert!(config.flush_interval_secs > 0);
    }

    #[test]
    fn test_batch_event_from_event() {
        let event = Event {
            device_id: "test_device".to_string(),
            timestamp: 1234567890,
            event_type: EventType::Install {
                method: InstallMethod::Homebrew,
            },
        };

        let batch_event: BatchEvent = event.into();
        assert_eq!(batch_event.timestamp, 1234567890);
        assert!(matches!(batch_event.event_type, EventType::Install { .. }));
    }

    #[test]
    fn test_telemetry_client_creation() {
        let client = TelemetryClient::new("test_device".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_send_event_offline_mode() {
        let config = TelemetryClientConfig {
            endpoint: DEFAULT_ENDPOINT.to_string(),
            enabled: true,
            offline_mode: true,
            batch_size: 10,
            flush_interval_secs: 30,
        };

        let client = TelemetryClient::with_config("test_device".to_string(), config).unwrap();
        assert!(!client.is_remote_enabled());

        let event = Event {
            device_id: "test_device".to_string(),
            timestamp: 1234567890,
            event_type: EventType::FirstLaunch {
                version: "0.4.0".to_string(),
            },
        };

        // Should not fail in offline mode
        assert!(client.send_event(event).is_ok());
    }
}
