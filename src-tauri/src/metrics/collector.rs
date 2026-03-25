use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Disks, Networks, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

use super::m_series::{self, CoreTopology};
use super::types::*;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn start_polling(app: AppHandle) {
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

        // Initialize disk baselines
        let mut prev_disk: HashMap<String, (u64, u64)> = HashMap::new();
        // Note: disk I/O tracking needs process-level stats or iostat.
        // sysinfo Disks only provides capacity info, not throughput.
        // We'll report 0 for read/write throughput in Phase 0 and add
        // proper I/O tracking in Phase 2.

        let mut prev_time = Instant::now();

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
                std::thread::sleep(POLL_INTERVAL);
                continue;
            }

            let now = Instant::now();
            let elapsed_secs = now.duration_since(prev_time).as_secs_f64();

            let snapshot = build_snapshot(
                &sys,
                &networks,
                &disks,
                &topology,
                &mut prev_net,
                &mut prev_disk,
                elapsed_secs,
            );

            if let Err(e) = app.emit("metric-snapshot", &snapshot) {
                eprintln!("[pulse-orbit] Failed to emit metric-snapshot: {e}");
            }

            prev_time = now;
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}

fn build_snapshot(
    sys: &System,
    networks: &Networks,
    disks: &Disks,
    topology: &Option<CoreTopology>,
    prev_net: &mut HashMap<String, (u64, u64)>,
    _prev_disk: &mut HashMap<String, (u64, u64)>,
    elapsed_secs: f64,
) -> MetricSnapshot {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let cpu = build_cpu_metrics(sys, topology);
    let memory = build_memory_metrics(sys);
    let disk = build_disk_metrics(disks);
    let network = build_network_metrics(networks, prev_net, elapsed_secs);
    let processes = build_process_list(sys);

    MetricSnapshot {
        timestamp,
        cpu,
        memory,
        disk,
        network,
        processes,
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
        _ => MemoryPressure::Normal, // default to normal if unavailable
    };

    MemoryMetrics {
        used_bytes: sys.used_memory(),
        total_bytes: sys.total_memory(),
        swap_used_bytes: sys.used_swap(),
        swap_total_bytes: sys.total_swap(),
        pressure,
    }
}

fn build_disk_metrics(disks: &Disks) -> DiskMetrics {
    let devices: Vec<DiskDevice> = disks
        .iter()
        .map(|d| DiskDevice {
            name: d.name().to_string_lossy().into_owned(),
            read_bytes_per_sec: 0,  // Phase 0: no I/O throughput tracking yet
            write_bytes_per_sec: 0, // Phase 0: no I/O throughput tracking yet
            total_bytes: d.total_space(),
            used_bytes: d.total_space() - d.available_space(),
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

fn build_process_list(sys: &System) -> Vec<ProcessInfo> {
    let mut procs: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, proc_)| ProcessInfo {
            pid: pid.as_u32(),
            name: proc_.name().to_string_lossy().into_owned(),
            cpu_percent: proc_.cpu_usage(),
            memory_bytes: proc_.memory(),
            status: format!("{:?}", proc_.status()),
        })
        .collect();

    // Sort by CPU% descending, take top 10
    procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(10);
    procs
}
