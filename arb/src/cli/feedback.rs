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

const BASE_URL: &str = "https://github.com/szj2ys/arb/issues/new";
const DISCUSSIONS_URL: &str = "https://github.com/szj2ys/arb/discussions";

impl FeedbackCommand {
    pub async fn run(&self) -> Result<()> {
        let url = self.build_url();

        println!("arb Feedback\n============\n");
        self.print_content(&url);
        open_browser(&url);

        Ok(())
    }

    fn build_url(&self) -> String {
        match (self.bug, self.feature) {
            (true, _) => format!("{}?template=bug_report.md", BASE_URL),
            (_, true) => format!("{}?template=feature_request.md", BASE_URL),
            _ => format!("{}/choose", BASE_URL),
        }
    }

    fn print_content(&self, url: &str) {
        if self.bug {
            self.print_bug_report();
        } else if self.feature {
            println!("Creating a feature request...\n");
        } else {
            self.print_general_feedback(url);
        }
    }

    fn print_bug_report(&self) {
        let version = env!("CARGO_PKG_VERSION");
        println!(
            "Creating a bug report...\n\nSystem information:\n  arb version: {}\n  OS: {}\n",
            version,
            get_os_info()
        );
    }

    fn print_general_feedback(&self, url: &str) {
        println!(
            "We'd love to hear from you!\n\n\
            How to provide feedback:\n\n  \
            1. GitHub Issues (preferred):\n     {}\n\n  \
            2. GitHub Discussions:\n     {}\n\n  \
            3. Quick commands:\n     \
            arb feedback --bug     Create a bug report\n     \
            arb feedback --feature Create a feature request\n\n\
            When reporting issues, please include:\n  \
            - macOS version\n  \
            - arb version (run: arb --version)\n  \
            - Steps to reproduce\n  \
            - Expected vs actual behavior\n",
            url, DISCUSSIONS_URL
        );
    }
}

fn open_browser(url: &str) {
    match open::that(url) {
        Ok(_) => println!("Opening feedback page in your browser..."),
        Err(_) => println!("Please visit the feedback URL manually:\n  {}", url),
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
