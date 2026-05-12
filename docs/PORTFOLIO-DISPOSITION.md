# Pulse Orbit — Portfolio Disposition

**Status:** Release Frozen — Tauri 2 + Rust macOS menu-bar system
monitor at **v1.0.0** on `origin/main` (memory drift: was "v2.0"
per project memory — actual canonical state is v1.0.0). .dmg
distribution build deps + Cargo.lock + baseline tests for
threshold checker and metric types + CSP hardened. **27th signing
cluster member.** Path quirk: local dir
`/Users/d/Projects/Pulse Orbit` has a space; origin repo
`saagpatel/Pulse-Orbit` uses hyphen.

> Disposition uses strict `origin/main` verification.
> **Memory drift recorded** — "v2.0" → actual v1.0.0.
> **Path quirk** — local space, origin hyphen.

---

## Verification posture

Only `origin` (`saagpatel/Pulse-Orbit`, hyphen). Clean migration.

`origin/main`:

- Tip: `b151149` chore: update Cargo.lock for version 1.0.0
- v1.0 release closeout cadence (matches APIReverse / NetworkDecoder
  signature):
  - `b151149` Cargo.lock for v1.0.0
  - `ecffedf` test: add baseline tests for threshold checker and
    metric types
  - `3539bb6` chore: bump version to 1.0.0 and add Content Security
    Policy
- Full OSS scaffolding wave
- Default branch: `main`

---

## Current state in one paragraph

Pulse Orbit is a macOS menu-bar system monitor built with Tauri 2
+ Rust. Tray icon opens a compact panel showing live CPU
(per-core, Apple Silicon E/P core split), memory (used/swap/
pressure normal/warn/critical), disk I/O (per-device via IOKit),
network (per-interface RX/TX), and top-10 process table. Metrics
sampled every 2 seconds, stored in local SQLite. **24-hour history**
aggregated into 5-minute windows. Configurable per-metric alert
thresholds with cooldown periods → native macOS notifications.
macOS-only (IOKit + Core Foundation dependencies). Per canonical
state: v1.0.0. **Memory drift correction**: prior memory described
this as "v2.0" — actual state is v1.0.0.

---

## Why "Release Frozen" — 27th signing cluster member

Standard Tauri 2 v1.0 signature: v1.0.0 bump + Cargo.lock + .dmg
deps + baseline tests + CSP. Distinguishing:

- **macOS-only architecture** (IOKit, Core Foundation, Apple
  Silicon E/P core APIs) — no cross-platform aspiration.
- **Menu-bar tray app shape** — similar to GlassLayer (overlay) but
  different distribution surface (tray icon vs borderless overlay).
- **24-hour metric retention** with 5-minute aggregation — SQLite
  rolling window is the load-bearing data model.

---

## Cluster taxonomy update

| Cluster | Count | Sub-shapes hint |
|---|---|---|
| **Signing (Apple desktop)** | **27** | (menu-bar tray sub-pattern: Pulse Orbit) |

A "menu-bar tray" sub-pattern is now recognizable inside the
signing cluster (Pulse Orbit; future macOS monitor tools would
batch here). Not formalized as a sub-shape — distribution is
identical to other signing cluster members.

---

## Unblock trigger (operator)

1. **Apple Developer ID + notarization credentials.**
2. **macOS-specific entitlements** — IOKit + system data access may
   need explicit entitlements depending on Apple's current sandbox
   policy. Verify before notarization.
3. **Performance posture** — sampling every 2s with per-process
   data has measurable CPU cost; verify the monitor itself isn't a
   top-10 CPU consumer in its own table.
4. **Path quirk** for tooling — local dir name has space,
   origin uses hyphen. Tooling that derives the repo name from
   directory may produce `Pulse Orbit` (with space) vs the actual
   `Pulse-Orbit`.
5. **Apple Silicon E/P core API stability** — Apple's
   PerformanceMonitor APIs are not part of public SDK historically;
   verify the implementation isn't using private APIs that would
   cause App Store rejection (App Store distribution wasn't
   declared, but if added later, this matters).
6. **Verify signed/notarized DMG** opens cleanly.

Estimated operator time: ~3 hours.

---

## Portfolio operating system instructions

| Aspect | Posture |
|---|---|
| Portfolio status | `Release Frozen` |
| Distribution channel | **DMG via Apple Developer ID** (macOS-only) |
| Version | **v1.0.0** (memory drift: was "v2.0") |
| Review cadence | Suspend overdue counting |
| Resurface conditions | (a) Apple signing credentials, (b) macOS-specific entitlement review, (c) Apple Silicon API change, (d) v1.1 scope |
| Co-batch with | Signing cluster — **now 27 repos** |
| Sub-pattern | **Menu-bar tray** (informal; Pulse Orbit is current sole inhabitant) |
| Special concern | **Apple Silicon E/P core API stability.** Verify implementation doesn't use private APIs. |
| Special concern | **Memory drift correction** — "v2.0" → "v1.0.0" needs memory record update. |
| Special concern | **Path quirk** — local dir name has space; origin repo uses hyphen. |
| Special concern | **Self-monitoring performance posture.** Monitor shouldn't be a top-10 CPU consumer in its own process table. |

---

## Reactivation procedure

1. Verify branch tracking.
2. Review stash `r15-pulseorbit-stash` (CLAUDE.md + package-lock.json
   mods + .claude/ + .codex/ + AGENTS.md + pnpm-lock.yaml). Operator
   npm→pnpm transition pattern continues.
3. **Update memory record**: v1.0.0 (not v2.0).
4. Test self-monitoring CPU cost during `cargo run`.
5. Run baseline tests for threshold checker + metric types.

---

## Last known reference

| Field | Value |
|---|---|
| `origin/main` tip | `b151149` chore: update Cargo.lock for version 1.0.0 |
| Default branch | `main` |
| Build system | Tauri 2 + Rust + macOS (IOKit, Core Foundation) |
| Version | **v1.0.0** (memory drift correction) |
| Platform | macOS only (IOKit + Apple Silicon E/P core APIs) |
| Distinguishing tech | Menu-bar tray app + 24-hour rolling metric history (5-min windows in SQLite) + Apple Silicon E/P core separation |
| Path quirk | Local dir `/Users/d/Projects/Pulse Orbit` (space) vs origin `saagpatel/Pulse-Orbit` (hyphen) |
| Migration state | No `legacy-origin` remote |
| Distinguishing feature | **27th signing cluster member. Menu-bar tray sub-pattern.** Memory drift correction (v2.0 → v1.0.0). |
