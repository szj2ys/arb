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
}

impl InitCommand {
    pub fn run(&self) -> anyhow::Result<()> {
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
    use std::os::unix::fs::PermissionsExt;

    pub fn run(update_only: bool) -> anyhow::Result<()> {
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
        config::HOME_DIR
            .join(".config")
            .join("arb")
            .join("zsh")
            .join("bin")
            .join("arb")
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
            config::HOME_DIR
                .join("Applications")
                .join("Arb.app")
                .join("Contents")
                .join("MacOS")
                .join("arb"),
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
            candidates.push(
                cwd.join("assets")
                    .join("shell-integration")
                    .join("setup_zsh.sh"),
            );
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(contents_dir) = exe.parent().and_then(|p| p.parent()) {
                candidates.push(contents_dir.join("Resources").join("setup_zsh.sh"));
            }
        }

        candidates.push(PathBuf::from(
            "/Applications/Arb.app/Contents/Resources/setup_zsh.sh",
        ));
        candidates.push(
            config::HOME_DIR
                .join("Applications")
                .join("Arb.app")
                .join("Contents")
                .join("Resources")
                .join("setup_zsh.sh"),
        );

        candidates
    }

    #[cfg(test)]
    mod tests {
        use super::*;

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
            let global = PathBuf::from(
                "/Applications/Arb.app/Contents/Resources/setup_zsh.sh",
            );
            assert!(
                candidates.contains(&global),
                "candidates should include the global /Applications path"
            );
        }

        #[test]
        fn should_include_user_applications_candidate() {
            let candidates = setup_script_candidates();
            let user = config::HOME_DIR
                .join("Applications/Arb.app/Contents/Resources/setup_zsh.sh");
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
