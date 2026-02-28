# Reddit Posts for arb Terminal Launch

## r/macapps (Primary)

**Title**: arb — A terminal that ships with Starship, Delta, z, and syntax highlighting built-in

**Body**:
```
Hey r/macapps,

I built **arb** because I was tired of spending hours configuring terminals every time I set up a new Mac.

Most terminals give you a blank slate. arb gives you a complete shell environment out of the box:

- **Starship prompt** — git status, fast, looks great
- **Delta** — beautiful diffs (perfect for reviewing AI code)
- **z** — jump to directories without typing paths
- **Syntax highlighting** — instant feedback as you type
- **Autosuggestions** — completions from your history
- **Split panes** — `Cmd+D`, no tmux needed
- **GPU-accelerated** — smooth scrolling, Metal rendering

Install:
```bash
brew tap szj2ys/arb && brew install arb
```

It's a ~40MB native macOS app. No electron. No config files. Just open and start coding.

Website with demo: https://arb-terminal.vercel.app
GitHub: https://github.com/szj2ys/arb

What do you think? Happy to answer questions!
```

---

## r/programming (Secondary)

**Title**: Show: arb — A terminal emulator that just works out of the box

**Body**:
```
Hey r/programming,

I built a terminal emulator called **arb** with a different philosophy: zero configuration, maximum productivity.

Instead of installing plugins and editing dotfiles for hours, arb ships with everything built-in:

- Starship prompt
- Delta for git diffs
- z for directory jumping
- Syntax highlighting
- Autosuggestions
- Native split panes

Built in Rust, GPU-accelerated, MIT licensed.

```bash
brew tap szj2ys/arb && brew install arb
```

GitHub: https://github.com/szj2ys/arb

The goal isn't to replace your carefully-tuned setup — it's for people who'd rather be coding than configuring. Would love your feedback!
```

---

## r/rust (Technical)

**Title**: [Show] arb — A GPU-accelerated terminal built in Rust

**Body**:
```
Hey r/rust,

I've been working on **arb**, a macOS terminal emulator built in Rust. It's based on WezTerm's core but with a focus on zero-config defaults.

**Technical details:**
- GPU-accelerated rendering via Metal
- ~40MB release binary (LTO + size optimization)
- ~100ms shell startup
- Lua scripting for customization
- Built-in shell integration (Starship, Delta, z)

**Architecture:**
- Workspace with 12+ crates
- Async I/O with smol
- Custom text layout with HarfBuzz
- Font rasterization with FreeType

```bash
brew tap szj2ys/arb && brew install arb
```

GitHub: https://github.com/szj2ys/arb

Happy to discuss implementation details!
```

---

## Posting Schedule & Tips

### Best Times to Post (US Timezones)
- **Weekday mornings**: 9-11 AM EST
- **Tuesday-Thursday**: Best engagement days
- **Avoid**: Friday afternoons, weekends (lower engagement)

### r/macapps Tips
- Flair your post with "App"
- Respond to comments quickly (first hour is critical)
- Be transparent about it being your own project
- Ask for feedback, not just upvotes

### r/programming Tips
- Follow "Show" format
- Focus on the problem it solves
- Be ready for technical questions

### General Strategy
1. Post to r/macapps first (primary audience)
2. Wait 24-48 hours before cross-posting
3. Customize the message for each subreddit
4. Engage genuinely in comments
5. Don't delete posts if they don't take off immediately

---

## Follow-up Comments (Ready to Copy)

### If someone asks "Why not just use iTerm2?"
```
Great question! iTerm2 is excellent if you enjoy configuring. arb is for people who don't:

- iTerm2: Install Starship, Delta, z, fonts separately (~1-2 hours setup)
- arb: Install and everything works immediately

arb is also smaller (~40MB vs ~55MB) and boots faster (~100ms shell startup).

If you already have a setup you love, keep it! arb is for people who'd rather be coding than configuring.
```

### If someone asks "How is this different from WezTerm?"
```
arb is built on WezTerm's core! The difference is the defaults:

- WezTerm: Bring your own config (~67MB binary)
- arb: Complete shell suite built-in (~40MB binary)

Same Lua config format, so migration is easy. arb just ships with sensible defaults so you can start working immediately.
```

### If someone mentions Ghostty
```
Ghostty is another great option! Key differences:

- Ghostty: Zig, some built-ins but still needs config
- arb: Rust, complete shell suite pre-configured, smaller binary

Both are good choices — try both and see which fits your workflow!
```
