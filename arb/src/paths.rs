//! Shared filesystem path helpers used across arb CLI modules (doctor, reset, init).

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Returns the user's home directory.
pub fn home_dir() -> PathBuf {
    config::HOME_DIR.clone()
}

/// Returns the default arb config directory (`~/.config/arb`).
pub fn config_home() -> PathBuf {
    home_dir().join(".config").join("arb")
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
