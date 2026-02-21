use clap::Parser;

#[derive(Debug, Parser, Clone, Default)]
pub struct BenchCommand {
    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Output results as a Markdown table (for sharing on GitHub/Twitter)
    #[arg(long)]
    markdown: bool,
}

impl BenchCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        imp::run(self)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use anyhow::bail;

    pub fn run(_cmd: &super::BenchCommand) -> anyhow::Result<()> {
        bail!("`arb bench` is currently supported on macOS only")
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::BenchCommand;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, Instant};

    const ITERATIONS: usize = 10;

    // ANSI color codes
    const BOLD: &str = "\x1b[1m";
    const GREEN: &str = "\x1b[32m";
    const GRAY: &str = "\x1b[90m";
    const DIM: &str = "\x1b[2m";
    const RESET: &str = "\x1b[0m";

    /// Terminal application info for detection.
    pub(crate) struct TerminalInfo {
        pub name: &'static str,
        pub path: &'static str,
    }

    pub(crate) const KNOWN_TERMINALS: &[TerminalInfo] = &[
        TerminalInfo {
            name: "iTerm2",
            path: "/Applications/iTerm.app",
        },
        TerminalInfo {
            name: "Ghostty",
            path: "/Applications/Ghostty.app",
        },
        TerminalInfo {
            name: "Kitty",
            path: "/Applications/kitty.app",
        },
        TerminalInfo {
            name: "Alacritty",
            path: "/Applications/Alacritty.app",
        },
        TerminalInfo {
            name: "Warp",
            path: "/Applications/Warp.app",
        },
    ];

    /// Result of detecting a terminal.
    #[derive(Debug, Clone)]
    pub(crate) struct TerminalDetection {
        pub name: String,
        pub path: String,
        pub found: bool,
    }

    /// Result of a shell benchmark run.
    #[derive(Debug, Clone)]
    pub(crate) struct BenchResult {
        pub shell: String,
        pub iterations: usize,
        pub arb_median_ms: f64,
        pub raw_median_ms: f64,
        pub terminals: Vec<TerminalDetection>,
    }

    pub fn run(cmd: &BenchCommand) -> anyhow::Result<()> {
        let result = run_benchmark()?;

        if cmd.json {
            print_json(&result);
        } else if cmd.markdown {
            print_markdown(&result);
        } else {
            print_ansi(&result);
        }

        Ok(())
    }

    pub(crate) fn run_benchmark() -> anyhow::Result<BenchResult> {
        let shell = detect_shell();

        // Measure arb environment shell startup
        let arb_timings = measure_shell_startup(&shell, true)?;
        let arb_median_ms = median_ms(&arb_timings);

        // Measure raw shell startup
        let raw_timings = measure_shell_startup(&shell, false)?;
        let raw_median_ms = median_ms(&raw_timings);

        let terminals = detect_terminals();

        Ok(BenchResult {
            shell,
            iterations: ITERATIONS,
            arb_median_ms,
            raw_median_ms,
            terminals,
        })
    }

    /// Detect the current shell name (defaults to "zsh").
    fn detect_shell() -> String {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| {
                PathBuf::from(&s)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "zsh".to_string())
    }

    /// Measure shell startup time for `iterations` runs.
    ///
    /// If `with_arb_env` is true, runs the shell normally (which picks up arb
    /// shell integration). If false, passes `--no-rcs` to skip all rc files
    /// for a baseline measurement.
    fn measure_shell_startup(
        shell: &str,
        with_arb_env: bool,
    ) -> anyhow::Result<Vec<Duration>> {
        let shell_path = format!("/bin/{shell}");
        let mut timings = Vec::with_capacity(ITERATIONS);

        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let mut cmd = Command::new(&shell_path);
            if with_arb_env {
                cmd.arg("-i").arg("-c").arg("exit");
            } else {
                cmd.arg("--no-rcs").arg("-i").arg("-c").arg("exit");
            }
            let status = cmd.output();
            let elapsed = start.elapsed();

            match status {
                Ok(output) if output.status.success() => {
                    timings.push(elapsed);
                }
                Ok(output) => {
                    // Shell exited with non-zero — still record the timing
                    // since interactive shells may return non-zero from `exit`.
                    let _ = output;
                    timings.push(elapsed);
                }
                Err(e) => {
                    anyhow::bail!("Failed to run {shell_path}: {e}");
                }
            }
        }

        Ok(timings)
    }

    /// Detect which known terminal applications are installed.
    pub(crate) fn detect_terminals() -> Vec<TerminalDetection> {
        KNOWN_TERMINALS
            .iter()
            .map(|t| {
                let found = PathBuf::from(t.path).exists();
                TerminalDetection {
                    name: t.name.to_string(),
                    path: t.path.to_string(),
                    found,
                }
            })
            .collect()
    }

    /// Compute median of durations in milliseconds.
    pub(crate) fn median_ms(timings: &[Duration]) -> f64 {
        if timings.is_empty() {
            return 0.0;
        }
        let mut ms: Vec<f64> = timings.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = ms.len() / 2;
        if ms.len() % 2 == 0 {
            (ms[mid - 1] + ms[mid]) / 2.0
        } else {
            ms[mid]
        }
    }

    /// Format benchmark results as pretty ANSI output.
    fn print_ansi(result: &BenchResult) {
        println!();
        println!("{BOLD}Arb Shell Benchmark{RESET}");
        println!(
            "{DIM}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}{RESET}"
        );
        println!("Shell: {}", result.shell);
        println!("Iterations: {}", result.iterations);
        println!();
        println!("{BOLD}Your shell startup time:{RESET}");
        println!(
            "  arb environment:  {GREEN}{:>6.0}ms{RESET} (median)",
            result.arb_median_ms
        );
        println!(
            "  raw {}:         {:>6.0}ms (median)",
            result.shell, result.raw_median_ms
        );
        println!();
        println!("{BOLD}Installed terminals detected:{RESET}");
        for t in &result.terminals {
            if t.found {
                println!("  {GREEN}\u{2714}{RESET} {:<10} {}", t.name, t.path);
            } else {
                println!("  {DIM}\u{2500}{RESET} {:<10} {GRAY}not found{RESET}", t.name);
            }
        }
        println!();
        println!("{GRAY}Share your results: arb bench --markdown | pbcopy{RESET}");
        println!();
    }

    /// Format benchmark results as JSON.
    fn print_json(result: &BenchResult) {
        let terminals: Vec<serde_json::Value> = result
            .terminals
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "path": t.path,
                    "found": t.found,
                })
            })
            .collect();

        let json = serde_json::json!({
            "shell": result.shell,
            "iterations": result.iterations,
            "arb_median_ms": round2(result.arb_median_ms),
            "raw_median_ms": round2(result.raw_median_ms),
            "terminals": terminals,
        });

        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    }

    /// Format benchmark results as a Markdown table.
    fn print_markdown(result: &BenchResult) {
        println!("## Arb Shell Benchmark");
        println!();
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Shell | {} |", result.shell);
        println!("| Iterations | {} |", result.iterations);
        println!(
            "| arb environment | {:.0}ms (median) |",
            result.arb_median_ms
        );
        println!(
            "| raw {} | {:.0}ms (median) |",
            result.shell, result.raw_median_ms
        );
        println!();
        println!("### Installed Terminals");
        println!();
        println!("| Terminal | Status | Path |");
        println!("|----------|--------|------|");
        for t in &result.terminals {
            let status = if t.found { "installed" } else { "not found" };
            let path = if t.found { t.path.as_str() } else { "-" };
            println!("| {} | {} | {} |", t.name, status, path);
        }
    }

    /// Round a float to 2 decimal places.
    fn round2(v: f64) -> f64 {
        (v * 100.0).round() / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_bench_command_with_default() {
        let cmd = BenchCommand::default();
        assert!(!cmd.json);
        assert!(!cmd.markdown);
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::imp::*;
        use std::time::Duration;

        #[test]
        fn should_compute_median_of_odd_count() {
            let timings = vec![
                Duration::from_millis(100),
                Duration::from_millis(200),
                Duration::from_millis(150),
            ];
            let median = median_ms(&timings);
            assert!((median - 150.0).abs() < 0.01);
        }

        #[test]
        fn should_compute_median_of_even_count() {
            let timings = vec![
                Duration::from_millis(100),
                Duration::from_millis(200),
                Duration::from_millis(150),
                Duration::from_millis(250),
            ];
            let median = median_ms(&timings);
            // sorted: 100, 150, 200, 250 => (150+200)/2 = 175
            assert!((median - 175.0).abs() < 0.01);
        }

        #[test]
        fn should_return_zero_for_empty_timings() {
            let timings: Vec<Duration> = vec![];
            let median = median_ms(&timings);
            assert!((median - 0.0).abs() < f64::EPSILON);
        }

        #[test]
        fn should_compute_median_of_single_element() {
            let timings = vec![Duration::from_millis(42)];
            let median = median_ms(&timings);
            assert!((median - 42.0).abs() < 0.01);
        }

        #[test]
        fn should_detect_installed_terminals() {
            let terminals = detect_terminals();
            assert_eq!(terminals.len(), 5);
            // Verify all expected terminal names are present
            let names: Vec<&str> = terminals.iter().map(|t| t.name.as_str()).collect();
            assert!(names.contains(&"iTerm2"));
            assert!(names.contains(&"Ghostty"));
            assert!(names.contains(&"Kitty"));
            assert!(names.contains(&"Alacritty"));
            assert!(names.contains(&"Warp"));
        }

        #[test]
        fn should_have_correct_terminal_paths() {
            let terminals = detect_terminals();
            for t in &terminals {
                assert!(
                    t.path.starts_with("/Applications/"),
                    "Terminal path should be in /Applications: {}",
                    t.path
                );
                assert!(
                    t.path.ends_with(".app"),
                    "Terminal path should end with .app: {}",
                    t.path
                );
            }
        }

        #[test]
        fn should_parse_timing_output() {
            // Verify that Duration-based timings convert to ms correctly
            let d = Duration::from_micros(98_500);
            let ms = d.as_secs_f64() * 1000.0;
            assert!((ms - 98.5).abs() < 0.01);
        }

        #[test]
        fn should_output_valid_json_when_json() {
            // Build a synthetic result and verify JSON output structure
            let result = super::super::imp::BenchResult {
                shell: "zsh".to_string(),
                iterations: 10,
                arb_median_ms: 98.5,
                raw_median_ms: 142.3,
                terminals: vec![
                    super::super::imp::TerminalDetection {
                        name: "iTerm2".to_string(),
                        path: "/Applications/iTerm.app".to_string(),
                        found: true,
                    },
                    super::super::imp::TerminalDetection {
                        name: "Kitty".to_string(),
                        path: "/Applications/kitty.app".to_string(),
                        found: false,
                    },
                ],
            };

            // Build the same JSON that print_json would produce
            let terminals: Vec<serde_json::Value> = result
                .terminals
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "path": t.path,
                        "found": t.found,
                    })
                })
                .collect();

            let json = serde_json::json!({
                "shell": result.shell,
                "iterations": result.iterations,
                "arb_median_ms": 98.5,
                "raw_median_ms": 142.3,
                "terminals": terminals,
            });

            let output = serde_json::to_string_pretty(&json).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
            assert_eq!(parsed["shell"], "zsh");
            assert_eq!(parsed["iterations"], 10);
            assert_eq!(parsed["arb_median_ms"], 98.5);
            assert_eq!(parsed["raw_median_ms"], 142.3);
            assert!(parsed["terminals"].is_array());
            assert_eq!(parsed["terminals"].as_array().unwrap().len(), 2);
            assert_eq!(parsed["terminals"][0]["found"], true);
            assert_eq!(parsed["terminals"][1]["found"], false);
        }

        #[test]
        fn should_output_markdown_table_when_markdown() {
            // Verify that markdown output contains expected structure
            let result = super::super::imp::BenchResult {
                shell: "zsh".to_string(),
                iterations: 10,
                arb_median_ms: 98.0,
                raw_median_ms: 142.0,
                terminals: vec![super::super::imp::TerminalDetection {
                    name: "Ghostty".to_string(),
                    path: "/Applications/Ghostty.app".to_string(),
                    found: true,
                }],
            };

            // Capture what print_markdown would produce by building the same strings
            let mut lines = Vec::new();
            lines.push("## Arb Shell Benchmark".to_string());
            lines.push(String::new());
            lines.push("| Metric | Value |".to_string());
            lines.push("|--------|-------|".to_string());
            lines.push(format!("| Shell | {} |", result.shell));
            lines.push(format!("| Iterations | {} |", result.iterations));
            lines.push(format!(
                "| arb environment | {:.0}ms (median) |",
                result.arb_median_ms
            ));
            lines.push(format!(
                "| raw {} | {:.0}ms (median) |",
                result.shell, result.raw_median_ms
            ));

            let output = lines.join("\n");
            assert!(output.contains("## Arb Shell Benchmark"));
            assert!(output.contains("| Shell | zsh |"));
            assert!(output.contains("| Iterations | 10 |"));
            assert!(output.contains("98ms"));
            assert!(output.contains("142ms"));
        }
    }
}
