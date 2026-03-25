# Pulse Orbit

## Overview
A native macOS menu bar system monitor built with Tauri 2 + Rust + React. Clicking the menu bar icon reveals a dropdown panel showing real-time CPU, memory, disk, network, and process stats. Rust polls system metrics every 2 seconds via the `sysinfo` crate and pushes data to React via Tauri events. SQLite stores 24-hour metric history for trend charts.

## Tech Stack
- **Rust**: 1.75+ (Tauri backend, sysinfo polling, SQLite writes)
- **Tauri**: 2.x (menu bar window, Tauri events, IPC commands)
- **React**: 18+ (hooks-based frontend, no class components)
- **TypeScript**: 5.x (strict mode)
- **Recharts**: 2.x (real-time and historical charts)
- **SQLite**: via `rusqlite` 0.31 (metric history, alert thresholds)
- **Tailwind CSS**: 3.x (utility-only, dark theme)

## Development Conventions
- TypeScript strict mode — no `any` types
- kebab-case for files, PascalCase for React components
- Conventional commits: `feat:`, `fix:`, `chore:`, `perf:`
- Rust: `clippy` clean before committing
- All Tauri commands defined in `src-tauri/src/commands/` — one file per domain
- All React views in `src/views/` — one file per panel

## Current Phase
**Phase 0: Foundation & Rust Metric Engine**
See IMPLEMENTATION-ROADMAP.md for full phase details.

## Key Decisions
| Decision | Choice | Why |
|----------|--------|-----|
| Window type | Menu bar app (NSStatusItem) | Ambient awareness without dock presence |
| Metric polling | Rust `sysinfo` crate + Tauri events | Low-level access, 2s polling doesn't need commands |
| M-series core breakdown | Graceful degradation | `powermetrics` requires sudo; degrade to aggregate if breakdown unavailable |
| Historical storage | SQLite with 5-min aggregates, 24h retention | 2s snapshots = 43,200 rows/day per metric; aggregation keeps DB small |
| Process kill | Two-step confirm (click → confirm modal → kill) | Prevents accidental kills |
| Chart library | Recharts | Already in user's stack, handles live data well |

## Do NOT
- Do not use `powermetrics` — it requires `sudo` and is not acceptable for a menu bar UX
- Do not store raw 2-second snapshots in SQLite — use 5-minute aggregates for history
- Do not use Tauri `invoke` (commands) for the polling loop — use `emit` events from a Rust background thread
- Do not add features not in the current phase of IMPLEMENTATION-ROADMAP.md
- Do not use `any` types in TypeScript — define all Tauri event payloads as interfaces
- Do not skip the kill confirmation modal — always two-step for process termination
