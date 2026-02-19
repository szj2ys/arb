// Version comparison unit tests live inside `arb/src/update.rs` as a
// `#[cfg(test)] mod tests` block within the `#[cfg(target_os = "macos")] mod imp`
// module because the helper functions (`is_newer_version`, `compare_versions`,
// `parse_version_numbers`, `sanitize_tag`, `format_version_for_display`) are private.
//
// This file is kept as a placeholder so the test layout in `arb/tests/` matches the
// task structure described in TODO.md.  Run `cargo nextest run -p arb` to execute
// both the unit tests in update.rs and the integration tests in this directory.
