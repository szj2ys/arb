pub fn arb_version() -> &'static str {
    // See build.rs
    env!("ARB_CI_TAG")
}

pub fn arb_target_triple() -> &'static str {
    // See build.rs
    env!("ARB_TARGET_TRIPLE")
}
