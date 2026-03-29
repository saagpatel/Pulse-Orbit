# Pulse Orbit

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![Tauri](https://img.shields.io/badge/tauri-2.x-24C8DB)
![License](https://img.shields.io/badge/license-MIT-green)

A macOS menu bar system monitor built with Tauri 2 and Rust. Click the tray icon to open a compact panel showing live CPU, memory, disk, network, and process metrics — all sampled every 2 seconds and stored locally in SQLite.

## Screenshot

![Pulse Orbit screenshot placeholder](docs/screenshot.png)

## Features

- **CPU** — Total usage, per-core breakdown, clock frequency. On Apple Silicon, efficiency and performance cores are identified separately with averaged utilization for each cluster.
- **Memory** — Used/total RAM, swap usage, and macOS memory pressure level (normal / warn / critical).
- **Disk** — Per-device read/write throughput (via IOKit) and used/total capacity.
- **Network** — Per-interface RX/TX rates and cumulative totals.
- **Processes** — Top 10 processes by CPU usage with per-process memory and network I/O.
- **Alert thresholds** — Configurable per-metric thresholds with cooldown periods; fires macOS notifications when breached.
- **History** — Metrics aggregated into 5-minute windows and retained for 24 hours in a local SQLite database.
- **Settings** — Adjustable polling interval (1–10 s), auto-launch at login toggle, and data storage stats.

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 |
| Backend | Rust 2021 — `sysinfo`, `rusqlite`, `r2d2`, `chrono`, `nix` |
| macOS APIs | IOKit (disk I/O), Core Foundation (memory pressure), Apple Silicon topology |
| Frontend | React 18, TypeScript, Vite, Tailwind CSS, Recharts |
| Persistence | SQLite via bundled `rusqlite` + `r2d2` connection pool |

## Prerequisites

- macOS 12 or later
- [Rust](https://rustup.rs/) (stable toolchain)
- Node.js 18+ and npm
- Xcode Command Line Tools (`xcode-select --install`)

## Getting Started

```bash
# Clone the repository
git clone <repo-url>
cd Pulse-Orbit

# Install frontend dependencies
npm install

# Run in development mode (opens the panel + hot reload)
npm run tauri dev

# Build a release .app bundle
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/macos/`.

## Project Structure

```
Pulse-Orbit/
├── src/                        # React frontend
│   ├── components/             # Shared UI components (sparkline, metric bar, etc.)
│   ├── hooks/                  # useMetrics — subscribes to Tauri metric-snapshot events
│   ├── views/                  # One view per tab: cpu, memory, disk, network, process, settings
│   ├── lib/                    # Formatting utilities
│   └── types/                  # Shared TypeScript types
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── metrics/            # Collector, per-subsystem readers, threshold checker
│   │   ├── commands/           # Tauri commands: history, processes, thresholds, settings
│   │   ├── db/                 # SQLite pool init, schema, aggregate writer
│   │   ├── notifications.rs    # macOS notification dispatch
│   │   └── lib.rs              # App setup, tray icon, polling startup
│   └── tauri.conf.json         # Window config (400×560, no decorations, always-on-top)
├── index.html
└── package.json
```

## License

MIT — see [LICENSE](LICENSE).
