pub fn arb_version() -> &'static str {
    // See build.rs
    env!("WEZTERM_CI_TAG")
}

pub fn arb_target_triple() -> &'static str {
    // See build.rs
    env!("WEZTERM_TARGET_TRIPLE")
}

#[deprecated(note = "Use arb_version() instead")]
pub fn wezterm_version() -> &'static str {
    arb_version()
}

#[deprecated(note = "Use arb_target_triple() instead")]
pub fn wezterm_target_triple() -> &'static str {
    arb_target_triple()
}
