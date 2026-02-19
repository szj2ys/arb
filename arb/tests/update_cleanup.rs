#[cfg(target_os = "macos")]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    fn create_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    fn now_unix_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn should_cleanup_old_update_dirs_when_older_than_7_days() {
        let temp = tempfile::tempdir().unwrap();
        let update_root = temp.path().join("updates");
        create_dir(&update_root);

        let eight_days_secs = Duration::from_secs(60 * 60 * 24 * 8).as_secs();
        let now = now_unix_secs();

        let old_dir = update_root.join(format!("v0.0.0-{}", now - eight_days_secs));
        create_dir(&old_dir);

        let new_dir = update_root.join(format!("v0.0.0-{}", now));
        create_dir(&new_dir);

        let pending_dir = update_root.join("pending");
        create_dir(&pending_dir);

        arb::update::cleanup_old_update_dirs_for_tests(&update_root).unwrap();

        assert!(!old_dir.exists());
        assert!(new_dir.exists());
        assert!(pending_dir.exists());
    }

    // ── Task 3 (TODO.md): update_cleanup edge cases ───────

    #[test]
    fn should_keep_recent_update_dirs() {
        let temp = tempfile::tempdir().unwrap();
        let update_root = temp.path().join("updates");
        create_dir(&update_root);

        let now = now_unix_secs();

        // A directory created 1 hour ago (well within the 7-day TTL).
        let one_hour_ago = now - 3600;
        let recent_dir = update_root.join(format!("v0.2.0-{}", one_hour_ago));
        create_dir(&recent_dir);

        // A directory created just now.
        let just_now_dir = update_root.join(format!("v0.3.0-{}", now));
        create_dir(&just_now_dir);

        // A directory created 6 days ago (still within 7-day TTL).
        let six_days_ago = now - Duration::from_secs(60 * 60 * 24 * 6).as_secs();
        let six_day_dir = update_root.join(format!("v0.1.0-{}", six_days_ago));
        create_dir(&six_day_dir);

        arb::update::cleanup_old_update_dirs_for_tests(&update_root).unwrap();

        assert!(
            recent_dir.exists(),
            "directory from 1 hour ago should be kept"
        );
        assert!(
            just_now_dir.exists(),
            "directory from just now should be kept"
        );
        assert!(
            six_day_dir.exists(),
            "directory from 6 days ago should be kept (within 7-day TTL)"
        );
    }

    #[test]
    fn should_ignore_non_directory_entries() {
        let temp = tempfile::tempdir().unwrap();
        let update_root = temp.path().join("updates");
        create_dir(&update_root);

        let now = now_unix_secs();
        let eight_days_secs = Duration::from_secs(60 * 60 * 24 * 8).as_secs();

        // Create a regular file (not a directory) with a name that looks old.
        let old_file = update_root.join(format!("v0.0.0-{}", now - eight_days_secs));
        fs::write(&old_file, "not a directory").unwrap();

        // Create another regular file with a normal name.
        let normal_file = update_root.join("some-metadata.json");
        fs::write(&normal_file, "{}").unwrap();

        arb::update::cleanup_old_update_dirs_for_tests(&update_root).unwrap();

        // Non-directory entries should be left untouched.
        assert!(
            old_file.exists(),
            "regular files should not be cleaned up even if name looks old"
        );
        assert!(
            normal_file.exists(),
            "regular files should not be affected by cleanup"
        );
    }

    #[test]
    fn should_handle_empty_updates_dir() {
        let temp = tempfile::tempdir().unwrap();
        let update_root = temp.path().join("updates");
        create_dir(&update_root);

        // An empty updates directory should not cause a panic or error.
        let result = arb::update::cleanup_old_update_dirs_for_tests(&update_root);
        assert!(
            result.is_ok(),
            "cleanup on empty updates directory should succeed without error"
        );
        assert!(
            update_root.exists(),
            "empty updates directory should still exist after cleanup"
        );
    }
}
