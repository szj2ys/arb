//! Anonymous usage telemetry for arb
//!
//! This module provides lightweight, privacy-preserving usage analytics.
//! All data is stored locally by default. No PII is collected.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Telemetry event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Installation completed
    Install { method: InstallMethod },
    /// First launch of the GUI
    FirstLaunch { version: String },
    /// Shell integration initialized
    ShellInit { shell: String },
    /// Feature used
    FeatureUse { feature: String },
    /// Update check performed
    UpdateCheck { has_update: bool },
    /// Feedback submitted
    Feedback { category: String },
    /// Diagnostic run
    Diagnostic { issues_found: u32 },
}

/// Installation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallMethod {
    Homebrew,
    Dmg,
    Cargo,
    Unknown,
}

/// A telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Anonymous device identifier
    pub device_id: String,
    /// Event timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Event type and data
    #[serde(flatten)]
    pub event_type: EventType,
}

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct Telemetry {
    device_id: String,
    data_dir: PathBuf,
    enabled: bool,
}

impl Telemetry {
    /// Initialize telemetry system
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("arb")
            .join("telemetry");

        fs::create_dir_all(&data_dir)?;

        let device_id = Self::get_or_create_device_id(&data_dir)?;
        let enabled = !std::env::var("ARB_DISABLE_TELEMETRY").is_ok();

        Ok(Self {
            device_id,
            data_dir,
            enabled,
        })
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record an event
    pub fn record(&self, event_type: EventType) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = Event {
            device_id: self.device_id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type,
        };

        self.persist_event(&event)
    }

    /// Get all recorded events
    pub fn get_events(&self) -> Result<Vec<Event>> {
        let events_file = self.data_dir.join("events.jsonl");
        if !events_file.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&events_file)?;
        let events: Vec<Event> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(events)
    }

    /// Get statistics summary
    pub fn get_stats(&self) -> Result<TelemetryStats> {
        let events = self.get_events()?;
        let mut stats = TelemetryStats::default();

        for event in &events {
            match event.event_type {
                EventType::Install { .. } => stats.install_count += 1,
                EventType::FirstLaunch { .. } => stats.first_launch_count += 1,
                EventType::ShellInit { ref shell } => {
                    *stats.shell_usage.entry(shell.clone()).or_insert(0) += 1;
                }
                EventType::FeatureUse { ref feature } => {
                    *stats.feature_usage.entry(feature.clone()).or_insert(0) += 1;
                }
                EventType::UpdateCheck { has_update } => {
                    stats.update_checks += 1;
                    if has_update {
                        stats.updates_available += 1;
                    }
                }
                EventType::Feedback { ref category } => {
                    *stats.feedback_categories.entry(category.clone()).or_insert(0) += 1;
                }
                EventType::Diagnostic { issues_found } => {
                    stats.diagnostics_run += 1;
                    stats.issues_found += issues_found;
                }
            }
        }

        stats.total_events = events.len() as u64;
        Ok(stats)
    }

    /// Clear all telemetry data
    pub fn clear(&self) -> Result<()> {
        let events_file = self.data_dir.join("events.jsonl");
        if events_file.exists() {
            fs::remove_file(&events_file)?;
        }
        Ok(())
    }

    fn get_or_create_device_id(data_dir: &PathBuf) -> Result<String> {
        let id_file = data_dir.join("device_id");
        if id_file.exists() {
            return Ok(fs::read_to_string(&id_file)?.trim().to_string());
        }

        let id = format!("arb_{}", uuid::Uuid::new_v4().as_simple());
        fs::write(&id_file, &id)?;
        Ok(id)
    }

    fn persist_event(&self, event: &Event) -> Result<()> {
        let events_file = self.data_dir.join("events.jsonl");
        let json = serde_json::to_string(event)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_file)?;

        writeln!(file, "{}", json)?;
        Ok(())
    }
}

/// Statistics summary
#[derive(Debug, Default, Clone)]
pub struct TelemetryStats {
    pub total_events: u64,
    pub install_count: u64,
    pub first_launch_count: u64,
    pub shell_usage: HashMap<String, u64>,
    pub feature_usage: HashMap<String, u64>,
    pub update_checks: u64,
    pub updates_available: u64,
    pub feedback_categories: HashMap<String, u64>,
    pub diagnostics_run: u64,
    pub issues_found: u32,
}

impl TelemetryStats {
    /// Format stats for display
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("📊 arb Usage Statistics\n");
        output.push_str("======================\n\n");
        output.push_str(&format!("Total Events: {}\n", self.total_events));
        output.push_str(&format!("Installations: {}\n", self.install_count));
        output.push_str(&format!("First Launches: {}\n", self.first_launch_count));
        output.push_str(&format!("Update Checks: {}\n", self.update_checks));
        output.push_str(&format!("Diagnostics Run: {}\n", self.diagnostics_run));
        output.push_str(&format!("Issues Found: {}\n", self.issues_found));

        if !self.shell_usage.is_empty() {
            output.push_str("\nShell Usage:\n");
            for (shell, count) in &self.shell_usage {
                output.push_str(&format!("  {}: {}\n", shell, count));
            }
        }

        if !self.feature_usage.is_empty() {
            output.push_str("\nFeature Usage:\n");
            let mut features: Vec<_> = self.feature_usage.iter().collect();
            features.sort_by(|a, b| b.1.cmp(a.1));
            for (feature, count) in features.iter().take(10) {
                output.push_str(&format!("  {}: {}\n", feature, count));
            }
        }

        output
    }
}

/// Global telemetry instance (lazy-initialized)
static TELEMETRY: OnceLock<Telemetry> = OnceLock::new();

/// Initialize global telemetry
pub fn init() -> Result<()> {
    let telemetry = Telemetry::new()?;
    let _ = TELEMETRY.set(telemetry);
    Ok(())
}

/// Record an event globally
pub fn record(event_type: EventType) {
    if let Some(telemetry) = TELEMETRY.get() {
        let _ = telemetry.record(event_type);
    }
}

/// Get global telemetry instance
pub fn get() -> Option<&'static Telemetry> {
    TELEMETRY.get()
}

/// Check if telemetry is initialized
pub fn is_initialized() -> bool {
    TELEMETRY.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn should_record_event_when_called() {
        let temp_dir = TempDir::new().unwrap();
        let telemetry = Telemetry {
            device_id: "test_device".to_string(),
            data_dir: temp_dir.path().to_path_buf(),
            enabled: true,
        };

        telemetry
            .record(EventType::FirstLaunch {
                version: "1.0.0".to_string(),
            })
            .unwrap();

        let events = telemetry.get_events().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, EventType::FirstLaunch { .. }));
    }

    #[test]
    fn should_not_record_when_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let telemetry = Telemetry {
            device_id: "test_device".to_string(),
            data_dir: temp_dir.path().to_path_buf(),
            enabled: false,
        };

        telemetry
            .record(EventType::FirstLaunch {
                version: "1.0.0".to_string(),
            })
            .unwrap();

        let events = telemetry.get_events().unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn should_calculate_stats_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let telemetry = Telemetry {
            device_id: "test_device".to_string(),
            data_dir: temp_dir.path().to_path_buf(),
            enabled: true,
        };

        telemetry
            .record(EventType::Install {
                method: InstallMethod::Homebrew,
            })
            .unwrap();
        telemetry
            .record(EventType::FirstLaunch {
                version: "1.0.0".to_string(),
            })
            .unwrap();
        telemetry
            .record(EventType::ShellInit {
                shell: "zsh".to_string(),
            })
            .unwrap();

        let stats = telemetry.get_stats().unwrap();
        assert_eq!(stats.install_count, 1);
        assert_eq!(stats.first_launch_count, 1);
        assert_eq!(stats.shell_usage.get("zsh"), Some(&1));
    }
}
