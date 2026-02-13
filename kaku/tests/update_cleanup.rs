#[cfg(target_os = "macos")]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    fn create_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    #[test]
    fn should_cleanup_old_update_dirs_when_older_than_7_days() {
        let temp = tempfile::tempdir().unwrap();
        let update_root = temp.path().join("updates");
        create_dir(&update_root);

        let eight_days_secs = Duration::from_secs(60 * 60 * 24 * 8).as_secs();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let old_dir = update_root.join(format!("v0.0.0-{}", now - eight_days_secs));
        create_dir(&old_dir);

        let new_dir = update_root.join(format!("v0.0.0-{}", now));
        create_dir(&new_dir);

        let pending_dir = update_root.join("pending");
        create_dir(&pending_dir);

        kaku::update::cleanup_old_update_dirs_for_tests(&update_root).unwrap();

        assert!(!old_dir.exists());
        assert!(new_dir.exists());
        assert!(pending_dir.exists());
    }
}
