//! Stats command for viewing local usage telemetry

use clap::Parser;

/// View local usage statistics
#[derive(Debug, Parser, Clone, Default)]
pub struct StatsCommand {
    /// Clear all telemetry data
    #[arg(long)]
    pub clear: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

impl StatsCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        use crate::telemetry::Telemetry;

        let telemetry = Telemetry::new()?;

        if self.clear {
            telemetry.clear()?;
            println!("Telemetry data cleared.");
            return Ok(());
        }

        if self.json {
            let events = telemetry.get_events()?;
            println!("{}", serde_json::to_string_pretty(&events)?);
            return Ok(());
        }

        let stats = telemetry.get_stats()?;
        println!("{}", stats.format());

        Ok(())
    }
}
