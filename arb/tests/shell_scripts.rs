/// Integration tests for shell scripts in assets/shell-integration/.
///
/// These tests validate that the shell scripts bundled with Arb are syntactically
/// valid bash and contain expected content markers. This prevents regressions where
/// a broken script ships to users and breaks their first-run experience.

mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Resolve the project root by walking up from CARGO_MANIFEST_DIR (which is `arb/`).
    fn project_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .expect("CARGO_MANIFEST_DIR should have a parent")
            .to_path_buf()
    }

    fn shell_integration_dir() -> PathBuf {
        project_root().join("assets").join("shell-integration")
    }

    fn assert_valid_bash_syntax(script: &Path) {
        assert!(
            script.exists(),
            "script does not exist: {}",
            script.display()
        );

        let output = Command::new("bash")
            .arg("-n")
            .arg(script)
            .output()
            .expect("failed to run bash -n for syntax check");

        assert!(
            output.status.success(),
            "bash syntax check failed for {}: {}",
            script.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn should_have_valid_bash_syntax_first_run() {
        let script = shell_integration_dir().join("first_run.sh");
        assert_valid_bash_syntax(&script);
    }

    #[test]
    fn should_have_valid_bash_syntax_setup_zsh() {
        let script = shell_integration_dir().join("setup_zsh.sh");
        assert_valid_bash_syntax(&script);
    }

    #[test]
    fn should_have_all_expected_shell_scripts() {
        let dir = shell_integration_dir();
        let expected_scripts = ["first_run.sh", "setup_zsh.sh", "install_delta.sh"];

        for name in &expected_scripts {
            let script = dir.join(name);
            assert!(
                script.exists(),
                "expected shell script {} not found in {}",
                name,
                dir.display()
            );
        }
    }

    #[test]
    fn should_reference_current_config_version() {
        let script = shell_integration_dir().join("first_run.sh");
        let content = std::fs::read_to_string(&script).expect("read first_run.sh");

        // first_run.sh persists a config version number.  The current version is 6.
        // This test ensures the script writes the expected version so that the
        // gui-startup handler can detect whether re-onboarding is needed.
        assert!(
            content.contains("echo \"6\""),
            "first_run.sh should contain the current config version number (6)"
        );
    }
}
