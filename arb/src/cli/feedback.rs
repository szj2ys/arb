use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::feedback_submitter::{FeedbackCategory, FeedbackData, FeedbackMetadata, FeedbackSubmitter};

const BASE_URL: &str = "https://github.com/szj2ys/arb/issues/new";
const DISCUSSIONS_URL: &str = "https://github.com/szj2ys/arb/discussions";

/// Open the feedback page in browser or submit via TUI
#[derive(Debug, Parser, Clone)]
pub struct FeedbackCommand {
    /// Create a bug report (opens TUI)
    #[arg(long)]
    bug: bool,

    /// Create a feature request (opens TUI)
    #[arg(long)]
    feature: bool,

    /// Submit praise/positive feedback (opens TUI)
    #[arg(long)]
    praise: bool,

    /// Skip TUI and open GitHub directly
    #[arg(long)]
    github: bool,
}

impl FeedbackCommand {
    pub async fn run(&self) -> Result<()> {
        // If --github flag, use old behavior
        if self.github {
            return self.run_github_mode().await;
        }

        // Check if backend is available
        let submitter = FeedbackSubmitter::new()?;
        let backend_available = submitter.health_check().await.unwrap_or(false);

        if !backend_available {
            println!("⚠️  Feedback service unavailable. Falling back to GitHub...\n");
            return self.run_github_mode().await;
        }

        // Run TUI for feedback submission
        self.run_tui_mode(&submitter).await
    }

    async fn run_tui_mode(&self, submitter: &FeedbackSubmitter) -> Result<()> {
        println!("arb Feedback");
        println!("============\n");

        // Determine category
        let category = if self.bug {
            FeedbackCategory::Bug
        } else if self.feature {
            FeedbackCategory::Feature
        } else if self.praise {
            FeedbackCategory::Praise
        } else {
            // Interactive category selection
            let categories = vec!["Bug Report", "Feature Request", "General Praise"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What type of feedback do you have?")
                .items(&categories)
                .default(0)
                .interact()
                .context("Failed to get category selection")?;

            match selection {
                0 => FeedbackCategory::Bug,
                1 => FeedbackCategory::Feature,
                2 => FeedbackCategory::Praise,
                _ => FeedbackCategory::Bug, // unreachable
            }
        };

        println!("\nCategory: {}\n", category);

        // Get description based on category
        let prompt = match category {
            FeedbackCategory::Bug => "Describe the bug (what happened?)",
            FeedbackCategory::Feature => "Describe the feature (what should it do?)",
            FeedbackCategory::Praise => "What do you like about arb?",
        };

        let content: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .validate(|input: &String| {
                if input.trim().len() < 5 {
                    Err("Please provide at least 5 characters")
                } else if input.trim().len() > 2000 {
                    Err("Please keep it under 2000 characters")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get description")?;

        // Get optional contact email
        let contact: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Your email (optional, for follow-up)")
            .allow_empty(true)
            .validate(|input: &String| {
                if input.is_empty() {
                    Ok(())
                } else if input.contains('@') {
                    Ok(())
                } else {
                    Err("Please enter a valid email or leave empty")
                }
            })
            .interact_text()
            .context("Failed to get contact")?;

        let contact = if contact.trim().is_empty() {
            None
        } else {
            Some(contact.trim().to_string())
        };

        // Confirm submission
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Submit feedback?")
            .default(true)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirm {
            println!("\nCancelled. No feedback submitted.");
            return Ok(());
        }

        // Submit feedback
        println!("\nSubmitting...");

        let feedback = FeedbackData {
            category: category.to_string(),
            content: content.trim().to_string(),
            contact,
            metadata: FeedbackMetadata::new(),
            honeypot: "".to_string(),
        };

        match submitter.submit(feedback).await {
            Ok(response) => {
                println!("✅ Feedback submitted successfully!");
                println!("\nReference ID: {}", response.id);
                println!("\nThank you for helping make arb better!");
                if category == FeedbackCategory::Bug {
                    println!("\nFor complex issues, you can also open a GitHub issue:");
                    println!("  {}", BASE_URL);
                }
            }
            Err(e) => {
                println!("❌ Failed to submit feedback: {}", e);
                println!("\nWould you like to open GitHub instead? [Y/n]");

                let fallback = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Open GitHub issues")
                    .default(true)
                    .interact()
                    .unwrap_or(false);

                if fallback {
                    let url = self.build_github_url(&category, &content);
                    open_browser(&url);
                }
            }
        }

        Ok(())
    }

    async fn run_github_mode(&self) -> Result<()> {
        let url = self.build_url();

        println!("arb Feedback");
        println!("============\n");
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

    fn build_github_url(&self, category: &FeedbackCategory, content: &str) -> String {
        let template = match category {
            FeedbackCategory::Bug => "bug_report.md",
            FeedbackCategory::Feature => "feature_request.md",
            _ => "feature_request.md",
        };

        // URL-encode the content for pre-filling (basic encoding)
        let encoded = content
            .replace('\n', "%0A")
            .replace(' ', "%20")
            .replace('#', "%23")
            .replace('&', "%26")
            .replace('?', "%3F");

        format!("{}?template={}&body={}", BASE_URL, template, encoded)
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
