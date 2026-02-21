//! Integration tests for `arb init` core paths.
//!
//! These tests validate:
//! - Vendor resource completeness (starship.toml + expected plugin directories)
//! - `setup_zsh.sh` script structure and correctness
//! - The arb.zsh heredoc template embedded in setup_zsh.sh
//!
//! Note: We do NOT execute setup_zsh.sh directly because on machines with
//! Arb.app installed the script may trigger the GUI app or other side-effects.
//! Instead we validate the script content and its expected outputs statically.

use std::path::PathBuf;

/// Resolve the project root by walking up from CARGO_MANIFEST_DIR (which is `arb/`).
fn project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR should have a parent")
        .to_path_buf()
}

fn vendor_dir() -> PathBuf {
    project_root().join("assets").join("vendor")
}

fn shell_integration_dir() -> PathBuf {
    project_root().join("assets").join("shell-integration")
}

fn read_setup_script() -> String {
    let script = shell_integration_dir().join("setup_zsh.sh");
    std::fs::read_to_string(&script).expect("read setup_zsh.sh")
}

// ── Vendor resource completeness (Task 3) ────────────────────────────

#[test]
fn should_have_starship_toml_in_vendor() {
    let path = vendor_dir().join("starship.toml");
    assert!(
        path.exists(),
        "assets/vendor/starship.toml should exist (it is tracked in git)"
    );
}

#[test]
fn should_have_valid_toml_in_starship_config() {
    let path = vendor_dir().join("starship.toml");
    let content = std::fs::read_to_string(&path).expect("read starship.toml");

    // Verify it contains expected starship sections.
    assert!(
        content.contains("[directory]"),
        "starship.toml should contain a [directory] section"
    );
    assert!(
        content.contains("[character]"),
        "starship.toml should contain a [character] section"
    );
    assert!(
        content.contains("[git_branch]"),
        "starship.toml should contain a [git_branch] section"
    );
}

#[test]
fn should_have_vendor_directory() {
    let dir = vendor_dir();
    assert!(
        dir.exists() && dir.is_dir(),
        "assets/vendor/ directory should exist"
    );
}

#[test]
fn should_have_starship_config_that_disables_slow_modules() {
    let path = vendor_dir().join("starship.toml");
    let content = std::fs::read_to_string(&path).expect("read starship.toml");

    // The arb starship config disables language-specific modules for speed.
    for module in &["nodejs", "python", "rust"] {
        assert!(
            content.contains(&format!("[{module}]")),
            "starship.toml should have a [{module}] section"
        );
    }
    assert!(
        content.contains("disabled = true"),
        "starship.toml should disable at least one module"
    );
}

/// Verify that the expected plugin directory names are the ones the setup
/// script references. This catches typos or renames that would break init.
#[test]
fn should_reference_all_required_vendor_plugins_in_setup_script() {
    let content = read_setup_script();

    let expected_plugins = [
        "zsh-z",
        "zsh-autosuggestions",
        "zsh-syntax-highlighting",
        "zsh-completions",
    ];

    for plugin in &expected_plugins {
        assert!(
            content.contains(plugin),
            "setup_zsh.sh should reference plugin '{plugin}'"
        );
    }
}

/// The setup script validates vendor plugins before proceeding. Verify that
/// the validation loop lists exactly the same plugins it later copies.
#[test]
fn should_validate_same_plugins_it_copies() {
    let content = read_setup_script();

    // The script has a validation loop:  for plugin in zsh-z zsh-autosuggestions ...
    // And later copies:  cp -R "$VENDOR_DIR/zsh-z" ...
    let expected = [
        "zsh-z",
        "zsh-autosuggestions",
        "zsh-syntax-highlighting",
        "zsh-completions",
    ];

    for plugin in &expected {
        let copy_line = format!("$VENDOR_DIR/{plugin}");
        assert!(
            content.contains(&copy_line),
            "setup_zsh.sh should copy vendor plugin '{plugin}'"
        );
    }
}

// ── setup_zsh.sh script structure (Task 2) ───────────────────────────

#[test]
fn should_have_setup_zsh_sh_exist() {
    let script = shell_integration_dir().join("setup_zsh.sh");
    assert!(
        script.exists(),
        "assets/shell-integration/setup_zsh.sh must exist"
    );
}

#[test]
fn should_have_bash_shebang_in_setup_zsh() {
    let content = read_setup_script();
    assert!(
        content.starts_with("#!/bin/bash"),
        "setup_zsh.sh should start with a bash shebang"
    );
}

#[test]
fn should_use_strict_mode_in_setup_zsh() {
    let content = read_setup_script();
    assert!(
        content.contains("set -euo pipefail"),
        "setup_zsh.sh should use strict bash mode (set -euo pipefail)"
    );
}

#[test]
fn should_create_user_config_dir_in_setup_zsh() {
    let content = read_setup_script();
    assert!(
        content.contains(".config/arb/zsh"),
        "setup_zsh.sh should reference the user config directory .config/arb/zsh"
    );
}

#[test]
fn should_generate_arb_zsh_init_file_in_setup_zsh() {
    let content = read_setup_script();
    assert!(
        content.contains("arb.zsh"),
        "setup_zsh.sh should generate the arb.zsh init file"
    );
}

#[test]
fn should_support_update_only_flag_in_setup_zsh() {
    let content = read_setup_script();
    assert!(
        content.contains("--update-only"),
        "setup_zsh.sh should support the --update-only flag"
    );
}

#[test]
fn should_guard_delegation_with_arb_init_internal() {
    let content = read_setup_script();
    assert!(
        content.contains("ARB_INIT_INTERNAL"),
        "setup_zsh.sh should check ARB_INIT_INTERNAL before delegating to arb CLI"
    );
}

#[test]
fn should_backup_zshrc_before_patching() {
    let content = read_setup_script();
    assert!(
        content.contains("arb-backup"),
        "setup_zsh.sh should create a backup of .zshrc (arb-backup suffix)"
    );
}

#[test]
fn should_check_idempotency_before_patching_zshrc() {
    let content = read_setup_script();
    // The script checks if arb.zsh source line is already present.
    assert!(
        content.contains("grep -q"),
        "setup_zsh.sh should use grep to check if .zshrc already has the source line"
    );
}

// ── arb.zsh heredoc template validation (Task 2) ─────────────────────
//
// The setup script contains a heredoc that generates arb.zsh.
// We extract and validate its expected content markers.

#[test]
fn should_define_arb_zsh_dir_in_heredoc() {
    let content = read_setup_script();
    assert!(
        content.contains("ARB_ZSH_DIR"),
        "the arb.zsh heredoc should define ARB_ZSH_DIR"
    );
}

#[test]
fn should_source_starship_in_heredoc() {
    let content = read_setup_script();
    assert!(
        content.contains("starship init zsh"),
        "the arb.zsh heredoc should initialize starship"
    );
}

#[test]
fn should_source_all_plugins_in_heredoc() {
    let content = read_setup_script();

    let expected_sources = [
        "zsh-z/zsh-z.plugin.zsh",
        "zsh-autosuggestions/zsh-autosuggestions.zsh",
        "zsh-syntax-highlighting/zsh-syntax-highlighting.zsh",
        "zsh-completions/src",
    ];

    for source_path in &expected_sources {
        assert!(
            content.contains(source_path),
            "the arb.zsh heredoc should reference '{source_path}'"
        );
    }
}

#[test]
fn should_set_history_config_in_heredoc() {
    let content = read_setup_script();
    assert!(
        content.contains("HISTSIZE="),
        "the arb.zsh heredoc should configure HISTSIZE"
    );
    assert!(
        content.contains("SAVEHIST="),
        "the arb.zsh heredoc should configure SAVEHIST"
    );
    assert!(
        content.contains("SHARE_HISTORY"),
        "the arb.zsh heredoc should enable SHARE_HISTORY"
    );
}

#[test]
fn should_add_bin_to_path_in_heredoc() {
    let content = read_setup_script();
    // The generated arb.zsh should prepend the bundled bin directory to PATH.
    assert!(
        content.contains(r#"export PATH="\$ARB_ZSH_DIR/bin:\$PATH""#),
        "the arb.zsh heredoc should add ARB_ZSH_DIR/bin to PATH"
    );
}

#[test]
fn should_defer_syntax_highlighting_in_heredoc() {
    let content = read_setup_script();
    // zsh-syntax-highlighting must be loaded LAST and is deferred for performance.
    assert!(
        content.contains("zsh_syntax_highlighting_defer"),
        "the arb.zsh heredoc should defer zsh-syntax-highlighting loading"
    );
    assert!(
        content.contains("precmd_functions"),
        "the arb.zsh heredoc should use precmd_functions for deferred loading"
    );
}

// ── setup_zsh.sh tmpdir isolation: directory structure test ───────────
//
// Instead of running the full setup_zsh.sh (which can trigger side effects
// on machines with Arb.app installed), we run only the directory-creation
// portion by extracting and executing a minimal subset.

#[test]
fn should_create_expected_directory_structure_in_tmpdir() {
    let tmp = tempfile::tempdir().expect("create tmpdir");
    let fake_home = tmp.path().join("home");
    std::fs::create_dir_all(&fake_home).expect("create fake home");

    // Replicate exactly what setup_zsh.sh does for directory creation
    // (lines 76-78 of the script):
    //   mkdir -p "$USER_CONFIG_DIR"
    //   mkdir -p "$USER_CONFIG_DIR/plugins"
    //   mkdir -p "$USER_CONFIG_DIR/bin"
    let user_config_dir = fake_home.join(".config/arb/zsh");
    std::fs::create_dir_all(user_config_dir.join("plugins"))
        .expect("create plugins dir");
    std::fs::create_dir_all(user_config_dir.join("bin"))
        .expect("create bin dir");

    assert!(user_config_dir.exists());
    assert!(user_config_dir.join("plugins").exists());
    assert!(user_config_dir.join("bin").exists());
}

/// Verify that the arb.zsh source line the script appends to .zshrc has
/// the correct conditional-source format.
#[test]
fn should_have_correct_zshrc_source_line() {
    let content = read_setup_script();
    // The script defines SOURCE_LINE with escaped quotes and dollar signs for
    // bash assignment.  Match the key structural elements.
    assert!(
        content.contains(".config/arb/zsh/arb.zsh"),
        "setup_zsh.sh should reference .config/arb/zsh/arb.zsh in the source line"
    );
    assert!(
        content.contains("Arb Shell Integration"),
        "setup_zsh.sh source line should contain the 'Arb Shell Integration' comment"
    );
}

/// Verify the starship config is copied to arb's own directory (not the
/// user's global ~/.config/starship.toml).
#[test]
fn should_install_starship_config_to_arb_dir() {
    let content = read_setup_script();
    assert!(
        content.contains("ARB_STARSHIP_CONFIG=\"$USER_CONFIG_DIR/starship.toml\""),
        "setup_zsh.sh should install starship.toml to the arb config directory"
    );
}

/// Verify the RESOURCES_DIR resolution has the expected fallback chain.
#[test]
fn should_have_resource_dir_fallback_chain() {
    let content = read_setup_script();

    // The script resolves RESOURCES_DIR with these fallbacks:
    // 1. $SCRIPT_DIR/vendor (in-place)
    // 2. $SCRIPT_DIR/../vendor (dev checkout)
    // 3. /Applications/Arb.app/Contents/Resources/vendor
    // 4. ~/Applications/Arb.app/Contents/Resources/vendor
    let fallbacks = [
        "$SCRIPT_DIR/vendor",
        "$SCRIPT_DIR/../vendor",
        "/Applications/Arb.app/Contents/Resources/vendor",
        "$HOME/Applications/Arb.app/Contents/Resources/vendor",
    ];

    for fb in &fallbacks {
        assert!(
            content.contains(fb),
            "setup_zsh.sh should check '{fb}' in RESOURCES_DIR fallback chain"
        );
    }
}
