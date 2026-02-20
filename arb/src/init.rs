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

        for candidate in [
            PathBuf::from("/Applications/Arb.app/Contents/MacOS/arb"),
            config::HOME_DIR
                .join("Applications")
                .join("Arb.app")
                .join("Contents")
                .join("MacOS")
                .join("arb"),
        ] {
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
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
}
