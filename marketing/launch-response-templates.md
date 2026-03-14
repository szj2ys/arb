# Launch Response Templates

Pre-written responses for Show HN launch and community engagement.

---

## Technical Questions

### Q: How is this different from Ghostty/Warp/iTerm2?

**Short (for quick replies):**
> Arb is zero-config with a built-in shell suite. Ghostty requires config for advanced features; Warp is cloud-dependent; iTerm2 needs manual plugin setup. Arb works offline with batteries included.

**Long (for detailed discussions):**
> Great question! Here's the breakdown:
>
> **vs Ghostty**: Both are GPU-accelerated and fast. Ghostty requires configuration for advanced features. Arb ships pre-configured with a complete shell environment (Starship, Delta, z, etc.) so you can be productive immediately.
>
> **vs Warp**: Warp has excellent AI features but requires a cloud account and connection. Arb works entirely offline, no login required, no telemetry. Your terminal, your data.
>
> **vs iTerm2**: iTerm2 is powerful but you'll spend time installing plugins, configuring colors, setting up your prompt. Arb includes all of that out of the box in a ~40MB binary.
>
> Arb sits in the middle: native performance like Ghostty, offline like iTerm2, but with the "it just works" experience.

---

### Q: Why another terminal emulator?

> WezTerm is excellent but complex to configure for the full modern shell experience. Arb pre-configures the best practices and adds a cohesive shell suite (Starship prompt, Delta diffs, z directory jumper, syntax highlighting, autosuggestions). It's for developers who'd rather be coding than configuring dotfiles.

---

### Q: Is it really GPU-accelerated?

> Yes, uses the same core rendering as WezTerm (OpenGL/Vulkan/Metal depending on platform). The difference is in the pre-configured experience, not the rendering engine.

---

### Q: Can I use my existing WezTerm config?

> Mostly yes! Arb is built on WezTerm's core and uses the same Lua configuration format. Most WezTerm configs work with minimal changes. We have a migration guide in the README.

---

### Q: Does it work with tmux?

> You can use tmux if you want, but Arb has native split panes (`Cmd+D` horizontal, `Cmd+Shift+D` vertical) and tab management. Many users find they don't need tmux anymore.

---

## Bug Reports

### Initial Response Template

> Thanks for reporting! To help us reproduce and fix this, could you share:
>
> - macOS version (`sw_vers`)
> - Arb version (`arb --version`)
> - Shell type (`echo $SHELL`)
> - Installation method (Homebrew or DMG)
> - Steps to reproduce
> - Expected vs actual behavior
>
> We'll look into it immediately. For urgent issues, you can also reach out via GitHub Discussions.

---

### Follow-up After Fix

> Fixed in [version/link]. Thanks again for the detailed report — it made all the difference. Let us know if you run into anything else!

---

## Feature Requests

### Standard Response

> Great idea! This would be a good fit. Would you mind opening a GitHub Discussion so we can:
> 1. Gather community interest
> 2. Discuss implementation approach
> 3. Track progress
>
> We prioritize based on demand and alignment with our zero-config philosophy.

---

### Out of Scope (Polite Decline)

> Thanks for the suggestion! After consideration, this doesn't align with our current focus on zero-config terminal experience. We're intentionally keeping the scope tight to ensure everything works seamlessly together.
>
> That said, Arb is MIT-licensed and extensible via Lua — you might be able to achieve this with a custom config. Happy to point you in the right direction if you're interested.

---

## Common Issues

### "arb init" failed

> Sorry you're hitting this! Let's diagnose:
>
> 1. Run `arb doctor` — it checks common issues
> 2. If that doesn't help, check:
>    - Shell permissions: `ls -la ~/.zshrc`
>    - Arb location: `ls -la /Applications/Arb.app`
> 3. Try: `arb reset && arb init`
>
> If still stuck, open an issue with the output of `arb doctor`.

---
### "Command not found" after install

> This usually means the shell integration hasn't been loaded. Try:
>
> 1. `arb init` (installs shell integration)
> 2. Open a **new** terminal tab/window
> 3. `which arb` should now show the wrapper
>
> The wrapper lives at `~/.config/arb/zsh/bin/arb`. If `arb init` succeeded but `arb` still isn't found, your PATH might not include `~/.config/arb/zsh/bin`.

---

## Positive Feedback Responses

### Simple Thanks

> Thanks! Glad you're enjoying it. If you have a moment, starring the repo helps others discover it ⭐

---

### Detailed Positive Feedback

> This is great to hear! We built Arb because we were frustrated with the same things. If you have any other feedback or ideas, we'd love to hear them. Also, consider sharing with others who might benefit — word of mouth is everything for a project like this.

---

## Critical Feedback

### Constructive Criticism

> Thanks for the honest feedback — this is exactly what we need to hear. Would you be open to elaborating in a GitHub Discussion? We want to understand the pain points better and see if we can address them.

---

### Comparisons to Competitors (When They're Right)

> Fair point. [Specific feature] is definitely something we're looking at. No timeline yet, but it's on our radar. Thanks for the push!

---

## Marketing/Partnership Inquiries

### Content Creator/Blog Request

> Thanks for reaching out! We'd love to collaborate. Please email [contact] with:
> - Your audience size and platform
> - What you'd like to cover
> - Timeline
>
> We'll get back to you within 48 hours.

---

### Enterprise/Team Inquiry

> Thanks for the interest! Arb is currently focused on individual developers. For team/enterprise features, please open a GitHub Discussion so we can understand your requirements better.

---

## Launch Metrics Targets

| Metric | Target | Current |
|--------|--------|---------|
| HN Upvotes | 100+ | TBD |
| HN Comments | 50+ | TBD |
| GitHub Stars | +500 first week | baseline |
| Website Visits | 10K+ first week | TBD |

## Response Time Goals

- **First 2 hours**: Respond to all comments within 15 minutes
- **Next 22 hours**: Respond within 1 hour
- **Ongoing**: Respond within 24 hours

## Response Principles

1. **Be human, not corporate** — Use first person, admit when we don't know
2. **Acknowledge first, investigate second** — Even a "thanks, looking into this" is better than silence
3. **Link to resources** — GitHub repo, docs, discussions
4. **Invite deeper engagement** — "Open an issue" / "Join the discussion"
5. **Don't over-promise** — It's okay to say "no timeline yet"
