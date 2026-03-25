use std::collections::HashMap;
use std::time::Instant;

use tauri::AppHandle;

use super::types::MetricSnapshot;
use crate::db::DbPool;
use crate::notifications;

struct CachedThreshold {
    threshold: f64,
    enabled: bool,
    cooldown_secs: u32,
    last_fired: Option<Instant>,
}

/// Checks metric values against stored thresholds and fires notifications.
pub struct ThresholdChecker {
    thresholds: HashMap<String, CachedThreshold>,
    last_reload: Instant,
}

const RELOAD_INTERVAL_SECS: u64 = 60;

impl ThresholdChecker {
    pub fn new(pool: &DbPool) -> Self {
        let mut checker = Self {
            thresholds: HashMap::new(),
            last_reload: Instant::now(),
        };
        checker.reload_from_db(pool);
        checker
    }

    /// Check the snapshot against thresholds. Fire notifications if breached.
    pub fn check(&mut self, snapshot: &MetricSnapshot, app: &AppHandle, pool: &DbPool) {
        // Periodically reload thresholds from DB
        if self.last_reload.elapsed().as_secs() >= RELOAD_INTERVAL_SECS {
            self.reload_from_db(pool);
            self.last_reload = Instant::now();
        }

        // CPU
        self.check_metric("cpu", snapshot.cpu.total_percent as f64, app);

        // Memory
        if snapshot.memory.total_bytes > 0 {
            let mem_pct =
                (snapshot.memory.used_bytes as f64 / snapshot.memory.total_bytes as f64) * 100.0;
            self.check_metric("memory", mem_pct, app);
        }

        // Disk write (max across all devices, in MB/s)
        let max_disk_write = snapshot
            .disk
            .devices
            .iter()
            .map(|d| d.write_bytes_per_sec as f64 / 1_048_576.0)
            .fold(0.0f64, f64::max);
        self.check_metric("disk_write", max_disk_write, app);

        // Net in (max across all interfaces, in MB/s)
        let max_net_in = snapshot
            .network
            .interfaces
            .iter()
            .map(|i| i.rx_bytes_per_sec as f64 / 1_048_576.0)
            .fold(0.0f64, f64::max);
        self.check_metric("net_in", max_net_in, app);
    }

    fn check_metric(&mut self, metric_type: &str, value: f64, app: &AppHandle) {
        let Some(cached) = self.thresholds.get_mut(metric_type) else {
            return;
        };

        if !cached.enabled || value <= cached.threshold {
            return;
        }

        // Check cooldown
        if let Some(last_fired) = cached.last_fired {
            if last_fired.elapsed().as_secs() < cached.cooldown_secs as u64 {
                return;
            }
        }

        // Fire notification
        notifications::send_threshold_alert(app, metric_type, value, cached.threshold);
        cached.last_fired = Some(Instant::now());
        eprintln!(
            "[pulse-orbit] Alert: {metric_type} = {value:.1} > threshold {:.1}",
            cached.threshold
        );
    }

    fn reload_from_db(&mut self, pool: &DbPool) {
        let Ok(conn) = pool.get() else { return };

        let Ok(mut stmt) = conn.prepare(
            "SELECT metric_type, threshold, enabled, cooldown_seconds FROM alert_thresholds",
        ) else {
            return;
        };

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, u32>(3)?,
            ))
        }) else {
            return;
        };

        for row in rows.flatten() {
            let (metric_type, threshold, enabled, cooldown_secs) = row;
            let existing_last_fired = self.thresholds.get(&metric_type).and_then(|c| c.last_fired);

            self.thresholds.insert(
                metric_type,
                CachedThreshold {
                    threshold,
                    enabled: enabled != 0,
                    cooldown_secs,
                    last_fired: existing_last_fired,
                },
            );
        }
    }
}
