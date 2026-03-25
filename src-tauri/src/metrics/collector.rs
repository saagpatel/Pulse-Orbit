use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::Timelike;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

use super::disk_io;
use super::gpu;
use super::m_series::{self, CoreTopology};
use super::proc_net;
use super::threshold_checker::ThresholdChecker;
use super::types::*;
use crate::db::writer::AggregateBuffer;
use crate::db::DbPool;

const DEFAULT_POLL_MS: u64 = 2000;

pub fn start_polling(app: AppHandle, pool: DbPool) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        let mut disks = Disks::new_with_refreshed_list();

        // Detect Apple Silicon topology once at startup
        let topology = m_series::detect_core_topology();
        if let Some(ref topo) = topology {
            eprintln!(
                "[pulse-orbit] Detected Apple Silicon: {}E + {}P cores",
                topo.efficiency_count, topo.performance_count
            );
        } else {
            eprintln!("[pulse-orbit] No Apple Silicon detected — all cores will be 'unknown'");
        }

        // Warm-up: first CPU reading is always 0
        sys.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();

        // Initialize network baselines
        let mut prev_net: HashMap<String, (u64, u64)> = HashMap::new();
        for (name, data) in &networks {
            prev_net.insert(
                name.to_string(),
                (data.total_received(), data.total_transmitted()),
            );
        }

        // Initialize disk I/O baselines from IOKit
        let mut prev_disk: HashMap<String, (u64, u64)> = disk_io::read_disk_io_stats();

        let mut prev_time = Instant::now();

        // Per-process network baselines
        let mut prev_proc_net: HashMap<u32, (u64, u64)> = HashMap::new();

        // Aggregate buffer for SQLite history
        let mut buffer = AggregateBuffer::new();

        // Threshold checker for alerts
        let mut checker = ThresholdChecker::new(&pool);

        // Dynamic polling interval (reloaded from DB periodically)
        let mut poll_interval_ms = load_poll_interval(&pool);
        let mut last_interval_check = Instant::now();

        // Skip the very first emission (no delta baseline)
        let mut warmup_done = false;

        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            sys.refresh_processes(ProcessesToUpdate::All, true);
            networks.refresh(true);
            disks.refresh(true);

            if !warmup_done {
                warmup_done = true;
                prev_time = Instant::now();
                std::thread::sleep(Duration::from_millis(poll_interval_ms));
                continue;
            }

            let now = Instant::now();
            let elapsed_secs = now.duration_since(prev_time).as_secs_f64();

            // Read current disk I/O from IOKit
            let current_disk_io = disk_io::read_disk_io_stats();

            // Read GPU utilization
            let gpu_reading = gpu::read_gpu_utilization();

            let snapshot = build_snapshot(
                &sys,
                &networks,
                &disks,
                &topology,
                &mut prev_net,
                &mut prev_disk,
                &current_disk_io,
                &mut prev_proc_net,
                gpu_reading.as_ref(),
                elapsed_secs,
            );

            if let Err(e) = app.emit("metric-snapshot", &snapshot) {
                eprintln!("[pulse-orbit] Failed to emit metric-snapshot: {e}");
            }

            // Check thresholds and fire notifications
            checker.check(&snapshot, &app, &pool);

            // Record to aggregate buffer and flush on 5-minute boundary
            buffer.record(&snapshot);
            let now_min = chrono::Local::now().minute();
            if now_min.is_multiple_of(5) && now_min != buffer.last_flush_minute {
                buffer.flush(&pool);
                buffer.last_flush_minute = now_min;
            }

            // Reload polling interval every 60s
            if last_interval_check.elapsed().as_secs() >= 60 {
                poll_interval_ms = load_poll_interval(&pool);
                last_interval_check = Instant::now();
            }

            // Update previous disk I/O for next delta
            prev_disk = current_disk_io;
            prev_time = now;
            std::thread::sleep(Duration::from_millis(poll_interval_ms));
        }
    });
}

fn load_poll_interval(pool: &DbPool) -> u64 {
    if let Ok(conn) = pool.get() {
        if let Ok(val) = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'polling_interval_ms'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(ms) = val.parse::<u64>() {
                if (1000..=10000).contains(&ms) {
                    return ms;
                }
            }
        }
    }
    DEFAULT_POLL_MS
}

#[allow(clippy::too_many_arguments)]
fn build_snapshot(
    sys: &System,
    networks: &Networks,
    disks: &Disks,
    topology: &Option<CoreTopology>,
    prev_net: &mut HashMap<String, (u64, u64)>,
    prev_disk: &mut HashMap<String, (u64, u64)>,
    current_disk_io: &HashMap<String, (u64, u64)>,
    prev_proc_net: &mut HashMap<u32, (u64, u64)>,
    gpu_reading: Option<&gpu::GpuReading>,
    elapsed_secs: f64,
) -> MetricSnapshot {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let cpu = build_cpu_metrics(sys, topology);
    let memory = build_memory_metrics(sys);
    let disk = build_disk_metrics(disks, prev_disk, current_disk_io, elapsed_secs);
    let network = build_network_metrics(networks, prev_net, elapsed_secs);
    let processes = build_process_list(sys, prev_proc_net, elapsed_secs);
    let gpu = gpu_reading.map(|r| GpuMetrics {
        utilization_percent: r.utilization_percent,
    });

    MetricSnapshot {
        timestamp,
        cpu,
        memory,
        disk,
        network,
        processes,
        gpu,
    }
}

fn build_cpu_metrics(sys: &System, topology: &Option<CoreTopology>) -> CpuMetrics {
    let cpus = sys.cpus();
    let total_percent = sys.global_cpu_usage();

    let per_core: Vec<CoreMetric> = cpus
        .iter()
        .enumerate()
        .map(|(i, cpu)| {
            let core_type = match topology {
                Some(topo) => {
                    if i < topo.efficiency_count {
                        CoreType::Efficiency
                    } else {
                        CoreType::Performance
                    }
                }
                None => CoreType::Unknown,
            };
            CoreMetric {
                core_index: i,
                percent: cpu.cpu_usage(),
                core_type,
            }
        })
        .collect();

    let frequency_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

    let m_series_breakdown = topology.as_ref().map(|topo| {
        let (e_sum, e_count) = per_core
            .iter()
            .filter(|c| matches!(c.core_type, CoreType::Efficiency))
            .fold((0.0f32, 0usize), |(sum, count), c| {
                (sum + c.percent, count + 1)
            });
        let (p_sum, p_count) = per_core
            .iter()
            .filter(|c| matches!(c.core_type, CoreType::Performance))
            .fold((0.0f32, 0usize), |(sum, count), c| {
                (sum + c.percent, count + 1)
            });

        MSeriesBreakdown {
            efficiency_cores_avg: if e_count > 0 {
                e_sum / e_count as f32
            } else {
                0.0
            },
            performance_cores_avg: if p_count > 0 {
                p_sum / p_count as f32
            } else {
                0.0
            },
            efficiency_core_count: topo.efficiency_count,
            performance_core_count: topo.performance_count,
        }
    });

    CpuMetrics {
        total_percent,
        per_core,
        frequency_mhz,
        m_series_breakdown,
    }
}

fn build_memory_metrics(sys: &System) -> MemoryMetrics {
    let pressure = match m_series::memory_pressure_level() {
        Some(1) => MemoryPressure::Normal,
        Some(2) => MemoryPressure::Warn,
        Some(4) => MemoryPressure::Critical,
        _ => MemoryPressure::Normal,
    };

    MemoryMetrics {
        used_bytes: sys.used_memory(),
        total_bytes: sys.total_memory(),
        swap_used_bytes: sys.used_swap(),
        swap_total_bytes: sys.total_swap(),
        pressure,
    }
}

fn build_disk_metrics(
    disks: &Disks,
    prev_disk: &HashMap<String, (u64, u64)>,
    current_disk_io: &HashMap<String, (u64, u64)>,
    elapsed_secs: f64,
) -> DiskMetrics {
    // Build a map of throughput rates from IOKit delta
    let mut throughput: HashMap<&str, (u64, u64)> = HashMap::new();
    for (name, &(cur_read, cur_write)) in current_disk_io {
        if let Some(&(prev_read, prev_write)) = prev_disk.get(name.as_str()) {
            let read_delta = cur_read.saturating_sub(prev_read);
            let write_delta = cur_write.saturating_sub(prev_write);
            throughput.insert(
                name.as_str(),
                (
                    (read_delta as f64 / elapsed_secs) as u64,
                    (write_delta as f64 / elapsed_secs) as u64,
                ),
            );
        }
    }

    let devices: Vec<DiskDevice> = disks
        .iter()
        .map(|d| {
            let name = d.name().to_string_lossy().into_owned();
            // Try to find IOKit throughput matching this disk's name
            let (read_rate, write_rate) = throughput
                .iter()
                .find(|(io_name, _)| name.contains(*io_name) || io_name.contains(name.as_str()))
                .map(|(_, rates)| *rates)
                .unwrap_or((0, 0));

            DiskDevice {
                name,
                read_bytes_per_sec: read_rate,
                write_bytes_per_sec: write_rate,
                total_bytes: d.total_space(),
                used_bytes: d.total_space() - d.available_space(),
            }
        })
        .collect();

    DiskMetrics { devices }
}

fn build_network_metrics(
    networks: &Networks,
    prev_net: &mut HashMap<String, (u64, u64)>,
    elapsed_secs: f64,
) -> NetworkMetrics {
    let interfaces: Vec<NetworkInterface> = networks
        .iter()
        .map(|(name, data)| {
            let (prev_rx, prev_tx) = prev_net
                .get(name.as_str())
                .copied()
                .unwrap_or((data.total_received(), data.total_transmitted()));

            let rx_delta = data.total_received().saturating_sub(prev_rx);
            let tx_delta = data.total_transmitted().saturating_sub(prev_tx);

            prev_net.insert(
                name.to_string(),
                (data.total_received(), data.total_transmitted()),
            );

            NetworkInterface {
                name: name.to_string(),
                rx_bytes_per_sec: (rx_delta as f64 / elapsed_secs) as u64,
                tx_bytes_per_sec: (tx_delta as f64 / elapsed_secs) as u64,
                total_rx_bytes: data.total_received(),
                total_tx_bytes: data.total_transmitted(),
            }
        })
        .collect();

    NetworkMetrics { interfaces }
}

fn build_process_list(
    sys: &System,
    prev_proc_net: &mut HashMap<u32, (u64, u64)>,
    elapsed_secs: f64,
) -> Vec<ProcessInfo> {
    let mut procs: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, proc_)| {
            let pid_u32 = pid.as_u32();

            // Per-process network delta
            let (rx_per_sec, tx_per_sec) =
                if let Some(usage) = proc_net::read_process_net_usage(pid_u32) {
                    let (prev_rx, prev_tx) = prev_proc_net
                        .get(&pid_u32)
                        .copied()
                        .unwrap_or((usage.rx_bytes, usage.tx_bytes));
                    let rx_delta = usage.rx_bytes.saturating_sub(prev_rx);
                    let tx_delta = usage.tx_bytes.saturating_sub(prev_tx);
                    prev_proc_net.insert(pid_u32, (usage.rx_bytes, usage.tx_bytes));
                    (
                        (rx_delta as f64 / elapsed_secs) as u64,
                        (tx_delta as f64 / elapsed_secs) as u64,
                    )
                } else {
                    (0, 0)
                };

            ProcessInfo {
                pid: pid_u32,
                name: proc_.name().to_string_lossy().into_owned(),
                cpu_percent: proc_.cpu_usage(),
                memory_bytes: proc_.memory(),
                status: format!("{:?}", proc_.status()),
                network_rx_bytes_per_sec: rx_per_sec,
                network_tx_bytes_per_sec: tx_per_sec,
            }
        })
        .collect();

    procs.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    procs.truncate(10);

    // Clean up stale PIDs from prev_proc_net
    let active_pids: std::collections::HashSet<u32> = procs.iter().map(|p| p.pid).collect();
    prev_proc_net.retain(|pid, _| active_pids.contains(pid));

    procs
}
