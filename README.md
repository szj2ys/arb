<p align="right"><a href="README_CN.md">中文</a> | English</p>

<div align="center">
  <h1>arb</h1>
  <p><em>A fast, out-of-the-box terminal built for AI coding.</em></p>
</div>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/rust-2021%2B-000000?style=flat-square&logo=rust" alt="Rust" />
</p>

<p align="center">
  <img src="assets/arb.jpeg" alt="arb Screenshot" width="1000" />
  <br/>
  arb is designed for an out-of-the-box experience.
</p>

## Features

- **Zero Config**: Polished defaults with JetBrains Mono, opencode theme, optimized macOS font rendering, smooth animations.
- **Built-in Shell Suite**: Comes pre-loaded with Starship, z, Delta, syntax highlighting, autosuggestions, and autocompletions.
- **Fast & Lightweight**: 40% smaller binary, instant startup, lazy loading, stripped-down GPU-accelerated core.
- **Lua Scripting**: Retains full Lua scripting support for infinite customization.

## Quick Start

1. Download the latest DMG release and drag to Applications.
2. Or install with Homebrew: `brew install szj2ys/arb/arb`
3. Open arb. The app is notarized by Apple, so it opens without security warnings
4. On first launch, arb will automatically set up your shell environment

## Usage Guide

arb comes with intuitive macOS-native shortcuts:

| Action | Shortcut |
| :--- | :--- |
| New Tab | `Cmd + T` |
| New Window | `Cmd + N` |
| Split Pane Vertical | `Cmd + D` |
| Split Pane Horizontal | `Cmd + Shift + D` |
| Zoom/Unzoom Pane | `Cmd + Shift + Enter` |
| Resize Pane | `Cmd + Ctrl + Arrows` |
| Close Tab/Pane | `Cmd + W` |
| Navigate Tabs | `Cmd + [`, `Cmd + ]` or `Cmd + 1-9` |
| Navigate Panes | `Cmd + Opt + Arrows` |
| Clear Screen | `Cmd + R` |
| Font Size | `Cmd + +`, `Cmd + -`, `Cmd + 0` |
| Smart Jump | `z <dir>` |
| Smart Select | `z -l <dir>` |
| Recent Dirs | `z -t` |

## Configuration

arb comes with a carefully curated suite of CLI tools, pre-configured for immediate productivity:

- **Starship**: A fast, customizable prompt showing git status, package versions, and execution time.
- **z**: A smarter cd command that learns your most used directories for instant navigation.
- **Delta**: A syntax-highlighting pager for git, diff, and grep output.
- **zsh-completions**: Extended command and subcommand completion definitions.
- **Syntax Highlighting**: Real-time command validation and coloring.
- **Autosuggestions**: Intelligent, history-based completions similar to Fish shell.

### Customization

arb is fully configurable via standard Lua scripts.

On macOS, bundled defaults in `arb.app/Contents/Resources/arb.lua` are fallback only, so user config is loaded first.

Use a single user config path: `~/.config/arb/arb.lua`.

### Updates & Reset

- Check/apply update from CLI: `arb update`
- Remove arb-managed shell defaults and integration: `arb reset` (or non-interactive `arb reset --yes`)
- GUI auto-update check uses numeric version comparison (for example `0.1.10` is correctly newer than `0.1.9`).

## Why arb?

I heavily rely on the CLI for both work and personal projects.

I used Alacritty for years, but its lack of multi-tab support became cumbersome for AI-assisted coding. Kitty has some aesthetic and positioning quirks I couldn't get past. Ghostty shows promise but font rendering needs work. Warp feels bloated and requires a login. iTerm2 is reliable but showing its age and harder to deeply customize.

I wanted an environment that was ready immediately, without extensive configuration—and something significantly faster and lighter.

So I built arb to be that environment: fast, polished, and ready to work.

### Performance

| Metric | Upstream | arb | Methodology |
| :--- | :--- | :--- | :--- |
| **Executable Size** | ~67 MB | ~40 MB | Aggressive symbol stripping & feature pruning |
| **Resources Volume** | ~100 MB | ~80 MB | Asset optimization & lazy-loaded assets |
| **Launch Latency** | Standard | Instant | Just-in-time initialization |
| **Shell Bootstrap** | ~200ms | ~100ms | Optimized environment provisioning |

Achieved through aggressive stripping of unused features, lazy loading of color schemes, and shell optimizations.

## Support

- If arb helped you, star the repo or share it with friends.
- Got ideas or found bugs? Open an issue/PR or check [CONTRIBUTING.md](CONTRIBUTING.md) for details.


## License

arb is licensed under the MIT License. See `LICENSE`.
