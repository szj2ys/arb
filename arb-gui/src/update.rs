use anyhow::anyhow;
use config::{configuration, arb_version};
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
        .header("User-Agent", &format!("arb/{}", arb_version()))
        .send(&mut latest)
        .map_err(|e| anyhow!("failed to query github releases: {}", e))?;

    let latest: Release = serde_json::from_slice(&latest)?;
    Ok(latest)
}

pub fn get_latest_release_info() -> anyhow::Result<Release> {
    get_github_release_info("https://api.github.com/repos/szj2ys/arb/releases/latest")
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

    let force_ui = std::env::var_os("ARB_ALWAYS_SHOW_UPDATE_UI").is_some();

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
                let current = arb_version();
                if is_newer(&latest.tag_name, current) || force_ui {
                    log::info!(
                        "latest release {} is newer than current build {}",
                        latest.tag_name,
                        current
                    );

                    let url = "https://github.com/szj2ys/arb/releases".to_string();

                    if force_ui || socks.is_empty() || socks[0] == my_sock {
                        persistent_toast_notification_with_click_to_open_url(
                            "Arb Update Available",
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
    use super::{compare_versions, is_newer, parse_version_numbers};
    use std::cmp::Ordering as CmpOrdering;

    // ── existing ────────────────────────────────────────────

    #[test]
    fn semver_numeric_comparison() {
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(!is_newer("0.2.0", "0.11.0"));
        assert!(!is_newer("0.1.1", "0.1.1"));
        assert!(is_newer("v0.1.2", "0.1.1"));
    }

    // ── Task 1: parse_version_numbers ───────────────────────

    #[test]
    fn should_parse_standard_semver() {
        assert_eq!(parse_version_numbers("1.2.3"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn should_parse_lowercase_v_prefix() {
        assert_eq!(parse_version_numbers("v1.0.0"), Some(vec![1, 0, 0]));
    }

    #[test]
    fn should_parse_uppercase_v_prefix() {
        assert_eq!(parse_version_numbers("V1.0.0"), Some(vec![1, 0, 0]));
    }

    #[test]
    fn should_return_none_for_empty_string() {
        assert_eq!(parse_version_numbers(""), None);
    }

    #[test]
    fn should_return_none_for_pure_non_numeric() {
        assert_eq!(parse_version_numbers("abc"), None);
    }

    #[test]
    fn should_parse_version_with_prerelease_suffix() {
        assert_eq!(parse_version_numbers("1.2.3-beta"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn should_parse_single_number() {
        assert_eq!(parse_version_numbers("5"), Some(vec![5]));
    }

    #[test]
    fn should_parse_four_segments() {
        assert_eq!(
            parse_version_numbers("1.2.3.4"),
            Some(vec![1, 2, 3, 4])
        );
    }

    // ── Task 2: compare_versions ────────────────────────────

    #[test]
    fn should_compare_equal_when_shorter_padded_with_zeros() {
        assert_eq!(
            compare_versions("1.0", "1.0.0"),
            Some(CmpOrdering::Equal)
        );
    }

    #[test]
    fn should_compare_greater_when_left_longer_with_nonzero() {
        assert_eq!(
            compare_versions("1.0.1", "1.0"),
            Some(CmpOrdering::Greater)
        );
    }

    #[test]
    fn should_compare_less_when_right_longer_with_nonzero() {
        assert_eq!(
            compare_versions("1.0", "1.0.1"),
            Some(CmpOrdering::Less)
        );
    }

    #[test]
    fn should_return_none_when_left_invalid() {
        assert_eq!(compare_versions("abc", "1.0"), None);
    }

    #[test]
    fn should_return_none_when_right_invalid() {
        assert_eq!(compare_versions("1.0", "xyz"), None);
    }

    // ── Task 3: is_newer edge cases ─────────────────────────

    #[test]
    fn should_reject_date_version_vs_semver() {
        // WezTerm date-version guard: date latest against semver current → false
        assert!(!is_newer("20240203-110000-abc", "0.1.0"));
    }

    #[test]
    fn should_compare_two_date_versions() {
        // Both are date versions; guard does not fire, numeric compare succeeds
        assert!(is_newer("20240204-110000-abc", "20240203-110000-abc"));
    }

    #[test]
    fn should_fallback_to_not_equal_when_unparseable_and_different() {
        // Neither parses → compare_versions returns None → latest != current → true
        assert!(is_newer("abc", "def"));
    }

    #[test]
    fn should_fallback_to_not_equal_when_unparseable_and_same() {
        // Neither parses → compare_versions returns None → latest == current → false
        assert!(!is_newer("abc", "abc"));
    }

    #[test]
    fn should_handle_uppercase_v_prefix_in_is_newer() {
        assert!(is_newer("V0.2.0", "v0.1.0"));
    }
}
