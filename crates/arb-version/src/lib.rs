//! Shared version comparison utilities for Arb.
//!
//! Extracts the duplicated version parsing/comparison logic that was present
//! in both `arb/src/update.rs` (CLI) and `arb-gui/src/update.rs` (GUI).

use std::cmp::Ordering;

/// Parse a version string into a vector of numeric segments.
///
/// Strips a leading `v` or `V` prefix, then splits on `.` and extracts the
/// leading digits from each segment. Returns `None` if any segment has no
/// leading digits, or if the string is empty after trimming.
///
/// # Examples
/// ```
/// use arb_version::parse_version_numbers;
/// assert_eq!(parse_version_numbers("v1.2.3"), Some(vec![1, 2, 3]));
/// assert_eq!(parse_version_numbers("1.0.0-beta"), Some(vec![1, 0, 0]));
/// assert_eq!(parse_version_numbers("not-a-version"), None);
/// ```
pub fn parse_version_numbers(version: &str) -> Option<Vec<u64>> {
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

/// Compare two version strings numerically, segment by segment.
///
/// Shorter version vectors are implicitly padded with zeros on the right.
/// Returns `None` if either version string cannot be parsed.
///
/// # Examples
/// ```
/// use arb_version::compare_versions;
/// use std::cmp::Ordering;
/// assert_eq!(compare_versions("1.2.3", "1.2.3"), Some(Ordering::Equal));
/// assert_eq!(compare_versions("1.0.1", "1.0"), Some(Ordering::Greater));
/// ```
pub fn compare_versions(left: &str, right: &str) -> Option<Ordering> {
    let left = parse_version_numbers(left)?;
    let right = parse_version_numbers(right)?;
    let max_len = left.len().max(right.len());
    for idx in 0..max_len {
        let l = left.get(idx).copied().unwrap_or(0);
        let r = right.get(idx).copied().unwrap_or(0);
        match l.cmp(&r) {
            Ordering::Equal => {}
            non_eq => return Some(non_eq),
        }
    }
    Some(Ordering::Equal)
}

/// Return `true` if `latest` represents a strictly newer version than `current`.
///
/// Includes a guard for WezTerm-style date versions (e.g. `20240203-110000-abc`):
/// if `latest` looks like a date version (starts with `"20"` and contains `'-'`)
/// while `current` does not, the function returns `false` because the two version
/// schemes are incomparable.
///
/// When neither version can be parsed, falls back to a raw string comparison
/// (after stripping the `v`/`V` prefix): returns `true` if they differ, `false`
/// if they are identical.
///
/// # Examples
/// ```
/// use arb_version::is_newer_version;
/// assert!(is_newer_version("0.4.0", "0.3.2"));
/// assert!(!is_newer_version("0.3.2", "0.3.2"));
/// assert!(!is_newer_version("20240203-110000-abc", "0.1.0"));
/// ```
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let latest_trimmed = latest.trim_start_matches(['v', 'V']);
    let current_trimmed = current.trim_start_matches(['v', 'V']);

    // If latest is a WezTerm-style date version (e.g. 20240203-...) and current
    // is SemVer (e.g. 0.1.0), treat the date version as an incompatible/older system.
    if latest_trimmed.starts_with("20")
        && latest_trimmed.contains('-')
        && !current_trimmed.starts_with("20")
    {
        return false;
    }

    match compare_versions(latest, current) {
        Some(Ordering::Greater) => true,
        Some(_) => false,
        None => latest_trimmed != current_trimmed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    // ── parse_version_numbers ──────────────────────────────────

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
        assert_eq!(parse_version_numbers("V1.2.3"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn should_return_none_for_empty_string() {
        assert_eq!(parse_version_numbers(""), None);
    }

    #[test]
    fn should_return_none_for_pure_non_numeric() {
        assert_eq!(parse_version_numbers("abc"), None);
        assert_eq!(parse_version_numbers("not-a-version"), None);
    }

    #[test]
    fn should_parse_version_with_prerelease_suffix() {
        assert_eq!(parse_version_numbers("1.2.3-beta"), Some(vec![1, 2, 3]));
        assert_eq!(parse_version_numbers("0.3.2-beta"), Some(vec![0, 3, 2]));
    }

    #[test]
    fn should_parse_single_number() {
        assert_eq!(parse_version_numbers("5"), Some(vec![5]));
    }

    #[test]
    fn should_parse_four_segments() {
        assert_eq!(parse_version_numbers("1.2.3.4"), Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn should_parse_single_date_style_segment() {
        // A bare date-style string like "20230101" parses as a single segment.
        assert_eq!(parse_version_numbers("20230101"), Some(vec![20230101]));
    }

    // ── compare_versions ───────────────────────────────────────

    #[test]
    fn should_compare_equal_versions() {
        assert_eq!(compare_versions("0.3.2", "0.3.2"), Some(Ordering::Equal));
    }

    #[test]
    fn should_compare_greater_numeric_not_lexicographic() {
        // 10 > 9 numerically, but "10" < "9" lexicographically
        assert_eq!(compare_versions("0.3.10", "0.3.9"), Some(Ordering::Greater));
    }

    #[test]
    fn should_compare_less() {
        assert_eq!(compare_versions("0.3.1", "0.3.2"), Some(Ordering::Less));
    }

    #[test]
    fn should_return_none_for_garbage() {
        assert_eq!(compare_versions("foo", "bar"), None);
    }

    #[test]
    fn should_compare_equal_when_shorter_padded_with_zeros() {
        assert_eq!(compare_versions("1.0", "1.0.0"), Some(Ordering::Equal));
    }

    #[test]
    fn should_compare_greater_when_left_longer_with_nonzero() {
        assert_eq!(compare_versions("1.0.1", "1.0"), Some(Ordering::Greater));
    }

    #[test]
    fn should_compare_less_when_right_longer_with_nonzero() {
        assert_eq!(compare_versions("1.0", "1.0.1"), Some(Ordering::Less));
    }

    #[test]
    fn should_return_none_when_left_invalid() {
        assert_eq!(compare_versions("abc", "1.0"), None);
    }

    #[test]
    fn should_return_none_when_right_invalid() {
        assert_eq!(compare_versions("1.0", "xyz"), None);
    }

    // ── is_newer_version ───────────────────────────────────────

    #[test]
    fn should_return_true_when_latest_is_newer() {
        assert!(is_newer_version("0.4.0", "0.3.2"));
    }

    #[test]
    fn should_return_false_when_versions_equal() {
        assert!(!is_newer_version("0.3.2", "0.3.2"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }

    #[test]
    fn should_return_false_when_latest_is_older() {
        assert!(!is_newer_version("0.3.1", "0.3.2"));
        assert!(!is_newer_version("0.1.0", "0.2.0"));
    }

    #[test]
    fn should_handle_v_prefix() {
        assert!(is_newer_version("v0.4.0", "0.3.2"));
        assert!(is_newer_version("v0.1.2", "0.1.1"));
        assert!(is_newer_version("V0.2.0", "v0.1.0"));
    }

    #[test]
    fn should_handle_major_bump_beats_high_minor() {
        assert!(is_newer_version("1.0.0", "0.99.99"));
    }

    #[test]
    fn should_compare_semver_correctly() {
        // Minor version bump
        assert!(is_newer_version("0.2.0", "0.1.99"));
        // Patch version bump
        assert!(is_newer_version("0.1.2", "0.1.1"));
    }

    #[test]
    fn should_handle_numeric_comparison_over_lexicographic() {
        assert!(is_newer_version("0.1.10", "0.1.9"));
        assert!(!is_newer_version("0.2.0", "0.11.0"));
        assert!(!is_newer_version("0.1.1", "0.1.1"));
    }

    // ── WezTerm date-version guard ─────────────────────────────

    #[test]
    fn should_reject_date_version_vs_semver() {
        // WezTerm date-version guard: date latest against semver current -> false
        assert!(!is_newer_version("20240203-110000-abc", "0.1.0"));
        assert!(!is_newer_version("20251231-235959-xyz", "0.3.2"));
        assert!(!is_newer_version("20200101-000000-000", "1.0.0"));
    }

    #[test]
    fn should_compare_two_date_versions() {
        // Both are date versions; guard does not fire, numeric compare succeeds
        assert!(is_newer_version("20240204-110000-abc", "20240203-110000-abc"));
    }

    // ── fallback when unparseable ──────────────────────────────

    #[test]
    fn should_fallback_to_not_equal_when_unparseable_and_different() {
        // Neither parses -> compare_versions returns None -> latest != current -> true
        assert!(is_newer_version("abc", "def"));
    }

    #[test]
    fn should_fallback_to_not_equal_when_unparseable_and_same() {
        // Neither parses -> compare_versions returns None -> latest == current -> false
        assert!(!is_newer_version("abc", "abc"));
    }
}
