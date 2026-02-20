use anyhow::{anyhow, bail, Context};
use clap::Parser;

#[derive(Debug, Parser, Clone, Default)]
pub struct UpdateCommand {
    /// Apply a previously downloaded update and restart Arb.
    ///
    /// By default, `arb update` will only download and stage the update
    /// so that you can apply it later without interrupting your current session.
    #[arg(long)]
    apply: bool,
}

impl UpdateCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        imp::run(self)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use anyhow::bail;

    pub fn run(_cmd: &super::UpdateCommand) -> anyhow::Result<()> {
        bail!("`arb update` is currently supported on macOS only")
    }
}

/// Internal helper, exposed so we can test cleanup logic without invoking
/// the full update flow.
#[cfg(target_os = "macos")]
pub fn cleanup_old_update_dirs_for_tests(update_root: &std::path::Path) -> anyhow::Result<()> {
    imp::cleanup_old_update_dirs_impl(update_root)
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use indicatif::{ProgressBar, ProgressStyle};
    use serde::Deserialize;
    use std::fs;
    use std::io::{Read as _, Write};
    use std::path::{Component, Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const RELEASE_API_URL: &str = "https://api.github.com/repos/szj2ys/arb/releases/latest";
    const LATEST_ZIP_URL: &str =
        "https://github.com/szj2ys/arb/releases/latest/download/arb_for_update.zip";
    const LATEST_SHA_URL: &str =
        "https://github.com/szj2ys/arb/releases/latest/download/arb_for_update.zip.sha256";
    const RELEASE_LATEST_URL: &str = "https://github.com/szj2ys/arb/releases/latest";
    const UPDATE_ZIP_NAME: &str = "arb_for_update.zip";
    const UPDATE_SHA_NAME: &str = "arb_for_update.zip.sha256";
    const BREW_CASK_NAME: &str = "szj2ys/arb/arb";

    const PENDING_DIR_REL: &str = "updates/pending";
    const PENDING_MARKER_NAME: &str = "pending-update.json";
    const OLD_UPDATE_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 7);

    #[derive(Debug, Deserialize)]
    struct GitHubRelease {
        tag_name: String,
        assets: Vec<GitHubAsset>,
    }

    #[derive(Debug, Deserialize)]
    struct GitHubAsset {
        name: String,
        browser_download_url: String,
    }

    struct BrewInfo {
        brew_bin: PathBuf,
        cask_name: String,
    }

    enum UpdateProvider {
        Direct,
        Brew(BrewInfo),
    }

    pub fn run(cmd: &UpdateCommand) -> anyhow::Result<()> {
        match resolve_update_provider()? {
            UpdateProvider::Brew(info) => {
                println!("Detected Homebrew-managed installation. Using brew upgrade...");
                return run_brew_upgrade(&info);
            }
            UpdateProvider::Direct => {}
        }

        cleanup_old_update_dirs().context("cleanup old update directories")?;

        let pending_dir = pending_dir();
        config::create_user_owned_dirs(&pending_dir).context("create pending updates directory")?;
        let marker_path = pending_marker_path_in(&pending_dir);

        if cmd.apply {
            return apply_pending_update(&marker_path);
        }

        let current_version = config::arb_version().to_string();
        let current_version_display = format_version_for_display(&current_version);
        println!("Current version: {}", current_version_display);
        println!("Checking latest release...");

        let release = match curl_get_text(RELEASE_API_URL, &current_version)
            .context("request release metadata")
            .and_then(|raw| {
                serde_json::from_str::<GitHubRelease>(&raw).context("parse release metadata")
            }) {
            Ok(release) => Some(release),
            Err(err) => {
                println!(
                    "Release API unavailable ({}). Falling back to latest asset URL.",
                    err
                );
                None
            }
        };

        if let Some(release) = &release {
            if !is_newer_version(&release.tag_name, &current_version) {
                println!(
                    "Already up to date. Current={} Latest={}",
                    current_version_display,
                    format_version_for_display(&release.tag_name)
                );
                return Ok(());
            }
        } else if let Some(tag_name) = resolve_latest_tag_from_redirect(&current_version)? {
            if !is_newer_version(&tag_name, &current_version) {
                println!(
                    "Already up to date. Current={} Latest={}",
                    current_version_display,
                    format_version_for_display(&tag_name)
                );
                return Ok(());
            }
        }

        let zip_url = release
            .as_ref()
            .and_then(|rel| find_asset(&rel.assets, UPDATE_ZIP_NAME))
            .map(|asset| asset.browser_download_url.as_str())
            .unwrap_or(LATEST_ZIP_URL);

        let sha_url = release
            .as_ref()
            .and_then(|rel| find_asset(&rel.assets, UPDATE_SHA_NAME))
            .map(|asset| asset.browser_download_url.as_str())
            .or(Some(LATEST_SHA_URL));

        let update_root = config::DATA_DIR.join("updates");
        config::create_user_owned_dirs(&update_root).context("create updates directory")?;

        let update_label = release
            .as_ref()
            .map(|r| r.tag_name.as_str())
            .unwrap_or("latest");
        let normalized_label = sanitize_tag(update_label);

        if let Ok(existing) = read_pending_marker(&marker_path) {
            if existing.tag == normalized_label {
                println!(
                    "Update v{} is already staged. Restart Arb or run `arb update --apply` to upgrade.",
                    format_version_for_display(&existing.tag)
                );
                println!("Staged at: {}", existing.new_app_path.display());
                return Ok(());
            }

            println!(
                "Found staged update v{}. Replacing with v{}...",
                format_version_for_display(&existing.tag),
                format_version_for_display(&normalized_label)
            );
            cleanup_pending_update(&marker_path).ok();
        }

        // Stage the update into a stable location so it can be applied later.
        let staging_dir = pending_dir.join(&normalized_label);
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        config::create_user_owned_dirs(&staging_dir).context("create update staging directory")?;

        let zip_path = staging_dir.join(UPDATE_ZIP_NAME);
        println!("Downloading {} ...", UPDATE_ZIP_NAME);
        download_to_file_with_progress(zip_url, &zip_path, &current_version)
            .context("failed to download update package")?;

        if let Some(sha_url) = sha_url {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .expect("valid spinner template"),
            );
            spinner.set_message("Downloading checksum...");
            spinner.enable_steady_tick(Duration::from_millis(100));

            match curl_get_text(sha_url, &current_version) {
                Ok(checksum_text) => {
                    spinner.finish_and_clear();
                    println!("Verifying package checksum...");
                    verify_sha256(&zip_path, &checksum_text)
                        .context("checksum verification failed")?;
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    println!(
                        "Checksum unavailable ({}). Continuing without checksum.",
                        err
                    );
                }
            }
        }

        let extracted_dir = staging_dir.join("extracted");
        config::create_user_owned_dirs(&extracted_dir).context("create extraction directory")?;

        run_status(
            Command::new("/usr/bin/ditto")
                .arg("-x")
                .arg("-k")
                .arg(&zip_path)
                .arg(&extracted_dir),
            "extract update package",
        )?;

        let new_app_path = find_arb_app(&extracted_dir).ok_or_else(|| {
            anyhow!(
                "update package `{}` does not contain `Arb.app`",
                UPDATE_ZIP_NAME
            )
        })?;
        if let Ok(new_version) = read_app_version(&new_app_path) {
            if !is_newer_version(&new_version, &current_version) {
                println!(
                    "Already up to date after download. Current={} Package={}",
                    current_version_display,
                    format_version_for_display(&new_version)
                );
                let _ = fs::remove_dir_all(&staging_dir);
                return Ok(());
            }
        }

        let marker = PendingUpdateMarker {
            tag: normalized_label.clone(),
            staging_dir: staging_dir.clone(),
            new_app_path: new_app_path.clone(),
            created_at: now_unix_seconds(),
        };
        write_pending_marker(&marker_path, &marker).context("write pending update marker")?;

        println!();
        println!(
            "New version v{} is ready. Restart Arb or run `arb update --apply` to upgrade.",
            format_version_for_display(update_label)
        );
        println!("Staged at: {}", marker.new_app_path.display());
        Ok(())
    }

    #[derive(Debug, Deserialize, serde::Serialize)]
    struct PendingUpdateMarker {
        tag: String,
        staging_dir: PathBuf,
        new_app_path: PathBuf,
        created_at: u64,
    }

    fn pending_dir() -> PathBuf {
        config::DATA_DIR.join(PENDING_DIR_REL)
    }

    fn pending_marker_path_in(pending_dir: &Path) -> PathBuf {
        pending_dir.join(PENDING_MARKER_NAME)
    }

    fn read_pending_marker(path: &Path) -> anyhow::Result<PendingUpdateMarker> {
        let raw = fs::read_to_string(path).with_context(|| {
            format!(
                "read pending update marker {}",
                path.as_os_str().to_string_lossy()
            )
        })?;
        serde_json::from_str(&raw).context("parse pending update marker")
    }

    fn write_pending_marker(path: &Path, marker: &PendingUpdateMarker) -> anyhow::Result<()> {
        let data = serde_json::to_vec_pretty(marker).context("serialize pending update marker")?;

        let mut tmp = path.to_path_buf();
        tmp.set_extension("json.tmp");
        fs::write(&tmp, data).with_context(|| {
            format!(
                "write pending update marker temp file {}",
                tmp.as_os_str().to_string_lossy()
            )
        })?;
        fs::rename(&tmp, path).with_context(|| {
            format!(
                "commit pending update marker {}",
                path.as_os_str().to_string_lossy()
            )
        })?;
        Ok(())
    }

    fn apply_pending_update(marker_path: &Path) -> anyhow::Result<()> {
        let marker = read_pending_marker(marker_path).context("read pending update marker")?;

        if !marker.new_app_path.exists() {
            cleanup_pending_update(marker_path).ok();
            bail!(
                "pending update is missing staged app at {}",
                marker.new_app_path.display()
            );
        }

        let target_app = resolve_target_app_path().context("resolve installed Arb.app path")?;
        ensure_can_write_target(&target_app)?;

        let update_root = config::DATA_DIR.join("updates");
        config::create_user_owned_dirs(&update_root).context("create updates directory")?;

        let now = now_unix_seconds();
        let helper_script = update_root.join(format!("apply-update-{}.sh", now));
        write_helper_script(&helper_script).context("write update helper script")?;

        spawn_update_helper(
            &helper_script,
            &target_app,
            &marker.new_app_path,
            &marker.staging_dir,
        )
        .context("spawn update helper")?;

        println!(
            "Applying staged update v{} in background...",
            format_version_for_display(&marker.tag)
        );

        // Best-effort: remove marker so future `arb update` won't think it's still pending.
        // The helper script will also clean up the staging directory.
        let _ = fs::remove_file(marker_path);
        Ok(())
    }

    fn cleanup_pending_update(marker_path: &Path) -> anyhow::Result<()> {
        let marker = match read_pending_marker(marker_path) {
            Ok(m) => m,
            Err(_) => {
                let _ = fs::remove_file(marker_path);
                return Ok(());
            }
        };

        let _ = fs::remove_file(marker_path);
        let _ = fs::remove_dir_all(&marker.staging_dir);
        Ok(())
    }

    fn cleanup_old_update_dirs() -> anyhow::Result<()> {
        let update_root = config::DATA_DIR.join("updates");
        cleanup_old_update_dirs_impl(&update_root)
    }

    pub(super) fn cleanup_old_update_dirs_impl(update_root: &Path) -> anyhow::Result<()> {
        if !update_root.exists() {
            return Ok(());
        }

        let now = SystemTime::now();
        let entries = match fs::read_dir(update_root) {
            Ok(e) => e,
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "read updates directory {}",
                        update_root.as_os_str().to_string_lossy()
                    )
                })
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("pending") {
                continue;
            }

            // Prefer deterministic cleanup based on directory name suffix `-<unix_seconds>`
            // (which matches how work dirs were historically created). Fall back to mtime.
            if let Some(age_secs) = estimate_update_dir_age_secs(&path) {
                if age_secs > OLD_UPDATE_TTL.as_secs() {
                    let _ = fs::remove_dir_all(&path);
                }
                continue;
            }

            let Ok(meta) = entry.metadata() else {
                continue;
            };
            let Ok(modified) = meta.modified() else {
                continue;
            };

            if now
                .duration_since(modified)
                .map(|age| age > OLD_UPDATE_TTL)
                .unwrap_or(false)
            {
                let _ = fs::remove_dir_all(&path);
            }
        }

        Ok(())
    }

    fn estimate_update_dir_age_secs(path: &Path) -> Option<u64> {
        let name = path.file_name()?.to_string_lossy();
        let ts = name.rsplit('-').next()?.parse::<u64>().ok()?;
        let now = now_unix_seconds();
        now.checked_sub(ts)
    }

    fn resolve_update_provider() -> anyhow::Result<UpdateProvider> {
        if let Some(provider) = std::env::var_os("ARB_UPDATE_PROVIDER") {
            let provider = provider.to_string_lossy().to_ascii_lowercase();
            return match provider.as_str() {
                "brew" => {
                    let brew_info = resolve_brew_info()?.ok_or_else(|| {
                        anyhow!(
                            "ARB_UPDATE_PROVIDER=brew but brew-managed Arb installation was not found"
                        )
                    })?;
                    Ok(UpdateProvider::Brew(brew_info))
                }
                "direct" => Ok(UpdateProvider::Direct),
                other => bail!("invalid ARB_UPDATE_PROVIDER `{}`", other),
            };
        }

        if let Some(brew_info) = resolve_brew_info()? {
            return Ok(UpdateProvider::Brew(brew_info));
        }

        let exe = std::env::current_exe().context("resolve current executable path")?;
        if let Some(target) = std::env::var_os("ARB_UPDATE_TARGET_APP") {
            let target = PathBuf::from(target);
            if path_contains_caskroom(&exe) || path_contains_caskroom(&target) {
                if find_brew_binary().is_none() {
                    bail!(
                        "Arb appears to be Homebrew-managed but `brew` was not found in PATH or standard locations"
                    );
                }
            }
        }

        Ok(UpdateProvider::Direct)
    }

    fn resolve_brew_info() -> anyhow::Result<Option<BrewInfo>> {
        let Some(brew_bin) = find_brew_binary() else {
            return Ok(None);
        };

        if is_brew_cask_installed(&brew_bin, BREW_CASK_NAME)? {
            return Ok(Some(BrewInfo {
                brew_bin,
                cask_name: BREW_CASK_NAME.to_string(),
            }));
        }

        // Old cask name "arb" conflicts with another software in homebrew/cask.
        // Do not use it; prompt user to migrate instead.
        if is_brew_cask_installed(&brew_bin, "arb")? {
            println!(
                "WARNING: Detected old Homebrew cask 'arb' which conflicts with another software."
            );
            println!("Please migrate to the new cask name manually:");
            println!();
            println!("  brew uninstall --cask arb");
            println!("  brew install --cask {}", BREW_CASK_NAME);
            println!();
            println!("After migration, run 'arb update' again.");
            // Return None to fall back to direct update from GitHub
            return Ok(None);
        }

        Ok(None)
    }

    fn find_brew_binary() -> Option<PathBuf> {
        for candidate in ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"] {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        std::env::var_os("PATH").and_then(|path_var| {
            std::env::split_paths(&path_var)
                .map(|dir| dir.join("brew"))
                .find(|candidate| candidate.exists())
        })
    }

    fn path_contains_caskroom(path: &Path) -> bool {
        path.components().any(|c| match c {
            Component::Normal(name) => name == "Caskroom",
            _ => false,
        })
    }

    fn is_brew_cask_installed(brew_bin: &Path, cask_name: &str) -> anyhow::Result<bool> {
        let output = Command::new(brew_bin)
            .arg("list")
            .arg("--cask")
            .arg("--versions")
            .arg(cask_name)
            .output()
            .with_context(|| format!("query brew cask installation for {}", cask_name))?;

        if output.status.success() {
            return Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        if stderr.contains("no such cask")
            || stderr.contains("not installed")
            || output.status.code() == Some(1)
        {
            return Ok(false);
        }

        bail!(
            "query brew cask installation for {} failed: {}",
            cask_name,
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }

    fn is_brew_cask_outdated(brew_bin: &Path, cask_name: &str) -> anyhow::Result<bool> {
        let output = run_output(
            Command::new(brew_bin)
                .arg("outdated")
                .arg("--cask")
                .arg("--quiet")
                .arg(cask_name),
            &format!("query brew cask outdated status for {}", cask_name),
        )?;
        Ok(!String::from_utf8_lossy(&output).trim().is_empty())
    }

    fn run_brew_upgrade(info: &BrewInfo) -> anyhow::Result<()> {
        match is_brew_cask_outdated(&info.brew_bin, &info.cask_name) {
            Ok(false) => {
                println!(
                    "Already up to date (brew cask `{}` has no available update).",
                    info.cask_name
                );
                return Ok(());
            }
            Ok(true) => {}
            Err(err) => {
                println!(
                    "Unable to pre-check brew outdated status ({}). Trying upgrade directly.",
                    err
                );
            }
        }

        let primary = Command::new(&info.brew_bin)
            .arg("upgrade")
            .arg("--cask")
            .arg(&info.cask_name)
            .status()
            .with_context(|| format!("failed to run brew upgrade for {}", info.cask_name))?;
        if primary.success() {
            return Ok(());
        }

        let fallback_name = if info.cask_name == BREW_CASK_NAME {
            "arb"
        } else {
            BREW_CASK_NAME
        };

        let fallback = Command::new(&info.brew_bin)
            .arg("upgrade")
            .arg("--cask")
            .arg(fallback_name)
            .status()
            .with_context(|| {
                format!("failed to run brew upgrade fallback for {}", fallback_name)
            })?;
        if fallback.success() {
            return Ok(());
        }

        bail!(
            "brew update failed (tried `brew upgrade --cask {}` and `brew upgrade --cask {}`)",
            info.cask_name,
            fallback_name
        )
    }

    fn resolve_latest_tag_from_redirect(current_version: &str) -> anyhow::Result<Option<String>> {
        let output = run_output(
            Command::new("/usr/bin/curl")
                .arg("--fail")
                .arg("--location")
                .arg("--silent")
                .arg("--show-error")
                .arg("--retry")
                .arg("2")
                .arg("--connect-timeout")
                .arg("10")
                .arg("--user-agent")
                .arg(format!("arb/{}", current_version))
                .arg("--write-out")
                .arg("%{url_effective}")
                .arg("--output")
                .arg("/dev/null")
                .arg(RELEASE_LATEST_URL),
            "resolve latest release tag via redirect",
        )?;
        let effective_url = String::from_utf8(output)
            .context("latest redirect url is not valid UTF-8")?
            .trim()
            .to_string();
        if effective_url.is_empty() {
            return Ok(None);
        }

        let tag = effective_url
            .rsplit('/')
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Ok(tag)
    }

    fn find_asset<'a>(assets: &'a [GitHubAsset], name: &str) -> Option<&'a GitHubAsset> {
        assets.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    fn sanitize_tag(tag: &str) -> String {
        tag.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn now_unix_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn curl_get_text(url: &str, current_version: &str) -> anyhow::Result<String> {
        let output = run_output(
            Command::new("/usr/bin/curl")
                .arg("--fail")
                .arg("--location")
                .arg("--silent")
                .arg("--show-error")
                .arg("--retry")
                .arg("3")
                .arg("--connect-timeout")
                .arg("15")
                .arg("--user-agent")
                .arg(format!("arb/{}", current_version))
                .arg(url),
            "request update metadata",
        )?;
        String::from_utf8(output).context("curl returned non-utf8 response")
    }

    fn download_to_file_with_progress(
        url: &str,
        output_path: &Path,
        current_version: &str,
    ) -> anyhow::Result<()> {
        let user_agent = format!("arb/{}", current_version);
        let resp = ureq::agent()
            .get(url)
            .set("User-Agent", &user_agent)
            .call()
            .context("failed to start download")?;

        let total_size = resp
            .header("Content-Length")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let pb = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "[{wide_bar:.cyan/blue}] {bytes} / {total_bytes} ({bytes_per_sec}, ETA {eta})",
                    )
                    .expect("valid progress bar template")
                    .progress_chars("=>-"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {bytes} downloaded ({bytes_per_sec})")
                    .expect("valid spinner template"),
            );
            pb
        };

        let mut reader = resp.into_reader();
        let mut file =
            fs::File::create(output_path).context("failed to create output file for download")?;
        let mut buf = [0u8; 8192];
        loop {
            let n = reader
                .read(&mut buf)
                .context("failed to read from download stream")?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .context("failed to write download data to file")?;
            pb.inc(n as u64);
        }
        pb.finish_and_clear();

        Ok(())
    }

    fn verify_sha256(zip_path: &Path, checksum_text: &str) -> anyhow::Result<()> {
        let expected = checksum_text
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("checksum file is empty"))?
            .trim()
            .to_ascii_lowercase();

        if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("checksum file has invalid sha256: {}", expected);
        }

        let output = run_output(
            Command::new("/usr/bin/shasum")
                .arg("-a")
                .arg("256")
                .arg(zip_path),
            "compute sha256",
        )?;
        let actual_line =
            String::from_utf8(output).context("`shasum` output was not valid UTF-8")?;
        let actual = actual_line
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("failed to parse `shasum` output"))?
            .trim()
            .to_ascii_lowercase();

        if actual != expected {
            bail!("sha256 mismatch (expected {}, got {})", expected, actual);
        }
        Ok(())
    }

    fn find_arb_app(extracted_dir: &Path) -> Option<PathBuf> {
        let direct = extracted_dir.join("Arb.app");
        if direct.exists() {
            return Some(direct);
        }

        let entries = fs::read_dir(extracted_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case("Arb.app"))
                .unwrap_or(false)
            {
                return Some(path);
            }
        }
        None
    }

    fn read_app_version(app_path: &Path) -> anyhow::Result<String> {
        let plist = app_path.join("Contents/Info.plist");
        let output = run_output(
            Command::new("/usr/libexec/PlistBuddy")
                .arg("-c")
                .arg("Print :CFBundleShortVersionString")
                .arg(&plist),
            "read downloaded app version",
        )?;
        let version = String::from_utf8(output)
            .context("downloaded app version is not valid UTF-8")?
            .trim()
            .to_string();
        if version.is_empty() {
            bail!("downloaded app version is empty");
        }
        Ok(version)
    }

    fn resolve_target_app_path() -> anyhow::Result<PathBuf> {
        if let Some(path) = std::env::var_os("ARB_UPDATE_TARGET_APP") {
            let app = PathBuf::from(path);
            if app.ends_with("Arb.app") {
                return Ok(app);
            }
            bail!("ARB_UPDATE_TARGET_APP must point to Arb.app");
        }

        let exe = std::env::current_exe().context("resolve current executable")?;
        for ancestor in exe.ancestors() {
            if ancestor
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case("Arb.app"))
                .unwrap_or(false)
            {
                return Ok(ancestor.to_path_buf());
            }
        }

        let default_app = PathBuf::from("/Applications/Arb.app");
        if default_app.exists() {
            return Ok(default_app);
        }

        bail!("cannot locate installed Arb.app; run this from installed Arb")
    }

    fn ensure_can_write_target(target_app: &Path) -> anyhow::Result<()> {
        let parent = target_app
            .parent()
            .ok_or_else(|| anyhow!("invalid app path: {}", target_app.display()))?;
        if !parent.exists() {
            bail!(
                "install location does not exist: {}",
                parent.as_os_str().to_string_lossy()
            );
        }

        let test_file = parent.join(format!(".arb-update-write-test-{}", now_unix_seconds()));
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&test_file)
        {
            Ok(mut f) => {
                let _ = f.write_all(b"ok");
                let _ = fs::remove_file(test_file);
                Ok(())
            }
            Err(err) => bail!(
                "no write permission in {} ({})",
                parent.as_os_str().to_string_lossy(),
                err
            ),
        }
    }

    fn write_helper_script(script_path: &Path) -> anyhow::Result<()> {
        let script = r#"#!/bin/bash
set -euo pipefail

TARGET_APP="$1"
NEW_APP="$2"
WORK_DIR="$3"
LOG_FILE="$WORK_DIR/update.log"
BACKUP_APP="${TARGET_APP}.backup.$(date +%s)"
TARGET_GUI="$TARGET_APP/Contents/MacOS/arb-gui"
TARGET_CLI="$TARGET_APP/Contents/MacOS/arb"

log() {
  printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$1" >>"$LOG_FILE"
}

rollback() {
  log "restore from backup"
  /bin/rm -rf "$TARGET_APP" || true
  if [[ -d "$BACKUP_APP" ]]; then
    /bin/mv "$BACKUP_APP" "$TARGET_APP" || true
  fi
}

log "start apply update"

for _ in $(seq 1 20); do
  if /usr/bin/pgrep -f "$TARGET_GUI" >/dev/null 2>&1 || /usr/bin/pgrep -f "$TARGET_CLI" >/dev/null 2>&1; then
    /usr/bin/pkill -TERM -f "$TARGET_GUI" >/dev/null 2>&1 || true
    /usr/bin/pkill -TERM -f "$TARGET_CLI" >/dev/null 2>&1 || true
    sleep 1
  else
    break
  fi
done

/usr/bin/pkill -KILL -f "$TARGET_GUI" >/dev/null 2>&1 || true
/usr/bin/pkill -KILL -f "$TARGET_CLI" >/dev/null 2>&1 || true

if [[ -d "$TARGET_APP" ]]; then
  log "backup existing app"
  /bin/mv "$TARGET_APP" "$BACKUP_APP"
fi

log "copy new app"
if ! /usr/bin/ditto "$NEW_APP" "$TARGET_APP"; then
  rollback
  exit 1
fi

/usr/bin/xattr -cr "$TARGET_APP" >/dev/null 2>&1 || true

if [[ -d "$BACKUP_APP" ]]; then
  /bin/rm -rf "$BACKUP_APP" || true
fi

log "refresh shell integration"
"$TARGET_CLI" init --update-only >/dev/null 2>&1 || true

log "relaunch app"
/usr/bin/open "$TARGET_APP" >/dev/null 2>&1 || true

log "done"
/bin/rm -f "$0" >/dev/null 2>&1 || true
/bin/rm -rf "$WORK_DIR" >/dev/null 2>&1 || true
"#;

        fs::write(script_path, script).with_context(|| {
            format!(
                "failed to write helper script to {}",
                script_path.as_os_str().to_string_lossy()
            )
        })?;
        run_status(
            Command::new("/bin/chmod").arg("700").arg(script_path),
            "chmod update helper script",
        )?;
        Ok(())
    }

    fn spawn_update_helper(
        script: &Path,
        target_app: &Path,
        new_app: &Path,
        work_dir: &Path,
    ) -> anyhow::Result<()> {
        Command::new("/usr/bin/nohup")
            .arg("/bin/bash")
            .arg(script)
            .arg(target_app)
            .arg(new_app)
            .arg(work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("launch detached updater helper")?;
        Ok(())
    }

    fn run_output(cmd: &mut Command, context_text: &str) -> anyhow::Result<Vec<u8>> {
        let output = cmd
            .output()
            .with_context(|| format!("failed to {}", context_text))?;
        if output.status.success() {
            return Ok(output.stdout);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{} failed: {}", context_text, stderr.trim());
    }

    fn run_status(cmd: &mut Command, context_text: &str) -> anyhow::Result<()> {
        let status = cmd
            .status()
            .with_context(|| format!("failed to {}", context_text))?;
        if status.success() {
            return Ok(());
        }
        bail!("{} failed with status {}", context_text, status);
    }

    use arb_version::is_newer_version;

    fn format_version_for_display(version: &str) -> String {
        version.trim().trim_start_matches(['v', 'V']).to_string()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // --- sanitize_tag tests ---

        #[test]
        fn sanitize_tag_keeps_valid_tag_unchanged() {
            assert_eq!(sanitize_tag("v0.3.2"), "v0.3.2");
        }

        #[test]
        fn sanitize_tag_replaces_special_characters() {
            assert_eq!(sanitize_tag("v0.3.2-beta+build"), "v0.3.2-beta_build");
        }

        #[test]
        fn sanitize_tag_replaces_spaces_and_punctuation() {
            assert_eq!(sanitize_tag("hello world!"), "hello_world_");
        }

        // --- format_version_for_display tests ---

        #[test]
        fn format_version_for_display_strips_lowercase_v() {
            assert_eq!(format_version_for_display("v0.3.2"), "0.3.2");
        }

        #[test]
        fn format_version_for_display_strips_uppercase_v() {
            assert_eq!(format_version_for_display("V0.3.2"), "0.3.2");
        }

        #[test]
        fn format_version_for_display_no_prefix_unchanged() {
            assert_eq!(format_version_for_display("0.3.2"), "0.3.2");
        }

        #[test]
        fn format_version_for_display_trims_whitespace() {
            assert_eq!(format_version_for_display("  v0.3.2  "), "0.3.2");
        }

        // ── Task 2 (TODO.md): BREW_CASK_NAME and URL constant regression tests ──

        #[test]
        fn should_have_correct_brew_cask_name() {
            assert_eq!(
                BREW_CASK_NAME, "szj2ys/arb/arb",
                "BREW_CASK_NAME must be 'szj2ys/arb/arb'"
            );
        }

        #[test]
        fn should_have_correct_github_urls() {
            let urls = [
                ("RELEASE_API_URL", RELEASE_API_URL),
                ("LATEST_ZIP_URL", LATEST_ZIP_URL),
                ("LATEST_SHA_URL", LATEST_SHA_URL),
                ("RELEASE_LATEST_URL", RELEASE_LATEST_URL),
            ];

            for (name, url) in &urls {
                assert!(
                    url.contains("szj2ys/arb"),
                    "{} must contain 'szj2ys/arb', got: {}",
                    name,
                    url
                );
                assert!(
                    !url.contains("tw93"),
                    "{} must NOT contain 'tw93', got: {}",
                    name,
                    url
                );
            }
        }
    }
}
