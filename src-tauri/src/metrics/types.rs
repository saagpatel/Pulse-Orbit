use serde::Serialize;

/// Full system snapshot emitted every 2 seconds via Tauri event.
#[derive(Clone, Debug, Serialize)]
pub struct MetricSnapshot {
    pub timestamp: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub processes: Vec<ProcessInfo>,
    pub gpu: Option<GpuMetrics>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GpuMetrics {
    pub utilization_percent: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CpuMetrics {
    pub total_percent: f32,
    pub per_core: Vec<CoreMetric>,
    pub frequency_mhz: u64,
    pub m_series_breakdown: Option<MSeriesBreakdown>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CoreMetric {
    pub core_index: usize,
    pub percent: f32,
    pub core_type: CoreType,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoreType {
    Efficiency,
    Performance,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct MSeriesBreakdown {
    pub efficiency_cores_avg: f32,
    pub performance_cores_avg: f32,
    pub efficiency_core_count: usize,
    pub performance_core_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemoryMetrics {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub pressure: MemoryPressure,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryPressure {
    Normal,
    Warn,
    Critical,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiskMetrics {
    pub devices: Vec<DiskDevice>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiskDevice {
    pub name: String,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub network_rx_bytes_per_sec: u64,
    pub network_tx_bytes_per_sec: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- DiskDevice field access ---

    #[test]
    fn disk_device_read_write_bytes_are_independent() {
        let dev = DiskDevice {
            name: "disk0".into(),
            read_bytes_per_sec: 1_000_000,
            write_bytes_per_sec: 500_000,
            total_bytes: 500_000_000_000,
            used_bytes: 100_000_000_000,
        };
        assert_eq!(dev.read_bytes_per_sec, 1_000_000);
        assert_eq!(dev.write_bytes_per_sec, 500_000);
        assert_eq!(dev.name, "disk0");
    }

    #[test]
    fn disk_used_never_exceeds_total() {
        let dev = DiskDevice {
            name: "disk1".into(),
            read_bytes_per_sec: 0,
            write_bytes_per_sec: 0,
            total_bytes: 256_000_000_000,
            used_bytes: 128_000_000_000,
        };
        assert!(dev.used_bytes <= dev.total_bytes);
    }

    // --- NetworkInterface field access ---

    #[test]
    fn network_interface_rx_tx_are_independent() {
        let iface = NetworkInterface {
            name: "en0".into(),
            rx_bytes_per_sec: 2_097_152,
            tx_bytes_per_sec: 524_288,
            total_rx_bytes: 1_000_000_000,
            total_tx_bytes: 500_000_000,
        };
        assert_eq!(iface.rx_bytes_per_sec, 2_097_152);
        assert_eq!(iface.tx_bytes_per_sec, 524_288);
        assert_eq!(iface.name, "en0");
    }

    // --- ProcessInfo field access ---

    #[test]
    fn process_info_stores_pid_and_name() {
        let proc = ProcessInfo {
            pid: 1234,
            name: "safari".into(),
            cpu_percent: 3.5,
            memory_bytes: 104_857_600,
            status: "running".into(),
            network_rx_bytes_per_sec: 0,
            network_tx_bytes_per_sec: 0,
        };
        assert_eq!(proc.pid, 1234);
        assert_eq!(proc.name, "safari");
        assert_eq!(proc.status, "running");
    }

    // --- MemoryMetrics usage ratio ---

    #[test]
    fn memory_used_fraction_is_correct() {
        let mem = MemoryMetrics {
            used_bytes: 8_589_934_592,   // 8 GB
            total_bytes: 17_179_869_184, // 16 GB
            swap_used_bytes: 0,
            swap_total_bytes: 0,
            pressure: MemoryPressure::Normal,
        };
        let fraction = mem.used_bytes as f64 / mem.total_bytes as f64;
        assert!(
            (fraction - 0.5).abs() < 1e-9,
            "8/16 GB = 50%, got {fraction}"
        );
    }

    #[test]
    fn memory_pressure_variants_are_distinct() {
        // Verify pattern matching exhausts all variants without compile error
        let pressures = [
            MemoryPressure::Normal,
            MemoryPressure::Warn,
            MemoryPressure::Critical,
        ];
        let labels: Vec<&str> = pressures
            .iter()
            .map(|p| match p {
                MemoryPressure::Normal => "normal",
                MemoryPressure::Warn => "warn",
                MemoryPressure::Critical => "critical",
            })
            .collect();
        assert_eq!(labels, vec!["normal", "warn", "critical"]);
    }

    // --- CoreType variants ---

    #[test]
    fn core_type_variants_map_correctly() {
        let core_types = [
            CoreType::Efficiency,
            CoreType::Performance,
            CoreType::Unknown,
        ];
        let labels: Vec<&str> = core_types
            .iter()
            .map(|ct| match ct {
                CoreType::Efficiency => "efficiency",
                CoreType::Performance => "performance",
                CoreType::Unknown => "unknown",
            })
            .collect();
        assert_eq!(labels, vec!["efficiency", "performance", "unknown"]);
    }

    // --- MSeriesBreakdown averages ---

    #[test]
    fn m_series_breakdown_core_counts_are_independent() {
        let breakdown = MSeriesBreakdown {
            efficiency_cores_avg: 12.5,
            performance_cores_avg: 78.3,
            efficiency_core_count: 4,
            performance_core_count: 8,
        };
        assert_eq!(breakdown.efficiency_core_count, 4);
        assert_eq!(breakdown.performance_core_count, 8);
        assert!((breakdown.efficiency_cores_avg - 12.5).abs() < 0.001);
        assert!((breakdown.performance_cores_avg - 78.3).abs() < 0.001);
    }
}
