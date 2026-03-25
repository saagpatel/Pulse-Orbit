# Pulse Orbit — Implementation Roadmap

## Architecture

### System Overview
```
[macOS System APIs]
        |
        ▼
[Rust: sysinfo crate — polls every 2s]
        |
        ├──→ [Tauri Event: metric-snapshot] ──→ [React: useMetrics hook] ──→ [Live Panel UIs]
        |
        └──→ [Rust: SQLite writer — 5-min aggregate] ──→ [SQLite DB] ──→ [Tauri Command: get-history] ──→ [Trend Charts]

[React: Process Kill] ──→ [Tauri Command: kill-process] ──→ [Rust: nix::sys::signal::kill]

[React: Set Threshold] ──→ [Tauri Command: set-threshold] ──→ [SQLite: alert_thresholds]
                                                               ↑
[Rust: Threshold Checker] ──────────────────────────────────┘
        |
        └──→ [macOS Notification: NSUserNotificationCenter]
```

### File Structure
```
pulse-orbit/
├── src/                          # React frontend
│   ├── views/
│   │   ├── CpuView.tsx           # Per-core bars + E/P breakdown (graceful degrade)
│   │   ├── MemoryView.tsx        # RAM + swap + memory pressure gauge
│   │   ├── DiskView.tsx          # Read/write throughput + storage usage
│   │   ├── NetworkView.tsx       # Bandwidth per interface + per-process breakdown
│   │   └── ProcessView.tsx       # Top-10 processes by CPU/RAM + kill flow
│   ├── components/
│   │   ├── PanelTabs.tsx         # Tab nav between the 5 views
│   │   ├── MetricBar.tsx         # Reusable horizontal bar with label + value
│   │   ├── SparklineChart.tsx    # Recharts 60-second rolling window chart
│   │   ├── TrendChart.tsx        # Recharts 24h area chart from SQLite data
│   │   ├── KillModal.tsx         # Two-step process kill confirmation dialog
│   │   └── ThresholdEditor.tsx   # Input component for alert threshold config
│   ├── hooks/
│   │   ├── useMetrics.ts         # Listens to Tauri 'metric-snapshot' events
│   │   ├── useHistory.ts         # Calls Tauri 'get-history' command for trend data
│   │   └── useProcesses.ts       # Subscribes to process list from metric snapshots
│   ├── types/
│   │   └── index.ts              # All shared TypeScript interfaces (see below)
│   ├── App.tsx                   # Root component — PanelTabs + view routing
│   └── main.tsx                  # Tauri app entry point
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Tauri app builder, menu bar window config
│   │   ├── metrics/
│   │   │   ├── mod.rs            # Module exports
│   │   │   ├── collector.rs      # sysinfo System struct, 2s polling loop
│   │   │   ├── types.rs          # Rust structs for all metric payloads
│   │   │   └── m_series.rs       # IOKit probe for E/P core detection (graceful degrade)
│   │   ├── commands/
│   │   │   ├── mod.rs            # Register all commands
│   │   │   ├── history.rs        # get_history(metric, duration_hours) → Vec<AggregateRow>
│   │   │   ├── processes.rs      # kill_process(pid) → Result<(), String>
│   │   │   └── thresholds.rs     # set_threshold, get_thresholds → SQLite CRUD
│   │   ├── db/
│   │   │   ├── mod.rs            # DB init, connection pool (r2d2 + rusqlite)
│   │   │   ├── migrations.rs     # Run embedded SQL migrations on first launch
│   │   │   └── writer.rs         # 5-min aggregate flush loop
│   │   └── notifications.rs      # macOS notification dispatch when thresholds breached
│   ├── migrations/
│   │   ├── 001_initial_schema.sql
│   │   └── 002_alert_thresholds.sql
│   ├── Cargo.toml
│   └── tauri.conf.json
├── CLAUDE.md
├── IMPLEMENTATION-ROADMAP.md
├── package.json
└── tsconfig.json
```

---

## Data Model

### SQLite Schema

```sql
-- Migration 001: Core metric history
CREATE TABLE metric_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_type TEXT    NOT NULL,  -- 'cpu', 'memory', 'disk_read', 'disk_write', 'net_in', 'net_out'
    interface   TEXT,              -- NULL for cpu/memory; interface name for net; device name for disk
    value_avg   REAL    NOT NULL,  -- Average over the 5-minute window
    value_max   REAL    NOT NULL,  -- Peak in the 5-minute window
    window_start DATETIME NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_snapshots_type_time ON metric_snapshots(metric_type, window_start DESC);
CREATE INDEX idx_snapshots_window    ON metric_snapshots(window_start DESC);

-- Migration 002: Alert thresholds
CREATE TABLE alert_thresholds (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_type TEXT    NOT NULL UNIQUE, -- 'cpu', 'memory', 'disk_read', 'net_in', etc.
    threshold   REAL    NOT NULL,        -- e.g. 90.0 for 90% CPU
    enabled     INTEGER NOT NULL DEFAULT 1,  -- 0 = disabled
    cooldown_seconds INTEGER NOT NULL DEFAULT 300, -- Min seconds between repeat alerts
    last_fired_at DATETIME,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Retention Policy
- Insert one row per metric_type per 5-minute window
- Purge rows where `window_start < datetime('now', '-24 hours')` — run on app startup and every hour
- At 5-min granularity: ~288 rows/metric/day × ~8 metrics = ~2,300 rows/day → negligible storage

---

## TypeScript Interfaces

```typescript
// src/types/index.ts

// Emitted by Rust every 2 seconds via Tauri event 'metric-snapshot'
export interface MetricSnapshot {
  timestamp: number;            // Unix ms
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disk: DiskMetrics;
  network: NetworkMetrics;
  processes: ProcessInfo[];
}

export interface CpuMetrics {
  total_percent: number;                  // 0–100
  per_core: CoreMetric[];                 // one per logical core
  frequency_mhz: number;
  m_series_breakdown: MSeriesBreakdown | null;  // null if not detectable
}

export interface CoreMetric {
  core_index: number;
  percent: number;              // 0–100
  core_type: 'efficiency' | 'performance' | 'unknown';
}

export interface MSeriesBreakdown {
  efficiency_cores_avg: number;
  performance_cores_avg: number;
  efficiency_core_count: number;
  performance_core_count: number;
}

export interface MemoryMetrics {
  used_bytes: number;
  total_bytes: number;
  swap_used_bytes: number;
  swap_total_bytes: number;
  pressure: 'normal' | 'warn' | 'critical'; // from macOS memory pressure API
}

export interface DiskMetrics {
  devices: DiskDevice[];
}

export interface DiskDevice {
  name: string;          // e.g. 'disk0'
  read_bytes_per_sec: number;
  write_bytes_per_sec: number;
  total_bytes: number;
  used_bytes: number;
}

export interface NetworkMetrics {
  interfaces: NetworkInterface[];
}

export interface NetworkInterface {
  name: string;          // e.g. 'en0', 'utun0'
  rx_bytes_per_sec: number;
  tx_bytes_per_sec: number;
  total_rx_bytes: number;
  total_tx_bytes: number;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_percent: number;   // 0–100
  memory_bytes: number;
  status: string;        // 'running' | 'sleeping' | etc.
}

// SQLite aggregate row returned by get-history command
export interface HistoryRow {
  window_start: string;  // ISO datetime
  value_avg: number;
  value_max: number;
}

export interface AlertThreshold {
  metric_type: string;
  threshold: number;
  enabled: boolean;
  cooldown_seconds: number;
}
```

---

## Tauri IPC Contracts

### Events (Rust → React, fire-and-forget every 2s)
| Event Name | Payload Type | Description |
|------------|-------------|-------------|
| `metric-snapshot` | `MetricSnapshot` | Full system state, emitted every 2s by background thread |

### Commands (React → Rust, request/response)
| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `get_history` | `{ metric_type: string, interface?: string, hours: number }` | `HistoryRow[]` | Returns 5-min aggregates for trend charts |
| `kill_process` | `{ pid: number }` | `Result<(), String>` | Sends SIGTERM; returns error string if failed |
| `get_thresholds` | none | `AlertThreshold[]` | All configured thresholds |
| `set_threshold` | `{ metric_type: string, threshold: number, enabled: boolean, cooldown_seconds: number }` | `Result<(), String>` | Upsert a threshold row |

---

## Rust Dependencies

```toml
# src-tauri/Cargo.toml additions
[dependencies]
tauri = { version = "2", features = ["macos-private-api"] }
sysinfo = "0.30"
rusqlite = { version = "0.31", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"
nix = { version = "0.27", features = ["signal", "process"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Frontend Dependencies

```bash
npm create tauri-app@latest pulse-orbit -- --template react-ts
cd pulse-orbit
npm install recharts@2.12
npm install -D tailwindcss@3 postcss autoprefixer
npx tailwindcss init -p
```

---

## Scope Boundaries

**In scope (v1):**
- Menu bar app with dropdown panel (5-tab layout)
- Real-time CPU, memory, disk, network, process panels
- 60-second rolling sparklines on each panel
- 24h trend charts from SQLite aggregates
- Top-10 process list with two-step kill flow
- M-series E/P core breakdown with graceful degradation
- Alert thresholds with macOS native notifications
- 24h data retention with hourly purge

**Out of scope (v1):**
- Per-process network stats (requires `libproc` — deferred to v2)
- GPU metrics (Apple GPU not exposed by sysinfo — deferred)
- Multiple machine monitoring
- Export / data download
- Settings persistence beyond alert thresholds
- Auto-launch at login (add post-v1)
- Any cloud sync

**Deferred to v2:**
- Per-process network breakdown via `libproc`
- GPU utilization via `IOKit` Metal stats
- Configurable polling interval
- Auto-launch at login plist generation

---

## Security & Credentials

- **No credentials.** Pulse Orbit reads only local system APIs — no external services, no auth tokens.
- **Process kill:** Uses POSIX `SIGTERM` via the `nix` crate. Only processes owned by the current user can be killed (macOS enforces this at the OS level). No privilege escalation.
- **Data stays local:** All metric data stored in `~/.local/share/pulse-orbit/metrics.db` (Tauri's app data dir). Nothing leaves the machine.
- **SQLite permissions:** DB file created with 0600 permissions (user read/write only).
- **No sudo required:** `powermetrics` is explicitly out of scope. All metrics sourced from unprivileged APIs only.

---

## Phase 0: Foundation & Rust Metric Engine (Week 1)

**Objective:** Tauri app scaffolded, Rust polling loop running, MetricSnapshot emitted every 2 seconds to a placeholder React console log. No UI, no SQLite yet — just proving the data pipeline works end-to-end.

**Tasks:**
1. Scaffold Tauri 2 app with `create-tauri-app` using `react-ts` template — **Acceptance:** `npm run tauri dev` launches a window without errors.
2. Implement `sysinfo::System` collector in `metrics/collector.rs` — polls CPU, memory, disk, network every 2s in a Tokio background task — **Acceptance:** `println!` logs show non-zero CPU and memory values every 2s in the Rust terminal.
3. Serialize `MetricSnapshot` struct to JSON and emit as Tauri event `metric-snapshot` — **Acceptance:** React `console.log` receives the event with correct shape (all fields present, no null panics).
4. Implement `metrics/m_series.rs` IOKit probe for E/P core type detection — **Acceptance:** On M4 Pro, `core_type` is `'efficiency'` or `'performance'` for each core. If probe fails, all cores return `'unknown'` and no panic occurs.
5. Configure `tauri.conf.json` for menu bar window: `"activation_policy": "accessory"`, `"visible": false` on startup, window width 400px, height 560px, no decorations — **Acceptance:** App has no dock icon; menu bar icon appears; clicking it shows/hides the window.

**Verification checklist:**
- [ ] `npm run tauri dev` → no compile errors
- [ ] Rust terminal output shows metric values updating every 2s
- [ ] React DevTools / console shows `metric-snapshot` events with all `MetricSnapshot` fields
- [ ] App icon appears in menu bar, no dock icon
- [ ] M-series breakdown: logs show E/P core types OR all `'unknown'` — no panic either way

**Risks:**
- `tauri.conf.json` menu bar config for Tauri 2 has different key names than Tauri 1 docs → Check Tauri 2 migration guide, specifically `app.macOSPrivateApi` and `app.withGlobalTauri`. Fallback: open a regular window first, convert to menu bar in a separate task.
- `sysinfo` per-core CPU values may all report the same aggregate on first call (needs a warm-up tick) → Initialize `System` and call `refresh_cpu()` twice before emitting; discard first snapshot.

---

## Phase 1: Live Panels — CPU, Memory, Process (Week 2)

**Objective:** Functional dropdown panel with three live-updating tabs: CPU, Memory, Process. User can see real system data and kill a process. Recharts sparklines showing the last 60 seconds.

**Tasks:**
1. Install and configure Tailwind CSS dark theme in the Tauri React project — **Acceptance:** A `bg-gray-900 text-gray-100` div renders correctly in `npm run tauri dev`.
2. Build `useMetrics` hook that subscribes to `metric-snapshot` events and maintains a rolling 60-entry circular buffer (120s at 2s polling = 60 samples) — **Acceptance:** Hook re-renders on every event; buffer never exceeds 60 entries.
3. Build `CpuView.tsx`: total usage bar, per-core grid with `MetricBar`, E/P breakdown section (conditional on `m_series_breakdown !== null`), 60s sparkline — **Acceptance:** All core bars animate smoothly; if E/P data is null, the breakdown section is hidden (no empty box).
4. Build `MemoryView.tsx`: used/total bar in GB, swap bar, memory pressure badge (`normal`=green, `warn`=amber, `critical`=red), 60s sparkline — **Acceptance:** Numbers match Activity Monitor within ~5%.
5. Build `ProcessView.tsx`: sorted table of top 10 by CPU%, columns: name, PID, CPU%, RAM — **Acceptance:** List updates every 2s; sort order reflects live CPU usage.
6. Implement `kill_process` Tauri command in `commands/processes.rs` using `nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGTERM)` — **Acceptance:** `tauri::invoke('kill_process', { pid })` terminates a test sleep process; returns an error string if PID is owned by another user.
7. Build `KillModal.tsx` two-step confirmation: click "Kill" → modal shows process name + PID → "Confirm Kill" button → invoke command → success toast or error toast — **Acceptance:** Confirming kills the process; cancelling does not.
8. Build `PanelTabs.tsx` with CPU / Memory / Process tabs — **Acceptance:** Tab switching is instant with no unmount/remount of live charts.

**Verification checklist:**
- [ ] Click menu bar icon → dropdown opens at correct position (below menu bar, right-aligned)
- [ ] CPU bars animate every 2s, values within 5% of Activity Monitor
- [ ] Memory used/total correct; swap shows 0 if not in use
- [ ] Memory pressure badge shows correct color state
- [ ] Process list shows top-10 updating live
- [ ] Kill flow: click Kill → modal appears → Confirm → process gone from list within 2s
- [ ] Kill flow: click Kill → Cancel → process still in list

**Risks:**
- Recharts re-renders may cause jank with 60 data points updating every 2s → Use `useMemo` on chart data; avoid creating new arrays on every render. Benchmark: < 5ms render time acceptable.
- Menu bar window positioning: Tauri 2 on macOS may not snap to menu bar icon automatically → Use `window.setPosition()` relative to the status item frame. Fallback: center of screen for v1, fix positioning in a patch.

---

## Phase 2: Disk, Network + SQLite History (Week 3)

**Objective:** All 5 panels complete. SQLite storing 5-minute aggregates. Trend charts showing 24h history. Retention purge running.

**Tasks:**
1. Build `DiskView.tsx`: per-device read/write throughput (MB/s), storage bar (used/total GB) — **Acceptance:** Values match `iostat -d 1 3` output within 10%.
2. Build `NetworkView.tsx`: per-interface rx/tx bandwidth (KB/s or MB/s auto-scaled), total transferred counters — **Acceptance:** Values match `nettop -m tcp` output within 10%. Note: per-process network breakdown is out of scope for v1 — show only interface-level totals.
3. Initialize SQLite in `db/mod.rs` using `r2d2_sqlite` connection pool (pool size: 3) stored in Tauri's app data dir at `~/.local/share/pulse-orbit/metrics.db` — **Acceptance:** DB file created on first launch; `migrations.rs` runs both migration SQL files without errors.
4. Implement 5-minute aggregate writer in `db/writer.rs`: buffer incoming MetricSnapshot values for each metric_type, flush to `metric_snapshots` table on the 5-minute boundary — **Acceptance:** After 10 minutes of running, `SELECT COUNT(*) FROM metric_snapshots` returns ~16–20 rows.
5. Implement hourly retention purge: delete rows where `window_start < datetime('now', '-24 hours')` — **Acceptance:** Run purge manually against a seeded DB with old rows; old rows deleted, new rows preserved.
6. Implement `get_history` Tauri command: query `metric_snapshots` for a given `metric_type` and `hours` window, return `Vec<HistoryRow>` — **Acceptance:** `invoke('get_history', { metric_type: 'cpu', hours: 6 })` returns correctly shaped array.
7. Build `TrendChart.tsx` using Recharts `AreaChart`: X-axis = time (formatted as HH:mm), Y-axis = value 0–100 (or bytes), renders `HistoryRow[]` — **Acceptance:** Chart renders with 24h of seeded data without layout overflow.
8. Add trend chart to each of the 5 panels — gated behind a "Show History" toggle to keep the default view compact — **Acceptance:** Toggle shows/hides chart without panel height jump.

**Verification checklist:**
- [ ] All 5 tabs present and functional
- [ ] Disk panel values within 10% of `iostat` output
- [ ] Network panel values within 10% of `nettop` output
- [ ] `metrics.db` created in correct app data dir
- [ ] After 10 minutes: `SELECT COUNT(*) FROM metric_snapshots` shows ~16–20 rows
- [ ] `get_history` command returns data for all metric types
- [ ] History chart renders correctly when toggled on in each panel
- [ ] After seeding old rows, purge function removes them correctly

**Risks:**
- `sysinfo` network interface byte counts are cumulative totals, not per-second rates → Calculate delta between snapshots: `(current_total - prev_total) / elapsed_seconds`. Initialize prev values on first snapshot and skip emission until second snapshot arrives.
- SQLite write contention between the main collector thread and the history command → Use `r2d2` connection pool; writer uses one connection, read commands use others. Pool size 3 is sufficient.

---

## Phase 3: Alerts + Polish (Week 4)

**Objective:** Alert threshold system working with macOS native notifications. UI polish pass. App ready for daily use.

**Tasks:**
1. Build `ThresholdEditor.tsx`: a settings-style UI with a slider/input per metric type (CPU %, Memory %, Disk Write MB/s, Net In MB/s), enable toggle, cooldown setting — **Acceptance:** Saving calls `set_threshold` command; values persist across app restarts.
2. Implement threshold checker in Rust collector loop: after each MetricSnapshot, compare `total_cpu_percent`, `memory_used/total`, disk and net rates against stored thresholds — **Acceptance:** When CPU is artificially high (run `stress-ng`), notification fires. Does not fire again until cooldown expires.
3. Implement macOS native notification dispatch in `notifications.rs` using Tauri's `notification` plugin — **Acceptance:** Notification appears in macOS Notification Center with title "Pulse Orbit" and body "[metric] exceeded [threshold]%".
4. Add a Settings tab (6th tab, gear icon) housing `ThresholdEditor` and a data retention info row showing current DB size and row count — **Acceptance:** Tab renders; threshold saves persist.
5. Polish pass: consistent spacing on all panels, empty states for disk/network when no activity, smooth tab transition (no height flash), correct number formatting (bytes → KB/MB/GB auto-scale) — **Acceptance:** Visual review against the design spec below.
6. Performance audit: measure React render time under continuous 2s events — **Acceptance:** React DevTools Profiler shows < 8ms render time per snapshot event.

**Verification checklist:**
- [ ] Threshold editor saves and persists across restart
- [ ] CPU threshold alert fires when CPU > threshold (test with `stress-ng --cpu 8 --timeout 30s`)
- [ ] Cooldown prevents duplicate notifications within the configured window
- [ ] Notification appears in macOS Notification Center
- [ ] All panels show correct number formatting (no raw bytes displayed)
- [ ] React render time < 8ms under continuous events (DevTools Profiler)
- [ ] DB size displayed correctly in Settings tab

**Risks:**
- Tauri 2 `notification` plugin setup requires `NSUserNotificationsUsageDescription` in `Info.plist` → Add this to `tauri.conf.json` under `bundle.macOS.infoPlist`. Fallback: macOS will just silently drop notifications without it — easy to diagnose.

---

## Visual Design Reference

**Layout:** 400px wide dropdown panel. Tabs at top (5 icons + label). Active panel below. No decorations, no title bar. Rounded bottom corners (8px). Drop shadow.

**Color palette (dark):**
- Background: `#0f1117` (slightly deeper than gray-900)
- Card/panel: `#1a1d27`
- Border: `#2d3148`
- Primary accent: `#6366f1` (indigo-500) — used for active tab, primary bars, sparkline fill
- Text primary: `#e2e8f0`
- Text secondary: `#64748b`
- Success/normal: `#22c55e`
- Warning: `#f59e0b`
- Critical: `#ef4444`

**Metric bar anatomy:** Label (left, 40% width) → bar (50% width, `bg-gray-700` track, accent fill) → value label (right, 10% width, monospace).

**Sparkline:** 60px tall, no axes, no legend, indigo fill with 20% opacity, indigo stroke 1.5px.

**Trend chart:** 120px tall when expanded, X-axis time labels (HH:mm), Y-axis 0–100, grid lines at 25/50/75.
