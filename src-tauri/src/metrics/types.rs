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
}
