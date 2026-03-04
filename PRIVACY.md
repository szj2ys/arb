# Privacy Policy for Arb

Last updated: March 4, 2026

## Overview

Arb is committed to protecting your privacy. This Privacy Policy explains how we collect, use, and safeguard your information when you use our terminal emulator.

## Data Collection

### What We Collect (Local Only)

Arb may collect anonymous usage statistics that are **stored locally on your device only**:

- **Installation events**: When you run `arb init`, we record that an installation occurred
- **Shell initialization**: Which shell type was initialized (e.g., zsh, bash)
- **Feature usage**: Which features you use (e.g., split panes, tabs)
- **Version information**: The version of Arb you're using
- **Anonymous device ID**: A randomly generated identifier that cannot be traced back to you

### What We Do NOT Collect

We explicitly do NOT collect:

- Your name, email, or any personal information
- Your command history or terminal input
- File paths or directory names
- Hostname or computer name
- IP address or network information
- Any content from your terminal sessions

## How Data is Used

The locally stored data is used solely for:

1. **Improving the product**: Understanding which features are most used
2. **Debugging**: Identifying common setup issues
3. **User insights**: Viewing your own usage statistics via `arb stats`

## Data Storage

All telemetry data is stored locally in:

```
~/.local/share/arb/telemetry/
```

This data:
- Never leaves your device by default
- Can be viewed anytime with `arb stats`
- Can be deleted anytime with `arb stats --clear`

## Opting Out

You can disable telemetry at any time:

### Temporary Disable
```bash
ARB_DISABLE_TELEMETRY=1 arb
```

### Permanent Disable
Add to your shell configuration:
```bash
export ARB_DISABLE_TELEMETRY=1
```

Or simply clear the data:
```bash
arb stats --clear
```

## Third-Party Services

### Website Analytics

Our website (szj2ys.github.io/arb) uses Plausible Analytics, which:
- Does not use cookies
- Does not collect personal data
- Is GDPR compliant
- Collects only aggregated, anonymous data

### GitHub

We host our code and releases on GitHub. Please refer to [GitHub's Privacy Policy](https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement) for information on how they handle data.

## Your Rights

You have the right to:
- View your data: `arb stats`
- Export your data: `arb stats --json`
- Delete your data: `arb stats --clear`
- Opt out completely: Set `ARB_DISABLE_TELEMETRY=1`

## Changes to This Policy

We may update this Privacy Policy from time to time. Changes will be posted on this page with an updated "Last updated" date.

## Contact

If you have questions about this Privacy Policy, please open an issue on our GitHub repository: https://github.com/szj2ys/arb

## Transparency

Our code is open source. You can audit exactly what data is collected by reviewing:
- `arb/src/telemetry.rs` - Telemetry implementation
- `arb/src/stats.rs` - Statistics command

We believe in transparency and giving users complete control over their data.
