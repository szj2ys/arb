use anyhow::anyhow;
use config::{configuration, wezterm_version};
use http_req::request::{HttpVersion, Request};
use http_req::uri::Uri;
use serde::*;
use std::cmp::Ordering as CmpOrdering;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use wezterm_toast_notification::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub url: String,
    pub body: String,
    pub html_url: String,
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub size: usize,
    pub url: String,
    pub browser_download_url: String,
}

fn get_github_release_info(uri: &str) -> anyhow::Result<Release> {
    let uri = Uri::try_from(uri)?;

    let mut latest = Vec::new();
    let _res = Request::new(&uri)
        .version(HttpVersion::Http10)
        .header("User-Agent", &format!("kaku/{}", wezterm_version()))
        .send(&mut latest)
        .map_err(|e| anyhow!("failed to query github releases: {}", e))?;

    /*
    println!("Status: {} {}", _res.status_code(), _res.reason());
    println!("{}", String::from_utf8_lossy(&latest));
    */

    let latest: Release = serde_json::from_slice(&latest)?;
    Ok(latest)
}

pub fn get_latest_release_info() -> anyhow::Result<Release> {
    get_github_release_info("https://api.github.com/repos/tw93/Kaku/releases/latest")
}

#[allow(unused)]
pub fn get_nightly_release_info() -> anyhow::Result<Release> {
    get_github_release_info("https://api.github.com/repos/wezterm/wezterm/releases/tags/nightly")
}

fn is_newer(latest: &str, current: &str) -> bool {
    let latest = latest.trim_start_matches('v');
    let current = current.trim_start_matches('v');

    // If latest is a WezTerm-style date version (e.g. 20240203-...) and current is SemVer (e.g. 0.1.0),
    // treat the date version as older/different system.
    if latest.starts_with("20") && latest.contains('-') && !current.starts_with("20") {
        return false;
    }

    match compare_versions(latest, current) {
        Some(CmpOrdering::Greater) => true,
        Some(_) => false,
        None => latest != current,
    }
}

fn compare_versions(left: &str, right: &str) -> Option<CmpOrdering> {
    let left = parse_version_numbers(left)?;
    let right = parse_version_numbers(right)?;
    let max_len = left.len().max(right.len());
    for idx in 0..max_len {
        let l = left.get(idx).copied().unwrap_or(0);
        let r = right.get(idx).copied().unwrap_or(0);
        match l.cmp(&r) {
            CmpOrdering::Equal => {}
            non_eq => return Some(non_eq),
        }
    }
    Some(CmpOrdering::Equal)
}

fn parse_version_numbers(version: &str) -> Option<Vec<u64>> {
    let cleaned = version.trim().trim_start_matches(['v', 'V']);
    let mut out = Vec::new();
    for part in cleaned.split('.') {
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        let value = digits.parse::<u64>().ok()?;
        out.push(value);
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

fn update_checker() {
    // Compute how long we should sleep for;
    // if we've never checked, give it a few seconds after the first
    // launch, otherwise compute the interval based on the time of
    // the last check.
    let update_interval = Duration::from_secs(configuration().check_for_updates_interval_seconds);
    let initial_interval = Duration::from_secs(10);

    let force_ui = std::env::var_os("KAKU_ALWAYS_SHOW_UPDATE_UI").is_some();

    let update_file_name = config::DATA_DIR.join("check_update");
    let delay = update_file_name
        .metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|_| ())
        .and_then(|systime| {
            let elapsed = systime.elapsed().unwrap_or(Duration::new(0, 0));
            update_interval.checked_sub(elapsed).ok_or(())
        })
        .unwrap_or(initial_interval);

    std::thread::sleep(if force_ui { initial_interval } else { delay });

    let my_sock = config::RUNTIME_DIR.join(format!("gui-sock-{}", unsafe { libc::getpid() }));

    loop {
        // Figure out which other wezterm-guis are running.
        // We have a little "consensus protocol" to decide which
        // of us will show the toast notification or show the update
        // window: the one of us that sorts first in the list will
        // own doing that, so that if there are a dozen gui processes
        // running, we don't spam the user with a lot of notifications.
        let socks = wezterm_client::discovery::discover_gui_socks();

        if configuration().check_for_updates {
            if let Ok(latest) = get_latest_release_info() {
                let current = wezterm_version();
                if is_newer(&latest.tag_name, current) || force_ui {
                    log::info!(
                        "latest release {} is newer than current build {}",
                        latest.tag_name,
                        current
                    );

                    let url = "https://github.com/tw93/Kaku/releases".to_string();

                    if force_ui || socks.is_empty() || socks[0] == my_sock {
                        persistent_toast_notification_with_click_to_open_url(
                            "Kaku Update Available",
                            "Click to download from releases",
                            &url,
                        );
                    }
                }

                config::create_user_owned_dirs(update_file_name.parent().unwrap()).ok();

                // Record the time of this check
                if let Ok(f) = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&update_file_name)
                {
                    serde_json::to_writer_pretty(f, &latest).ok();
                }
            }
        }

        std::thread::sleep(Duration::from_secs(
            configuration().check_for_updates_interval_seconds,
        ));
    }
}

pub fn start_update_checker() {
    static CHECKER_STARTED: AtomicBool = AtomicBool::new(false);
    if let Ok(false) =
        CHECKER_STARTED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
    {
        std::thread::Builder::new()
            .name("update_checker".into())
            .spawn(update_checker)
            .expect("failed to spawn update checker thread");
    }
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn semver_numeric_comparison() {
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(!is_newer("0.2.0", "0.11.0"));
        assert!(!is_newer("0.1.1", "0.1.1"));
        assert!(is_newer("v0.1.2", "0.1.1"));
    }
}
