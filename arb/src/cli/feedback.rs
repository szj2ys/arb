use anyhow::Result;

/// Open the feedback page in browser or print feedback information
#[derive(Debug, clap::Parser, Clone)]
pub struct FeedbackCommand {}

impl FeedbackCommand {
    pub async fn run(&self, _client: wezterm_client::client::Client) -> Result<()> {
        let feedback_url = "https://github.com/szj2ys/arb/issues/new/choose";

        println!("arb Feedback");
        println!("============");
        println!();
        println!("We'd love to hear from you!");
        println!();
        println!("How to provide feedback:");
        println!();
        println!("  1. GitHub Issues (preferred):");
        println!("     {}", feedback_url);
        println!();
        println!("  2. GitHub Discussions:");
        println!("     https://github.com/szj2ys/arb/discussions");
        println!();
        println!("  3. Email:");
        println!("     Open an issue on GitHub for bugs or feature requests.");
        println!();
        println!("When reporting issues, please include:");
        println!("  - macOS version");
        println!("  - arb version (run: arb --version)");
        println!("  - Steps to reproduce");
        println!("  - Expected vs actual behavior");
        println!();

        // Try to open browser
        match open::that(feedback_url) {
            Ok(_) => {
                println!("Opening feedback page in your browser...");
            }
            Err(_) => {
                println!("Please visit the feedback URL manually.");
            }
        }

        Ok(())
    }
}
