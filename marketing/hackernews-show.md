# arb — A GPU-accelerated terminal built for AI coding

## Show HN

**arb** is a macOS terminal emulator that ships with a complete shell environment out of the box. No plugins. No configuration files. Just install and start coding.

### The Problem

Every terminal says "fast and customizable," but how many hours have you spent:
- Installing and configuring Starship
- Setting up Delta for git diffs
- Adding z for directory jumping
- Getting syntax highlighting to work
- Finding a Nerd Font that renders correctly

By the time you're done, you've forgotten what you actually wanted to build.

### What Makes arb Different

```bash
brew tap szj2ys/arb && brew install arb
```

That's it. Open it and your shell is fully equipped with:

- **Starship prompt** — git-aware, fast, zero config
- **Delta** — beautiful diffs for code review (perfect for AI-generated changes)
- **z** — smart directory jumping
- **Syntax highlighting** — instant feedback as you type
- **Autosuggestions** — grayed-out completions from history
- **JetBrains Mono Nerd Font** — proper icons without manual install
- **Native split panes** — `Cmd+D` to split, no tmux needed

### Built for AI Coding

When you're using Claude Code, Cursor, or Aider, you're reviewing a lot of AI-generated diffs. arb makes this painless:

1. Split the terminal (`Cmd+D`)
2. Run `git diff` in one pane — Delta renders it beautifully
3. Your prompt shows git status automatically
4. Navigate with `z` to jump between projects

No context switching. No plugin hunting.

### Technical Details

- **Binary size**: ~40 MB (vs iTerm2's ~55 MB, WezTerm's ~67 MB)
- **Shell startup**: ~100ms
- **GPU-accelerated**: Metal rendering, smooth scrolling
- **Lua scripting**: Fully configurable when you need it
- **MIT licensed**: No telemetry, no accounts, no lock-in

### Who This Is For

- Developers who want their terminal to "just work"
- AI coding tool users who review a lot of diffs
- People tired of maintaining dotfiles across machines
- Anyone who wants a fast, native macOS terminal

### Try It

```bash
brew tap szj2ys/arb && brew install arb
```

Or download the DMG from GitHub Releases.

### Links

- GitHub: https://github.com/szj2ys/arb
- Website: https://szj2ys.github.io/arb/ (with split pane demo)
- Docs: https://github.com/szj2ys/arb/blob/main/README.md

---

*arb is built on WezTerm's core with a focus on zero-config defaults. It's not trying to replace your carefully-tuned setup — it's for people who'd rather be coding than configuring.*
