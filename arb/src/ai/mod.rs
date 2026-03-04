//! AI Terminal features for Arb
//!
//! Provides LLM integration, AI command assistance, and agent capabilities.

pub mod provider;
pub use provider::*;

pub mod command;
pub use command::AiCommand;
