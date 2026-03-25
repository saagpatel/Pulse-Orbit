/** Emitted by Rust every 2 seconds via Tauri event 'metric-snapshot' */
export interface MetricSnapshot {
	timestamp: number;
	cpu: CpuMetrics;
	memory: MemoryMetrics;
	disk: DiskMetrics;
	network: NetworkMetrics;
	processes: ProcessInfo[];
}

export interface CpuMetrics {
	total_percent: number;
	per_core: CoreMetric[];
	frequency_mhz: number;
	m_series_breakdown: MSeriesBreakdown | null;
}

export interface CoreMetric {
	core_index: number;
	percent: number;
	core_type: "efficiency" | "performance" | "unknown";
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
	pressure: "normal" | "warn" | "critical";
}

export interface DiskMetrics {
	devices: DiskDevice[];
}

export interface DiskDevice {
	name: string;
	read_bytes_per_sec: number;
	write_bytes_per_sec: number;
	total_bytes: number;
	used_bytes: number;
}

export interface NetworkMetrics {
	interfaces: NetworkInterface[];
}

export interface NetworkInterface {
	name: string;
	rx_bytes_per_sec: number;
	tx_bytes_per_sec: number;
	total_rx_bytes: number;
	total_tx_bytes: number;
}

export interface ProcessInfo {
	pid: number;
	name: string;
	cpu_percent: number;
	memory_bytes: number;
	status: string;
}

/** SQLite aggregate row returned by get-history command (Phase 2) */
export interface HistoryRow {
	window_start: string;
	value_avg: number;
	value_max: number;
}

/** Alert threshold config (Phase 3) */
export interface AlertThreshold {
	metric_type: string;
	threshold: number;
	enabled: boolean;
	cooldown_seconds: number;
}
