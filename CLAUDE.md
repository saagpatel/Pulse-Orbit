# Pulse Orbit

macOS menu bar system monitor — Tauri 2 + Rust + React, shipped at v1.0.0.

## Stack

- **Rust**: 1.75+ (Tauri backend, sysinfo polling, SQLite writes)
- **Tauri**: 2.x (menu bar window, Tauri events, IPC commands)
- **React**: 18+ (hooks-based frontend, no class components)
- **TypeScript**: 5.x (strict mode)
- **Recharts**: 2.x (real-time and historical charts)
- **SQLite**: via `rusqlite` 0.32 (metric history, alert thresholds)
- **Tailwind CSS**: 3.x (utility-only, dark theme)

## Build / Run

```bash
# Development
pnpm tauri dev

# Build release app
pnpm tauri build
```

## Architecture

- Rust polls system metrics every 2 seconds via `sysinfo` and pushes data to React via Tauri `emit` events from a background thread (not `invoke` commands).
- All Tauri commands live in `src-tauri/src/commands/` — one file per domain.
- All React views in `src/views/` — one file per panel.
- SQLite stores 24-hour history using 5-minute aggregates (raw 2s snapshots = 43,200 rows/day per metric; aggregation keeps DB small).

## Key Decisions

| Decision | Choice | Why |
|----------|--------|-----|
| Window type | Menu bar app (NSStatusItem) | Ambient awareness without dock presence |
| Metric polling | Rust `sysinfo` crate + Tauri events | Low-level access, 2s polling doesn't need commands |
| M-series core breakdown | Graceful degradation | `powermetrics` requires sudo; degrade to aggregate if breakdown unavailable |
| Historical storage | SQLite with 5-min aggregates, 24h retention | 2s snapshots = 43,200 rows/day per metric; aggregation keeps DB small |
| Process kill | Two-step confirm (click → confirm modal → kill) | Prevents accidental kills |
| Chart library | Recharts | Already in user's stack, handles live data well |

## Gotchas

- **`powermetrics` is off-limits**: requires `sudo`, unacceptable for a menu bar UX. Use `sysinfo` + graceful degradation for M-series core breakdown.
- **SQLite history**: store 5-minute aggregates only; raw 2-second snapshots produce 43,200 rows/day per metric and bloat the DB.
- **Polling loop**: drive via Tauri `emit` events from a Rust background thread, not `invoke` commands.
- **TypeScript**: strict mode enforced — define all Tauri event payloads as interfaces; `any` types block clippy-equivalent discipline.
- **Process kill**: always two-step (click → confirm modal → kill); skipping the modal risks accidental termination.

## Conventions

- TypeScript strict mode; kebab-case files, PascalCase React components.
- Conventional commits: `feat:`, `fix:`, `chore:`, `perf:`.
- Rust: `clippy` clean before committing.
- Feature scope: follow IMPLEMENTATION-ROADMAP.md; all v1.0.0 phases complete including GPU (IOKit), per-process network I/O (`proc_pid_rusage`), auto-launch, and configurable polling.

<!-- portfolio-context:start -->
# Portfolio Context

## What This Project Is

A native macOS menu bar system monitor built with Tauri 2 + Rust + React. Clicking the menu bar icon reveals a dropdown panel showing real-time CPU, memory, disk, network, and process stats. Rust polls system metrics every 2 seconds via the `sysinfo` crate and pushes data to React via Tauri events. SQLite stores 24-hour metric history for trend charts.

## Current State

**All phases shipped (v1.0.0)**
All four roadmap phases are complete, plus several features originally listed as v2 scope: GPU monitoring via IOKit, per-process network I/O via `proc_pid_rusage`, auto-launch at login, and configurable polling interval. See IMPLEMENTATION-ROADMAP.md for phase details.

## Stack

- **Rust**: 1.75+ (Tauri backend, sysinfo polling, SQLite writes)
- **Tauri**: 2.x (menu bar window, Tauri events, IPC commands)
- **React**: 18+ (hooks-based frontend, no class components)
- **TypeScript**: 5.x (strict mode)
- **Recharts**: 2.x (real-time and historical charts)
- **SQLite**: via `rusqlite` 0.32 (metric history, alert thresholds)
- **Tailwind CSS**: 3.x (utility-only, dark theme)

## How To Run

```bash
# Development
pnpm tauri dev

# Build release app
pnpm tauri build
```

## Known Risks

- Do not use `powermetrics` — it requires `sudo` and is not acceptable for a menu bar UX
- Do not store raw 2-second snapshots in SQLite — use 5-minute aggregates for history
- Do not use Tauri `invoke` (commands) for the polling loop — use `emit` events from a Rust background thread
- Do not add features not in the current phase of IMPLEMENTATION-ROADMAP.md
- Do not use `any` types in TypeScript — define all Tauri event payloads as interfaces
- Do not skip the kill confirmation modal — always two-step for process termination

## Next Recommended Move

Use this context plus the README and supporting docs to resume the next active task, then promote the repo beyond minimum-viable by capturing a dedicated handoff, roadmap, or discovery artifact.

<!-- portfolio-context:end -->

<!-- secondbrain-breadcrumb -->
## SecondBrain knowledge vault

Prior lessons, decisions, and context for this project live in SecondBrain at `wiki/maps/projects/pulse-orbit.md`. The whole vault is searchable via the `engraph` MCP — query it for this project + its stack before non-trivial work.
