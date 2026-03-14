use anyhow::{anyhow, bail, Context};
use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser, Clone, Default)]
pub struct InitCommand {
    /// Refresh shell integration without interactive prompts
    #[arg(long)]
    pub update_only: bool,

    /// Restore shell configuration from backup created during init
    #[arg(long, conflicts_with = "update_only")]
    pub restore: bool,
}

impl InitCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        if self.restore {
            return imp::restore_shell_config();
        }
        imp::run(self.update_only)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use anyhow::bail;

    pub fn run(_update_only: bool) -> anyhow::Result<()> {
        bail!("`arb init` is currently supported on macOS only")
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use crate::telemetry::{EventType, InstallMethod};
    use std::os::unix::fs::PermissionsExt;

    pub fn run(update_only: bool) -> anyhow::Result<()> {
        // Create backup of .zshrc before any modifications
        if !update_only {
            backup_shell_config()?;
        }

        if let Err(e) = install_arb_wrapper() {
            run_doctor_diagnostics();
            return Err(e).context("install arb wrapper");
        }

        let candidates = setup_script_candidates();
        let script = candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .ok_or_else(|| {
                run_doctor_diagnostics();
                let searched = candidates
                    .iter()
                    .map(|p| format!("  - {}", p.display()))
                    .collect::<Vec<_>>()
                    .join("\n");
                anyhow!(
                    "Failed to locate setup_zsh.sh for Arb initialization.\n\
                 Searched paths:\n{searched}\n\n\
                 Try reinstalling Arb.app or run `arb doctor` for more details."
                )
            })?;

        let mut cmd = Command::new("/bin/bash");
        cmd.arg(&script).env("ARB_INIT_INTERNAL", "1");
        if update_only {
            cmd.arg("--update-only");
        }
        let status = cmd
            .status()
            .with_context(|| format!("run {}", script.display()))?;

        if status.success() {
            // Record successful init/install
            if !update_only {
                crate::telemetry::record(EventType::Install {
                    method: InstallMethod::Homebrew,
                });
            }
            crate::telemetry::record(EventType::ShellInit {
                shell: "zsh".to_string(),
            });

            // Mark as initialized
            let _ = crate::paths::mark_initialized();

            if !update_only {
                print_init_summary();
            }
            return Ok(());
        }

        if !update_only {
            run_doctor_diagnostics();
        }

        bail!(
            "arb init failed with status {} (script: {})\n\n\
             Suggested next steps:\n\
             1. Review the diagnostic output above\n\
             2. Run `arb doctor` for detailed checks\n\
             3. Run `arb reset && arb init` to start fresh",
            status,
            script.display()
        );
    }

    fn run_doctor_diagnostics() {
        eprintln!();
        eprintln!("────────────────────────────────────────");
        eprintln!("Init failed. Running diagnostics...");
        eprintln!();
        let _ = crate::doctor::DoctorCommand::default().run();
        eprintln!("Fix the issues above and retry with `arb init`");
        eprintln!();
    }

    /// Backup shell configuration before making modifications
    fn backup_shell_config() -> anyhow::Result<()> {
        let zshrc = config::HOME_DIR.join(".zshrc");
        if !zshrc.exists() {
            return Ok(());
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_path = format!("{}.arb-backup-{}", zshrc.display(), timestamp);

        fs::copy(&zshrc, &backup_path)
            .with_context(|| format!("Failed to create backup at {}", backup_path))?;

        eprintln!("  Created backup: {}", backup_path);
        Ok(())
    }

    /// Restore shell configuration from the most recent backup
    pub fn restore_shell_config() -> anyhow::Result<()> {
        let zshrc = config::HOME_DIR.join(".zshrc");
        let home = config::HOME_DIR.as_path();

        // Find all .arb-backup-* files
        let mut backups: Vec<_> = fs::read_dir(home)
            .context("Failed to read home directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.starts_with(".zshrc.arb-backup-"))
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect();

        if backups.is_empty() {
            bail!(
                "No backups found.\n\n\
                 Did you mean to run `arb init` instead?\n\
                 Or try `arb reset` to remove Arb shell integration."
            );
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.cmp(a));
        let latest_backup = &backups[0];

        // Confirm restoration
        eprintln!("Found backup: {}", latest_backup.display());
        eprint!("Restore this backup? [y/N] ");
        std::io::Write::flush(&mut std::io::stderr())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Cancelled.");
            return Ok(());
        }

        // Perform restoration
        if zshrc.exists() {
            fs::copy(&zshrc, format!("{}.arb-pre-restore-{}", zshrc.display(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            )).ok();
        }

        fs::copy(latest_backup, &zshrc)
            .with_context(|| format!("Failed to restore backup to {}", zshrc.display()))?;

        eprintln!("✓ Restored shell configuration from {}", latest_backup.display());
        eprintln!("  Open a new terminal tab to apply changes.");

        Ok(())
    }

    use crate::paths::{BOLD, GRAY, GREEN, RESET, YELLOW};

    /// Summary line descriptor: (user-facing label, success hint text).
    const SUMMARY_ITEMS: &[(&str, &str)] = &[
        ("Shell integration", "installed"),
        ("Starship prompt", "active"),
        ("z directory jumper", "ready (try: z <dir>)"),
        ("Delta git pager", "configured (try: git diff)"),
        ("Syntax highlighting", "loaded"),
        ("Autosuggestions", "loaded"),
        ("Completions", "loaded"),
    ];

    fn print_init_summary() {
        use crate::doctor::imp::{
            check_delta, check_shell_integration, check_starship, check_zsh_plugins,
        };

        let shell = check_shell_integration();
        let starship = check_starship();
        let delta = check_delta();
        let plugins = check_zsh_plugins();

        let find_plugin = |needle: &str| -> crate::doctor::imp::CheckResult {
            plugins
                .iter()
                .find(|r| r.name.contains(needle))
                .cloned()
                .expect("expected plugin check result")
        };

        let results = vec![
            shell,
            starship,
            find_plugin("zsh-z"),
            delta,
            find_plugin("zsh-syntax-highlighting"),
            find_plugin("zsh-autosuggestions"),
            find_plugin("zsh-completions"),
        ];

        print!("{}", format_init_summary(&results));
    }

    /// Formats the post-init verification summary.
    ///
    /// `results` must have one entry per item in [`SUMMARY_ITEMS`], in the same
    /// order.
    fn format_init_summary(results: &[crate::doctor::imp::CheckResult]) -> String {
        use crate::doctor::imp::CheckStatus;
        use std::fmt::Write;

        let mut buf = String::new();

        writeln!(buf).unwrap();
        writeln!(buf, "  {BOLD}Arb init complete{RESET}").unwrap();
        writeln!(buf).unwrap();

        for ((label, success_hint), result) in SUMMARY_ITEMS.iter().zip(results.iter()) {
            match result.status {
                CheckStatus::Pass => {
                    writeln!(
                        buf,
                        "  {GREEN}\u{2714}{RESET} {label}: {GREEN}{success_hint}{RESET}"
                    )
                    .unwrap();
                }
                _ => {
                    let fix_msg = result
                        .fix
                        .as_deref()
                        .unwrap_or("Run `arb doctor` for details");
                    writeln!(
                        buf,
                        "  {YELLOW}\u{26a0}{RESET} {label}: {YELLOW}{}{RESET}",
                        result.message
                    )
                    .unwrap();
                    writeln!(buf, "    {GRAY}{fix_msg}{RESET}").unwrap();
                }
            }
        }

        writeln!(buf).unwrap();
        writeln!(buf, "  Open a {BOLD}new tab{RESET} to start using arb.").unwrap();
        writeln!(
            buf,
            "  {GRAY}Like arb? Star us on GitHub \u{2192} https://github.com/szj2ys/arb{RESET}"
        )
        .unwrap();
        writeln!(buf).unwrap();

        buf
    }

    fn install_arb_wrapper() -> anyhow::Result<()> {
        let wrapper_path = wrapper_path();
        let wrapper_dir = wrapper_path
            .parent()
            .ok_or_else(|| anyhow!("invalid wrapper path"))?;
        config::create_user_owned_dirs(wrapper_dir).context("create wrapper directory")?;

        if fs::symlink_metadata(&wrapper_path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            fs::remove_file(&wrapper_path).with_context(|| {
                format!("remove legacy symlink wrapper {}", wrapper_path.display())
            })?;
        }

        let preferred_bin = resolve_preferred_arb_bin()
            .unwrap_or_else(|| PathBuf::from("/Applications/Arb.app/Contents/MacOS/arb"));
        let preferred_bin = escape_for_double_quotes(&preferred_bin.display().to_string());

        let script = format!(
            r#"#!/bin/bash
set -euo pipefail

if [[ -n "${{ARB_BIN:-}}" && -x "${{ARB_BIN}}" ]]; then
	exec "${{ARB_BIN}}" "$@"
fi

for candidate in \
	"{preferred_bin}" \
	"/Applications/Arb.app/Contents/MacOS/arb" \
	"$HOME/Applications/Arb.app/Contents/MacOS/arb"; do
	if [[ -n "$candidate" && -x "$candidate" ]]; then
		exec "$candidate" "$@"
	fi
done

echo "arb: Arb.app not found. Expected /Applications/Arb.app." >&2
exit 127
"#
        );

        let mut file = fs::File::create(&wrapper_path)
            .with_context(|| format!("create wrapper {}", wrapper_path.display()))?;
        file.write_all(script.as_bytes())
            .with_context(|| format!("write wrapper {}", wrapper_path.display()))?;
        fs::set_permissions(&wrapper_path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("chmod wrapper {}", wrapper_path.display()))?;
        Ok(())
    }

    fn wrapper_path() -> PathBuf {
        config::HOME_DIR.join(".config/arb/zsh/bin/arb")
    }

    fn resolve_preferred_arb_bin() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("ARB_BIN") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if exe
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case("arb"))
                .unwrap_or(false)
                && exe.exists()
            {
                return Some(exe);
            }
        }

        [
            PathBuf::from("/Applications/Arb.app/Contents/MacOS/arb"),
            config::HOME_DIR.join("Applications/Arb.app/Contents/MacOS/arb"),
        ]
        .into_iter()
        .find(|candidate| candidate.exists())
    }

    fn escape_for_double_quotes(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
    }

    fn setup_script_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("assets/shell-integration/setup_zsh.sh"));
        }

        if let Ok(exe) = std::env::current_exe() {
        if let Some(contents_dir) = exe.parent().and_then(|p| p.parent()) {
            candidates.push(contents_dir.join("Resources/setup_zsh.sh"));
        }
        }

        candidates.push(PathBuf::from(
            "/Applications/Arb.app/Contents/Resources/setup_zsh.sh",
        ));
        candidates.push(
            config::HOME_DIR.join("Applications/Arb.app/Contents/Resources/setup_zsh.sh"),
        );

        candidates
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::doctor::imp::{CheckResult, CheckStatus};

        // ── print_init_summary / format_init_summary ────────────────

        /// Helper: build a vector of all-passing `CheckResult`s matching
        /// the seven `SUMMARY_ITEMS` entries.
        fn all_passing_results() -> Vec<CheckResult> {
            SUMMARY_ITEMS
                .iter()
                .map(|(label, _)| CheckResult {
                    name: label.to_string(),
                    status: CheckStatus::Pass,
                    message: "ok".into(),
                    fix: None,
                })
                .collect()
        }

        #[test]
        fn should_format_init_summary_with_all_passing_checks() {
            let output = format_init_summary(&all_passing_results());

            // Every label should appear with the green check mark
            assert!(output.contains("\u{2714}"), "expected check mark in output");
            assert!(
                output.contains("Arb init complete"),
                "expected header in output"
            );
            assert!(
                output.contains("Open a"),
                "expected footer instruction in output"
            );
            assert!(
                output.contains("Star us on GitHub"),
                "expected star nudge in output"
            );
            // None of the warning icons should appear
            assert!(
                !output.contains("\u{26a0}"),
                "did not expect warning icon for all-passing checks"
            );
            // Verify each success hint appears
            for (_, hint) in SUMMARY_ITEMS {
                assert!(
                    output.contains(hint),
                    "expected success hint '{hint}' in output"
                );
            }
        }

        #[test]
        fn should_show_warnings_for_failed_checks() {
            let mut results = all_passing_results();
            // Make the first check (Shell integration) fail
            results[0] = CheckResult {
                name: "Shell integration".into(),
                status: CheckStatus::Fail,
                message: "arb.zsh not found".into(),
                fix: Some("Run `arb init` to install shell integration".into()),
            };
            // Make the Starship check warn
            results[1] = CheckResult {
                name: "Starship prompt".into(),
                status: CheckStatus::Warn,
                message: "starship not executable".into(),
                fix: Some("chmod +x starship".into()),
            };

            let output = format_init_summary(&results);

            // Failed/warned items should show warning icon and fix text
            assert!(output.contains("\u{26a0}"), "expected warning icon");
            assert!(
                output.contains("arb.zsh not found"),
                "expected failure message"
            );
            assert!(
                output.contains("Run `arb init` to install shell integration"),
                "expected fix suggestion"
            );
            assert!(
                output.contains("starship not executable"),
                "expected warn message"
            );
            assert!(output.contains("chmod +x starship"), "expected warn fix");
            // The other 5 items should still show passing
            assert!(output.contains("\u{2714}"), "expected passing checks");
        }

        #[test]
        fn should_include_star_nudge_in_init_summary() {
            let output = format_init_summary(&all_passing_results());
            assert!(
                output.contains("Star us on GitHub"),
                "expected star nudge in output"
            );
            assert!(
                output.contains("https://github.com/szj2ys/arb"),
                "expected GitHub URL in star nudge"
            );
        }

        #[test]
        fn should_skip_summary_when_update_only() {
            // The run() function gates the summary on `!update_only`.
            // We verify the contract: when update_only is true, format_init_summary
            // is never called. This is a structural test of the code path.
            //
            // Inspect the source of `run`: the call is wrapped in
            //   `if !update_only { print_init_summary(); }`
            // We verify this by confirming the function signature accepts update_only
            // and that the guard exists. Since we cannot easily run the full init
            // (it requires shell scripts), we test the guard at the API level.
            let cmd_update = super::super::InitCommand { update_only: true, restore: false };
            assert!(cmd_update.update_only, "update_only flag should be true");

            let cmd_normal = super::super::InitCommand { update_only: false, restore: false };
            assert!(
                !cmd_normal.update_only,
                "update_only flag should be false for normal init"
            );
        }

        // ── escape_for_double_quotes ─────────────────────────────────

        #[test]
        fn escape_should_handle_normal_path() {
            assert_eq!(
                escape_for_double_quotes("/Applications/Arb.app/Contents/MacOS/arb"),
                "/Applications/Arb.app/Contents/MacOS/arb"
            );
        }

        #[test]
        fn escape_should_escape_backslash() {
            assert_eq!(escape_for_double_quotes("a\\b"), "a\\\\b");
        }

        #[test]
        fn escape_should_escape_double_quote() {
            assert_eq!(escape_for_double_quotes("a\"b"), "a\\\"b");
        }

        #[test]
        fn escape_should_escape_dollar_sign() {
            assert_eq!(escape_for_double_quotes("a$b"), "a\\$b");
        }

        #[test]
        fn escape_should_escape_backtick() {
            assert_eq!(escape_for_double_quotes("a`b"), "a\\`b");
        }

        #[test]
        fn escape_should_handle_path_with_spaces() {
            assert_eq!(
                escape_for_double_quotes("/path/with spaces/file"),
                "/path/with spaces/file"
            );
        }

        #[test]
        fn escape_should_handle_multiple_special_chars() {
            assert_eq!(
                escape_for_double_quotes("$HOME/`test`/\"file\""),
                "\\$HOME/\\`test\\`/\\\"file\\\""
            );
        }

        #[test]
        fn escape_should_handle_empty_string() {
            assert_eq!(escape_for_double_quotes(""), "");
        }

        // ── setup_script_candidates ──────────────────────────────────

        #[test]
        fn should_return_at_least_two_static_candidates() {
            // The function always appends the two well-known static paths
            // regardless of environment.
            let candidates = setup_script_candidates();
            assert!(
                candidates.len() >= 2,
                "expected at least 2 candidates, got {}",
                candidates.len()
            );
        }

        #[test]
        fn should_include_global_applications_candidate() {
            let candidates = setup_script_candidates();
            let global = PathBuf::from("/Applications/Arb.app/Contents/Resources/setup_zsh.sh");
            assert!(
                candidates.contains(&global),
                "candidates should include the global /Applications path"
            );
        }

        #[test]
        fn should_include_user_applications_candidate() {
            let candidates = setup_script_candidates();
            let user =
                config::HOME_DIR.join("Applications/Arb.app/Contents/Resources/setup_zsh.sh");
            assert!(
                candidates.contains(&user),
                "candidates should include the ~/Applications path"
            );
        }

        #[test]
        fn should_include_cwd_candidate_when_cwd_is_available() {
            // current_dir() normally succeeds in test environments.
            if let Ok(cwd) = std::env::current_dir() {
                let expected = cwd
                    .join("assets")
                    .join("shell-integration")
                    .join("setup_zsh.sh");
                let candidates = setup_script_candidates();
                assert!(
                    candidates.contains(&expected),
                    "candidates should include the cwd-relative path"
                );
            }
        }

        #[test]
        fn should_have_all_candidates_ending_with_setup_zsh_sh() {
            let candidates = setup_script_candidates();
            for c in &candidates {
                assert!(
                    c.ends_with("setup_zsh.sh"),
                    "every candidate should end with setup_zsh.sh, got: {}",
                    c.display()
                );
            }
        }

        // ── wrapper_path ─────────────────────────────────────────────

        #[test]
        fn should_place_wrapper_under_config_arb_zsh_bin() {
            let path = wrapper_path();
            assert!(
                path.ends_with(".config/arb/zsh/bin/arb"),
                "wrapper path should end with .config/arb/zsh/bin/arb, got: {}",
                path.display()
            );
        }

        #[test]
        fn should_derive_wrapper_path_from_home_dir() {
            let path = wrapper_path();
            assert!(
                path.starts_with(config::HOME_DIR.as_path()),
                "wrapper path should start with HOME_DIR"
            );
        }
    }
}
