use std::collections::HashMap;

use chrono::Local;

use super::DbPool;
use crate::metrics::types::MetricSnapshot;

struct Accumulator {
    sum: f64,
    count: u32,
    max: f64,
}

impl Accumulator {
    fn new() -> Self {
        Self {
            sum: 0.0,
            count: 0,
            max: f64::NEG_INFINITY,
        }
    }

    fn record(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        if value > self.max {
            self.max = value;
        }
    }

    fn avg(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

/// Buffers metric values in memory and flushes 5-minute aggregates to SQLite.
pub struct AggregateBuffer {
    /// (metric_type, interface) → accumulator
    data: HashMap<(String, Option<String>), Accumulator>,
    pub last_flush_minute: u32,
    pub last_purge_hour: u32,
}

impl AggregateBuffer {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            data: HashMap::new(),
            last_flush_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_purge_hour: now.format("%H").to_string().parse().unwrap_or(0),
        }
    }

    /// Record metric values from a snapshot into the accumulator.
    pub fn record(&mut self, snapshot: &MetricSnapshot) {
        // CPU total
        self.record_value("cpu", None, snapshot.cpu.total_percent as f64);

        // Memory as percentage
        if snapshot.memory.total_bytes > 0 {
            let mem_pct =
                (snapshot.memory.used_bytes as f64 / snapshot.memory.total_bytes as f64) * 100.0;
            self.record_value("memory", None, mem_pct);
        }

        // Disk I/O per device
        for dev in &snapshot.disk.devices {
            self.record_value("disk_read", Some(&dev.name), dev.read_bytes_per_sec as f64);
            self.record_value(
                "disk_write",
                Some(&dev.name),
                dev.write_bytes_per_sec as f64,
            );
        }

        // Network per interface
        for iface in &snapshot.network.interfaces {
            self.record_value("net_in", Some(&iface.name), iface.rx_bytes_per_sec as f64);
            self.record_value("net_out", Some(&iface.name), iface.tx_bytes_per_sec as f64);
        }
    }

    fn record_value(&mut self, metric_type: &str, interface: Option<&str>, value: f64) {
        let key = (metric_type.to_string(), interface.map(String::from));
        self.data
            .entry(key)
            .or_insert_with(Accumulator::new)
            .record(value);
    }

    /// Flush accumulated data to SQLite. Call when crossing a 5-minute boundary.
    pub fn flush(&mut self, pool: &DbPool) {
        if self.data.is_empty() {
            return;
        }

        // Window start = current time truncated to 5-minute boundary
        let now = Local::now();
        let minute = now.minute();
        let window_minute = minute - (minute % 5);
        let window_start = now
            .with_minute(window_minute)
            .and_then(|t| t.with_second(0))
            .unwrap_or(now)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let Ok(conn) = pool.get() else {
            eprintln!("[pulse-orbit] Failed to get DB connection for flush");
            return;
        };

        let mut inserted = 0u32;
        for ((metric_type, interface), acc) in self.data.drain() {
            if acc.count == 0 {
                continue;
            }
            let result = conn.execute(
                "INSERT INTO metric_snapshots (metric_type, interface, value_avg, value_max, window_start) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![metric_type, interface, acc.avg(), acc.max, window_start],
            );
            if let Err(e) = result {
                eprintln!("[pulse-orbit] Failed to insert metric row: {e}");
            } else {
                inserted += 1;
            }
        }

        if inserted > 0 {
            eprintln!("[pulse-orbit] Flushed {inserted} aggregate rows (window: {window_start})");
        }

        // Check for hourly purge
        let current_hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
        if current_hour != self.last_purge_hour {
            self.last_purge_hour = current_hour;
            super::purge_old_rows(pool.as_ref());
        }
    }
}

use chrono::Timelike;
