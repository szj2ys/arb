//! Shared helpers used across arb CLI modules (doctor, reset, init, bench).

use std::path::PathBuf;
use std::process::{Command, Stdio};

// ANSI escape codes — single source of truth for CLI output styling.
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const PURPLE_BOLD: &str = "\x1b[1;35m";
pub const GRAY: &str = "\x1b[90m";
pub const RESET: &str = "\x1b[0m";

/// Returns the user's home directory.
pub fn home_dir() -> PathBuf {
    config::HOME_DIR.clone()
}

/// Returns the arb config directory, preferring `config::CONFIG_DIRS` if available.
pub fn config_home() -> PathBuf {
    config::CONFIG_DIRS
        .first()
        .cloned()
        .unwrap_or_else(|| home_dir().join(".config").join("arb"))
}

/// Returns the path to `.zshrc`, respecting the `ZDOTDIR` environment variable.
pub fn zshrc_path() -> PathBuf {
    if let Some(zdotdir) = std::env::var_os("ZDOTDIR") {
        PathBuf::from(zdotdir).join(".zshrc")
    } else {
        home_dir().join(".zshrc")
    }
}

/// Checks whether a command is available by running `<name> --version`.
pub fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
