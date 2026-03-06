use anyhow::Result;
use clap::Parser;

/// Open the feedback page in browser or print feedback information
#[derive(Debug, Parser, Clone)]
pub struct FeedbackCommand {
    /// Create a bug report
    #[arg(long)]
    bug: bool,

    /// Create a feature request
    #[arg(long)]
    feature: bool,
}

impl FeedbackCommand {
    pub async fn run(&self) -> Result<()> {
        let base_url = "https://github.com/szj2ys/arb/issues/new";

        // Determine the URL based on command type
        let url = if self.bug {
            format!("{}?template=bug_report.md", base_url)
        } else if self.feature {
            format!("{}?template=feature_request.md", base_url)
        } else {
            format!("{}/choose", base_url)
        };

        // Collect system info
        let version = env!("CARGO_PKG_VERSION");
        let os_info = get_os_info();

        println!("arb Feedback");
        println!("============");
        println!();

        if self.bug {
            println!("Creating a bug report...");
            println!();
            println!("System information:");
            println!("  arb version: {}", version);
            println!("  OS: {}", os_info);
            println!();
        } else if self.feature {
            println!("Creating a feature request...");
            println!();
        } else {
            println!("We'd love to hear from you!");
            println!();
            println!("How to provide feedback:");
            println!();
            println!("  1. GitHub Issues (preferred):");
            println!("     {}", url);
            println!();
            println!("  2. GitHub Discussions:");
            println!("     https://github.com/szj2ys/arb/discussions");
            println!();
            println!("  3. Quick commands:");
            println!("     arb feedback --bug     Create a bug report");
            println!("     arb feedback --feature Create a feature request");
            println!();
            println!("When reporting issues, please include:");
            println!("  - macOS version");
            println!("  - arb version (run: arb --version)");
            println!("  - Steps to reproduce");
            println!("  - Expected vs actual behavior");
            println!();
        }

        // Try to open browser
        match open::that(&url) {
            Ok(_) => {
                println!("Opening feedback page in your browser...");
            }
            Err(_) => {
                println!("Please visit the feedback URL manually:");
                println!("  {}", url);
            }
        }

        Ok(())
    }
}

/// Get macOS version information
fn get_os_info() -> String {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| format!("macOS {}", s.trim()))
        .unwrap_or_else(|| "macOS (unknown version)".to_string())
}
