# Pulse Orbit

[![Rust](https://img.shields.io/badge/rust-%23dea584?style=flat-square&logo=rust)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Your Mac's vital signs, one click away — and a 24-hour memory to prove it.

A macOS menu bar system monitor built with Tauri 2 and Rust. Click the tray icon to open a compact panel showing live CPU, memory, disk, network, and process metrics sampled every 2 seconds and stored locally in SQLite. Configurable alert thresholds fire macOS notifications when breached.

## Features

- **CPU monitoring** — total usage, per-core breakdown, Apple Silicon efficiency/performance core separation
- **Memory** — used/total RAM, swap, and macOS memory pressure (normal/warn/critical)
- **Disk I/O** — per-device read/write throughput via IOKit and used/total capacity
- **Network** — per-interface RX/TX rates and cumulative totals
- **Process table** — top 10 processes by CPU with per-process memory and network I/O
- **24-hour history** — metrics aggregated into 5-minute windows, retained locally in SQLite
- **Alert thresholds** — configurable per-metric with cooldown periods and native macOS notifications

## Quick Start

### Prerequisites
- Rust stable toolchain
- Node.js 20+ and pnpm
- macOS (IOKit and Core Foundation APIs)

### Installation
```bash
git clone https://github.com/saagpatel/Pulse-Orbit
cd Pulse-Orbit
pnpm install
```

### Usage
```bash
# Development
pnpm tauri dev

# Build release app
pnpm tauri build
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2 |
| Backend | Rust 2021 — sysinfo, rusqlite, r2d2, chrono, nix |
| macOS APIs | IOKit (disk I/O), Core Foundation (memory pressure) |
| Frontend | React 18 + TypeScript + Tailwind CSS + Recharts |
| Persistence | SQLite via rusqlite + r2d2 connection pool |

## License

MIT
