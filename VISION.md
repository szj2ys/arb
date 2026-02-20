# Vision & Mission

## Tagline

> Fast by default. Deep when you need it.

## Mission

Make the terminal disappear — so developers can focus entirely on thinking and creating.

## Vision

The best tool is the one you never think about. arb aims to be the default terminal for the AI coding era — zero-config startup, GPU-level performance, Lua-level depth — so developers spend zero time configuring and 100% of their energy on building.

## Design Principles

These principles guide every decision in arb. When in doubt, refer back here.

### 1. Zero-Config First

The out-of-the-box experience should be excellent. A developer should open arb for the first time and immediately feel productive — no config files, no plugin hunting, no theme shopping. Sensible defaults are a feature, not a compromise.

### 2. Fast, Then Beautiful, Then Powerful

Performance is non-negotiable. Visual polish comes next. Deep customization is available but never required. This is the priority order when trade-offs arise.

### 3. Ordinary on the Surface, Extraordinary Underneath

arb should feel like a calm, familiar terminal — until you need more. Lua scripting, split panes, smart directory jumping, and git-aware tooling are there when you reach for them, invisible when you don't.

### 4. Opinionated but Not Locked

We ship strong defaults (JetBrains Mono, Arb Dark, Starship prompt, Delta pager). Every default is overridable. We never force a workflow — we offer a great starting point.

### 5. Less is More

Inherited from WezTerm, arb aggressively strips what developers don't need. Every feature must earn its place. A smaller, faster, more focused terminal is better than a feature-complete but bloated one.

### 6. No Gatekeeping

No login. No account. No telemetry. No paywall. MIT licensed. The terminal is a developer's most personal tool — it should respect that.

## Who We Build For

Developers who live in the CLI — especially those using AI coding tools (Claude Code, Copilot, Cursor, etc.) — and want a terminal that works perfectly from day one without sacrificing the depth to grow with them.

## What We Don't Optimize For

- **Enterprise features** — no team management, no SSO, no dashboards.
- **Cross-editor integration** — arb is a standalone terminal, not an IDE plugin.
- **Backward compatibility with legacy systems** — we target modern macOS first and move fast.

## How to Use This Document

- **Before adding a feature**: Does it align with the mission? Does it serve our target user? Does it pass the "earn its place" bar?
- **When making trade-offs**: Refer to the priority order — performance > polish > power.
- **When reviewing PRs**: Ask if the change makes arb simpler and faster, not just more capable.
