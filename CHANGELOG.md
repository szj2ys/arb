# Changelog

## [0.3.2] - 2026-02-16

### Fixes
- fix: statically link harfbuzz/freetype to eliminate Homebrew dylib dependency
- fix: add vendored submodules (harfbuzz, freetype2, libpng, zlib) for portable builds
- fix: CI/release workflows now checkout submodules for successful cross-arch builds
- fix: rename remaining Kaku references to Arb in issue templates and first_run.sh

## [0.3.1] - 2026-02-15

### Changes
- Rename project identifiers from kaku to arb.
- Improve build robustness when vendored submodules are missing (use system freetype/harfbuzz via pkg-config).
- Fix macOS app bundle naming/resources and packaging.

## [0.3.0] - 2026-02-14

### Features
- feat(tabbar): add vertical tab bar and rename support
- feat(themes): add curated theme switching (#4)
- feat(update): stage updates and apply with --apply
- feat(macos): add Cmd+F search binding

### Fixes
- fix(macos): handle titlebar double-click zoom

### Chores
- chore: add TODO.md to gitignore for worktree task files
- chore(license): consolidate to single LICENSE file
