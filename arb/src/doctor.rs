use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Parser, Clone, Default)]
pub struct DoctorCommand {}

impl DoctorCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        imp::run()
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use anyhow::bail;

    pub fn run() -> anyhow::Result<()> {
        bail!("`arb doctor` is currently supported on macOS only")
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;

    // ANSI color codes
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const BOLD: &str = "\x1b[1m";
    const GRAY: &str = "\x1b[90m";
    const RESET: &str = "\x1b[0m";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum CheckStatus {
        Pass,
        Fail,
        Warn,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct CheckResult {
        pub name: String,
        pub status: CheckStatus,
        pub message: String,
        pub fix: Option<String>,
    }

    fn print_result(result: &CheckResult) {
        let (icon, color) = match result.status {
            CheckStatus::Pass => ("\u{2714}", GREEN), // ✔
            CheckStatus::Fail => ("\u{2718}", RED),   // ✘
            CheckStatus::Warn => ("\u{26a0}", YELLOW), // ⚠
        };
        println!("  {color}{icon}{RESET}  {BOLD}{}{RESET}: {}", result.name, result.message);
        if let Some(ref fix) = result.fix {
            println!("     {GRAY}{fix}{RESET}");
        }
    }

    pub fn run() -> anyhow::Result<()> {
        println!();
        println!("{BOLD}Arb Doctor{RESET}");
        println!("{GRAY}Diagnosing your Arb setup...{RESET}");
        println!();

        let results = run_all_checks();

        for result in &results {
            print_result(result);
        }

        println!();

        let pass_count = results.iter().filter(|r| r.status == CheckStatus::Pass).count();
        let fail_count = results.iter().filter(|r| r.status == CheckStatus::Fail).count();
        let warn_count = results.iter().filter(|r| r.status == CheckStatus::Warn).count();

        if fail_count == 0 && warn_count == 0 {
            println!("{GREEN}{BOLD}All {pass_count} checks passed.{RESET}");
        } else {
            println!(
                "{BOLD}{pass_count} passed{RESET}, \
                 {YELLOW}{warn_count} warning(s){RESET}, \
                 {RED}{fail_count} failed{RESET}"
            );
        }
        println!();

        Ok(())
    }

    pub(crate) fn run_all_checks() -> Vec<CheckResult> {
        let mut results = Vec::new();
        results.push(check_shell_integration());
        results.push(check_starship());
        results.push(check_delta());
        results.push(check_user_config());
        results.push(check_app_bundle());
        results.push(check_version());
        results.extend(check_zsh_plugins());
        results
    }

    fn home_dir() -> PathBuf {
        config::HOME_DIR.clone()
    }

    fn config_home() -> PathBuf {
        config::CONFIG_DIRS
            .first()
            .cloned()
            .unwrap_or_else(|| home_dir().join(".config").join("arb"))
    }

    fn zshrc_path() -> PathBuf {
        if let Some(zdotdir) = std::env::var_os("ZDOTDIR") {
            PathBuf::from(zdotdir).join(".zshrc")
        } else {
            home_dir().join(".zshrc")
        }
    }

    // -- Check 1: Shell integration --

    pub(crate) fn check_shell_integration() -> CheckResult {
        let arb_zsh = config_home().join("zsh").join("arb.zsh");
        if !arb_zsh.exists() {
            return CheckResult {
                name: "Shell integration".into(),
                status: CheckStatus::Fail,
                message: format!("{} not found", arb_zsh.display()),
                fix: Some("Run `arb init` to install shell integration".into()),
            };
        }

        let zshrc = zshrc_path();
        if !zshrc.exists() {
            return CheckResult {
                name: "Shell integration".into(),
                status: CheckStatus::Warn,
                message: format!(
                    "{} exists but {} not found",
                    arb_zsh.display(),
                    zshrc.display()
                ),
                fix: Some(format!(
                    "Create {} and source arb.zsh from it",
                    zshrc.display()
                )),
            };
        }

        match std::fs::read_to_string(&zshrc) {
            Ok(content) => {
                if content.contains("arb/zsh/arb.zsh") {
                    CheckResult {
                        name: "Shell integration".into(),
                        status: CheckStatus::Pass,
                        message: "arb.zsh exists and is sourced in .zshrc".into(),
                        fix: None,
                    }
                } else {
                    CheckResult {
                        name: "Shell integration".into(),
                        status: CheckStatus::Fail,
                        message: format!(
                            "{} exists but not sourced in {}",
                            arb_zsh.display(),
                            zshrc.display()
                        ),
                        fix: Some("Run `arb init` to restore shell integration".into()),
                    }
                }
            }
            Err(_) => CheckResult {
                name: "Shell integration".into(),
                status: CheckStatus::Warn,
                message: format!("Could not read {}", zshrc.display()),
                fix: Some("Check file permissions on your .zshrc".into()),
            },
        }
    }

    // -- Check 2: Starship --

    pub(crate) fn check_starship() -> CheckResult {
        let starship = config_home().join("zsh").join("bin").join("starship");
        if !starship.exists() {
            return CheckResult {
                name: "Starship".into(),
                status: CheckStatus::Fail,
                message: format!("{} not found", starship.display()),
                fix: Some("Run `arb init` to install the bundled starship binary".into()),
            };
        }

        // Check if executable
        match std::fs::metadata(&starship) {
            Ok(meta) => {
                use std::os::unix::fs::PermissionsExt;
                if meta.permissions().mode() & 0o111 != 0 {
                    CheckResult {
                        name: "Starship".into(),
                        status: CheckStatus::Pass,
                        message: "Bundled starship binary is present and executable".into(),
                        fix: None,
                    }
                } else {
                    CheckResult {
                        name: "Starship".into(),
                        status: CheckStatus::Fail,
                        message: format!("{} exists but is not executable", starship.display()),
                        fix: Some(format!("Run: chmod +x {}", starship.display())),
                    }
                }
            }
            Err(_) => CheckResult {
                name: "Starship".into(),
                status: CheckStatus::Warn,
                message: format!("Could not read metadata for {}", starship.display()),
                fix: None,
            },
        }
    }

    // -- Check 3: Delta --

    pub(crate) fn check_delta() -> CheckResult {
        let in_path = command_exists("delta");
        let git_pager = git_config_get("core.pager")
            .map(|v| v.trim().to_string())
            .unwrap_or_default();
        let has_git_config = git_pager == "delta";

        match (in_path, has_git_config) {
            (true, true) => CheckResult {
                name: "Delta".into(),
                status: CheckStatus::Pass,
                message: "delta is in PATH and configured as git pager".into(),
                fix: None,
            },
            (true, false) => CheckResult {
                name: "Delta".into(),
                status: CheckStatus::Warn,
                message: "delta is in PATH but not set as git core.pager".into(),
                fix: Some("Run `arb init` to configure delta as git pager".into()),
            },
            (false, _) => CheckResult {
                name: "Delta".into(),
                status: CheckStatus::Warn,
                message: "delta not found in PATH".into(),
                fix: Some("Run `arb init` to install delta via shell integration".into()),
            },
        }
    }

    // -- Check 4: User config --

    pub(crate) fn check_user_config() -> CheckResult {
        let config_path = config::CONFIG_DIRS
            .first()
            .cloned()
            .unwrap_or_else(|| config_home())
            .join("arb.lua");

        if !config_path.exists() {
            return CheckResult {
                name: "User config".into(),
                status: CheckStatus::Warn,
                message: format!("{} not found", config_path.display()),
                fix: Some("Run `arb config` to create a default configuration".into()),
            };
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                let lua = match config::lua::make_lua_context(&config_path) {
                    Ok(lua) => lua,
                    Err(err) => {
                        return CheckResult {
                            name: "User config".into(),
                            status: CheckStatus::Warn,
                            message: format!("Could not create Lua context: {err:#}"),
                            fix: None,
                        };
                    }
                };

                // Parse/compile without executing.
                // This catches syntax errors without running user code.
                let chunk = lua
                    .load(content.trim_start_matches('\u{FEFF}'))
                    .set_name(config_path.to_string_lossy());

                // `into_function()` returns a Lua function that borrows `lua`.
                // We don't need the function itself, only whether compilation succeeds.
                let parse_result = chunk.into_function().map(|_| ());

                match parse_result {
                    Ok(()) => {
                        if content.contains("return") {
                            CheckResult {
                                name: "User config".into(),
                                status: CheckStatus::Pass,
                                message: format!("{} exists and parses", config_path.display()),
                                fix: None,
                            }
                        } else {
                            CheckResult {
                                name: "User config".into(),
                                status: CheckStatus::Warn,
                                message: format!(
                                    "{} parses but may not return a config",
                                    config_path.display()
                                ),
                                fix: Some("Ensure arb.lua ends with `return config`".into()),
                            }
                        }
                    }
                    Err(err) => CheckResult {
                        name: "User config".into(),
                        status: CheckStatus::Fail,
                        message: format!("{} failed to parse: {err:#}", config_path.display()),
                        fix: Some("Fix Lua syntax errors (open with `arb config`)".into()),
                    },
                }
            }
            Err(e) => CheckResult {
                name: "User config".into(),
                status: CheckStatus::Fail,
                message: format!("Could not read {}: {}", config_path.display(), e),
                fix: Some("Check file permissions on arb.lua".into()),
            },
        }
    }

    // -- Check 5: App bundle --

    pub(crate) fn check_app_bundle() -> CheckResult {
        let candidates = [
            PathBuf::from("/Applications/Arb.app"),
            home_dir().join("Applications").join("Arb.app"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return CheckResult {
                    name: "App bundle".into(),
                    status: CheckStatus::Pass,
                    message: format!("Found {}", candidate.display()),
                    fix: None,
                };
            }
        }

        CheckResult {
            name: "App bundle".into(),
            status: CheckStatus::Fail,
            message: "Arb.app not found in /Applications or ~/Applications".into(),
            fix: Some("Install Arb.app to /Applications".into()),
        }
    }

    // -- Check 6: Version --

    pub(crate) fn check_version() -> CheckResult {
        let arb_ver = config::arb_version();

        let config_version_path = config_home().join(".arb_config_version");
        let config_ver = std::fs::read_to_string(&config_version_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "not found".to_string());

        CheckResult {
            name: "Version".into(),
            status: CheckStatus::Pass,
            message: format!("arb {}, config version: {}", arb_ver, config_ver),
            fix: None,
        }
    }

    // -- Check 7: Zsh plugins --

    pub(crate) fn check_zsh_plugins() -> Vec<CheckResult> {
        let plugins_dir = config_home().join("zsh").join("plugins");
        let expected_plugins = [
            "zsh-z",
            "zsh-autosuggestions",
            "zsh-syntax-highlighting",
            "zsh-completions",
        ];

        expected_plugins
            .iter()
            .map(|plugin| {
                let plugin_path = plugins_dir.join(plugin);
                if plugin_path.exists() {
                    CheckResult {
                        name: format!("Zsh plugin: {}", plugin),
                        status: CheckStatus::Pass,
                        message: "installed".into(),
                        fix: None,
                    }
                } else {
                    CheckResult {
                        name: format!("Zsh plugin: {}", plugin),
                        status: CheckStatus::Warn,
                        message: format!("{} not found", plugin_path.display()),
                        fix: Some("Run `arb init` to install zsh plugins".into()),
                    }
                }
            })
            .collect()
    }

    // -- Helpers --

    fn command_exists(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn git_config_get(key: &str) -> Option<String> {
        Command::new("git")
            .args(["config", "--global", "--get", key])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_doctor_command_with_default() {
        let cmd = DoctorCommand::default();
        // Just ensure it can be constructed
        let _ = format!("{:?}", cmd);
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::imp::*;

        #[test]
        fn should_return_check_result_for_shell_integration() {
            let result = check_shell_integration();
            assert!(!result.name.is_empty());
            assert!(!result.message.is_empty());
            // The status depends on the environment, but the check should not panic
        }

        #[test]
        fn should_return_check_result_for_starship() {
            let result = check_starship();
            assert_eq!(result.name, "Starship");
            assert!(!result.message.is_empty());
        }

        #[test]
        fn should_return_check_result_for_delta() {
            let result = check_delta();
            assert_eq!(result.name, "Delta");
            assert!(!result.message.is_empty());
        }

        #[test]
        fn should_return_check_result_for_user_config() {
            let result = check_user_config();
            assert_eq!(result.name, "User config");
            assert!(!result.message.is_empty());
        }

        #[test]
        fn should_return_check_result_for_app_bundle() {
            let result = check_app_bundle();
            assert_eq!(result.name, "App bundle");
            assert!(!result.message.is_empty());
        }

        #[test]
        fn should_return_check_result_for_version() {
            let result = check_version();
            assert_eq!(result.name, "Version");
            assert!(result.message.contains("arb"));
        }

        #[test]
        fn should_return_check_results_for_all_zsh_plugins() {
            let results = check_zsh_plugins();
            assert_eq!(results.len(), 4);
            assert!(results[0].name.contains("zsh-z"));
            assert!(results[1].name.contains("zsh-autosuggestions"));
            assert!(results[2].name.contains("zsh-syntax-highlighting"));
            assert!(results[3].name.contains("zsh-completions"));
        }

        #[test]
        fn should_run_all_checks_without_panicking() {
            let results = run_all_checks();
            // 1 shell + 1 starship + 1 delta + 1 config + 1 bundle + 1 version + 4 plugins = 10
            assert!(results.len() >= 10);
        }

        #[test]
        fn should_have_valid_status_variants() {
            let pass = CheckStatus::Pass;
            let fail = CheckStatus::Fail;
            let warn = CheckStatus::Warn;
            assert_eq!(pass, CheckStatus::Pass);
            assert_eq!(fail, CheckStatus::Fail);
            assert_eq!(warn, CheckStatus::Warn);
            assert_ne!(pass, fail);
            assert_ne!(pass, warn);
        }
    }
}
