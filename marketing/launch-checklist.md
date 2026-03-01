# Launch Checklist for Arb

> **Status**: Ready for Launch 🚀
> **Last Updated**: 2026-03-01

---

## Pre-Launch (1-2 Days Before)

### Product Readiness
- [x] Build passes (`cargo build --release`)
- [x] Tests pass (`cargo nextest run`)
- [x] Homebrew formula works (`brew tap szj2ys/arb && brew install arb`)
- [x] DMG notarized and signed
- [x] `arb doctor` command works
- [x] `arb update` command works

### Website Readiness
- [x] Website deployed to Vercel
- [x] All SEO landing pages live
- [x] Plausible Analytics working
- [x] Social preview image optimized
- [x] GitHub link working
- [x] Install command correct

### Content Readiness
- [x] Show HN post written (`marketing/hackernews-show.md`)
- [x] Reddit posts written (`marketing/reddit-posts.md`)
- [x] Response templates prepared
- [x] README up to date

---

## Launch Day

### Timing (Critical!)

**Hacker News**:
- **Best time**: Tuesday or Wednesday, 8-10 AM PST
- **Why**: Tech workers starting their day, highest engagement
- **Backup**: Thursday same time
- **Avoid**: Fridays, weekends, holidays

**Reddit**:
- **r/macapps**: Tuesday-Thursday, 9-11 AM EST
- **r/programming**: Same, but wait 24-48h after HN
- **r/rust**: Any weekday morning

### Launch Sequence

1. **T-0:00** - Post to Hacker News
   - Use title: "Show HN: arb – A GPU-accelerated terminal that just works out of the box"
   - Include GitHub link
   - Monitor for first 2 hours (critical for ranking)

2. **T+2:00** - Monitor HN Comments
   - Respond quickly to questions
   - Use prepared response templates
   - Be authentic, not defensive

3. **T+24-48h** - Post to Reddit
   - Start with r/macapps (primary audience)
   - Customize message per subreddit
   - Cross-post after 48h if HN went well

4. **Ongoing** - Social Sharing
   - Twitter/X post
   - LinkedIn (if relevant to your network)
   - Dev.to article (optional)

---

## Post-Launch (First Week)

### Hour 1-2 (Critical!)
- [ ] Respond to every HN comment
- [ ] Monitor Plausible Analytics for traffic spike
- [ ] Watch for technical issues (crashes, install failures)
- [ ] Be ready to hotfix if critical bug found

### Day 1
- [ ] Continue responding to comments
- [ ] Thank users who star the repo
- [ ] Post to Reddit (if not done)
- [ ] Monitor GitHub issues

### Week 1
- [ ] Daily GitHub issue triage
- [ ] Respond to all Reddit comments
- [ ] Track download metrics
- [ ] Collect feedback for next iteration

---

## Metrics to Track

| Metric | Tool | Target |
|--------|------|--------|
| GitHub Stars | GitHub | +500 in first week |
| Website Visitors | Plausible | 10K+ in first week |
| Downloads | GitHub Releases | 1000+ in first month |
| HN Ranking | Hacker News | Front page (top 30) |
| Reddit Upvotes | Reddit | 100+ on r/macapps |

---

## Response Templates

### "Why not iTerm2?"
```
Great question! iTerm2 is excellent if you enjoy configuring. arb is for people who don't:

- iTerm2: Install Starship, Delta, z, fonts separately (~1-2 hours setup)
- arb: Install and everything works immediately

arb is also smaller (~40MB vs ~55MB) and boots faster (~100ms shell startup).

If you already have a setup you love, keep it! arb is for people who'd rather be coding than configuring.
```

### "How is this different from WezTerm?"
```
arb is built on WezTerm's core! The difference is the defaults:

- WezTerm: Bring your own config (~67MB binary)
- arb: Complete shell suite built-in (~40MB binary)

Same Lua config format, so migration is easy. arb just ships with sensible defaults so you can start working immediately.
```

### "What about Ghostty/Warp/Alacritty?"
```
All great options! Key differences:

- **Ghostty**: Zig, some built-ins but still needs config
- **Warp**: Closed source, requires account/login
- **Alacritty**: Minimal, requires extensive config
- **arb**: Rust, complete shell suite pre-configured, no login, MIT licensed

Try them all and see which fits your workflow best!
```

### Install Issues
```
Sorry you're having trouble! Let's fix this:

1. Run `arb doctor` - it checks common issues
2. If that doesn't help, please open an issue with:
   - macOS version
   - Installation method (brew or DMG)
   - Error message (if any)

We'll get you sorted quickly!
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Launch flops | Have backup subreddits; don't delete posts |
| Technical issues | `arb doctor` ready; quick patch release process |
| Negative feedback | Respond authentically; acknowledge limitations |
| Server overload | Vercel handles traffic; monitor GitHub rate limits |
| Competitor comparison | Be respectful; focus on different use cases |

---

## Post-Launch Actions

### If Launch Goes Well 🎉
- [ ] Write launch retrospective
- [ ] Plan next feature based on feedback
- [ ] Consider Product Hunt launch (2+ weeks later)
- [ ] Build email list for updates

### If Launch Underperforms 😔
- [ ] Analyze why (timing? messaging? product?)
- [ ] Iterate on positioning
- [ ] Try different communities
- [ ] Focus on organic growth (SEO, content)

---

## Emergency Contacts

- **GitHub Issues**: https://github.com/szj2ys/arb/issues
- **Critical bugs**: Create hotfix branch immediately
- **Security issues**: See SECURITY.md

---

## Final Checklist

- [ ] Website live and tested
- [ ] Install command verified
- [ ] Content reviewed and ready
- [ ] Calendar blocked for launch day
- [ ] Notifications enabled for GitHub/Reddit/HN
- [ ] Coffee ready ☕

**Good luck! 🚀**
