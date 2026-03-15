//! Feedback submission client for sending feedback to the arb backend
//!
//! This module provides HTTP client functionality to submit user feedback
//! (bug reports, feature requests, general praise) to the remote backend.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_ENDPOINT: &str = "https://arb-feedback.fly.dev";
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Feedback category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackCategory {
    /// Bug report
    Bug,
    /// Feature request
    Feature,
    /// General praise
    Praise,
}

impl std::fmt::Display for FeedbackCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackCategory::Bug => write!(f, "bug"),
            FeedbackCategory::Feature => write!(f, "feature"),
            FeedbackCategory::Praise => write!(f, "praise"),
        }
    }
}

/// Feedback metadata with system information
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackMetadata {
    /// arb version
    pub version: String,
    /// Operating system info
    pub os: String,
    /// Current shell
    pub shell: String,
}

impl Default for FeedbackMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackMetadata {
    /// Create metadata from current system
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            os: get_os_info(),
            shell: get_shell_info(),
        }
    }
}

/// Feedback data to submit
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackData {
    /// Category: bug, feature, or praise
    pub category: String,
    /// Feedback content/description
    pub content: String,
    /// Optional contact email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    /// System metadata
    pub metadata: FeedbackMetadata,
    /// Honeypot field for spam protection (should be empty)
    #[serde(rename = "_hp")]
    pub honeypot: String,
}

/// Response from feedback submission
#[derive(Debug, Deserialize)]
pub struct SubmissionResponse {
    /// Feedback ID
    pub id: String,
    /// Status message
    pub status: String,
}

/// HTTP client for submitting feedback
#[derive(Debug, Clone)]
pub struct FeedbackSubmitter {
    endpoint: String,
    http_client: reqwest::Client,
}

impl FeedbackSubmitter {
    /// Create a new feedback submitter with default endpoint
    pub fn new() -> Result<Self> {
        Self::with_endpoint(DEFAULT_ENDPOINT.to_string())
    }

    /// Create a new feedback submitter with custom endpoint
    pub fn with_endpoint(endpoint: String) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            endpoint,
            http_client,
        })
    }

    /// Submit feedback to backend
    pub async fn submit(&self, feedback: FeedbackData) -> Result<SubmissionResponse> {
        let url = format!("{}/v1/feedback", self.endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&feedback)
            .send()
            .await
            .context("Failed to send feedback request")?;

        if response.status().is_success() {
            let submission: SubmissionResponse =
                response.json().await.context("Failed to parse response")?;
            Ok(submission)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Backend error ({}): {}", status, body))
        }
    }

    /// Check if backend is available
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.endpoint);
        match self.http_client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Get macOS version information
fn get_os_info() -> String {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(
            || "macOS (unknown version)".to_string(),
            |s| format!("macOS {}", s.trim()),
        )
}

/// Get current shell information
fn get_shell_info() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.split('/').next_back().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_category_display() {
        assert_eq!(FeedbackCategory::Bug.to_string(), "bug");
        assert_eq!(FeedbackCategory::Feature.to_string(), "feature");
        assert_eq!(FeedbackCategory::Praise.to_string(), "praise");
    }

    #[test]
    fn test_feedback_data_serialization() {
        let feedback = FeedbackData {
            category: "bug".to_string(),
            content: "Test feedback".to_string(),
            contact: Some("test@example.com".to_string()),
            metadata: FeedbackMetadata {
                version: "0.4.0".to_string(),
                os: "macOS 15.3".to_string(),
                shell: "zsh".to_string(),
            },
            honeypot: "".to_string(),
        };

        let json = serde_json::to_string(&feedback).unwrap();
        assert!(json.contains("bug"));
        assert!(json.contains("Test feedback"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_feedback_data_without_contact() {
        let feedback = FeedbackData {
            category: "feature".to_string(),
            content: "Test feature request".to_string(),
            contact: None,
            metadata: FeedbackMetadata {
                version: "0.4.0".to_string(),
                os: "macOS 15.3".to_string(),
                shell: "zsh".to_string(),
            },
            honeypot: "".to_string(),
        };

        let json = serde_json::to_string(&feedback).unwrap();
        // Should not contain contact field when None
        assert!(!json.contains("contact"));
    }

    #[test]
    fn test_submitter_creation() {
        let submitter = FeedbackSubmitter::new();
        assert!(submitter.is_ok());
    }
}
