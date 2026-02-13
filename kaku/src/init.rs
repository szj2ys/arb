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
        bail!("`kaku init` is currently supported on macOS only")
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    pub fn run(update_only: bool) -> anyhow::Result<()> {
        install_kaku_wrapper().context("install kaku wrapper")?;

        let script = resolve_setup_script()
            .ok_or_else(|| anyhow!("failed to locate setup_zsh.sh for Kaku initialization"))?;

        let mut cmd = Command::new("/bin/bash");
        cmd.arg(&script).env("KAKU_INIT_INTERNAL", "1");
        if update_only {
            cmd.arg("--update-only");
        }
        let status = cmd
            .status()
            .with_context(|| format!("run {}", script.display()))?;

        if status.success() {
            return Ok(());
        }

        bail!("kaku init failed with status {}", status);
    }

    fn install_kaku_wrapper() -> anyhow::Result<()> {
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

        let preferred_bin = resolve_preferred_kaku_bin()
            .unwrap_or_else(|| PathBuf::from("/Applications/Kaku.app/Contents/MacOS/kaku"));
        let preferred_bin = escape_for_double_quotes(&preferred_bin.display().to_string());

        let script = format!(
            r#"#!/bin/bash
set -euo pipefail

if [[ -n "${{KAKU_BIN:-}}" && -x "${{KAKU_BIN}}" ]]; then
	exec "${{KAKU_BIN}}" "$@"
fi

for candidate in \
	"{preferred_bin}" \
	"/Applications/Kaku.app/Contents/MacOS/kaku" \
	"$HOME/Applications/Kaku.app/Contents/MacOS/kaku"; do
	if [[ -n "$candidate" && -x "$candidate" ]]; then
		exec "$candidate" "$@"
	fi
done

echo "kaku: Kaku.app not found. Expected /Applications/Kaku.app." >&2
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
            .join("kaku")
            .join("zsh")
            .join("bin")
            .join("kaku")
    }

    fn resolve_preferred_kaku_bin() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("KAKU_BIN") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if exe
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case("kaku"))
                .unwrap_or(false)
                && exe.exists()
            {
                return Some(exe);
            }
        }

        for candidate in [
            PathBuf::from("/Applications/Kaku.app/Contents/MacOS/kaku"),
            config::HOME_DIR
                .join("Applications")
                .join("Kaku.app")
                .join("Contents")
                .join("MacOS")
                .join("kaku"),
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

    fn resolve_setup_script() -> Option<PathBuf> {
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
            "/Applications/Kaku.app/Contents/Resources/setup_zsh.sh",
        ));
        candidates.push(
            config::HOME_DIR
                .join("Applications")
                .join("Kaku.app")
                .join("Contents")
                .join("Resources")
                .join("setup_zsh.sh"),
        );

        candidates.into_iter().find(|p| p.exists())
    }
}
