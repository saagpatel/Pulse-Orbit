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

/// Pure predicate: returns true when a metric value should trigger an alert.
/// Extracted for testability — no Tauri or DB dependencies.
pub(crate) fn threshold_breached(value: f64, threshold: f64, enabled: bool) -> bool {
    enabled && value > threshold
}

/// Converts raw bytes-per-second to MB/s for threshold comparison.
pub(crate) fn bytes_per_sec_to_mbps(bytes_per_sec: u64) -> f64 {
    bytes_per_sec as f64 / 1_048_576.0
}

/// Returns the maximum value across an iterator, or 0.0 if the iterator is empty.
pub(crate) fn max_of(values: impl Iterator<Item = f64>) -> f64 {
    values.fold(0.0f64, f64::max)
}

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
        let max_disk_write = max_of(
            snapshot
                .disk
                .devices
                .iter()
                .map(|d| bytes_per_sec_to_mbps(d.write_bytes_per_sec)),
        );
        self.check_metric("disk_write", max_disk_write, app);

        // Net in (max across all interfaces, in MB/s)
        let max_net_in = max_of(
            snapshot
                .network
                .interfaces
                .iter()
                .map(|i| bytes_per_sec_to_mbps(i.rx_bytes_per_sec)),
        );
        self.check_metric("net_in", max_net_in, app);
    }

    fn check_metric(&mut self, metric_type: &str, value: f64, app: &AppHandle) {
        let Some(cached) = self.thresholds.get_mut(metric_type) else {
            return;
        };

        if !threshold_breached(value, cached.threshold, cached.enabled) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- threshold_breached ---

    #[test]
    fn breach_when_value_exceeds_threshold_and_enabled() {
        assert_eq!(threshold_breached(95.0, 80.0, true), true);
    }

    #[test]
    fn no_breach_when_value_equals_threshold() {
        // Strictly greater-than — equal does not fire
        assert_eq!(threshold_breached(80.0, 80.0, true), false);
    }

    #[test]
    fn no_breach_when_value_below_threshold() {
        assert_eq!(threshold_breached(50.0, 80.0, true), false);
    }

    #[test]
    fn no_breach_when_disabled_even_if_value_exceeds() {
        assert_eq!(threshold_breached(99.0, 10.0, false), false);
    }

    #[test]
    fn no_breach_at_zero_value_zero_threshold_enabled() {
        // 0.0 is not strictly greater than 0.0
        assert_eq!(threshold_breached(0.0, 0.0, true), false);
    }

    // --- bytes_per_sec_to_mbps ---

    #[test]
    fn one_mebibyte_per_sec_equals_one_mbps() {
        let result = bytes_per_sec_to_mbps(1_048_576);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn zero_bytes_per_sec_is_zero_mbps() {
        assert_eq!(bytes_per_sec_to_mbps(0), 0.0);
    }

    #[test]
    fn ten_mebibytes_per_sec() {
        let result = bytes_per_sec_to_mbps(10 * 1_048_576);
        assert_eq!(result, 10.0);
    }

    // --- max_of ---

    #[test]
    fn max_of_empty_iterator_returns_zero() {
        assert_eq!(max_of(std::iter::empty()), 0.0);
    }

    #[test]
    fn max_of_single_value() {
        assert_eq!(max_of([42.5_f64].into_iter()), 42.5);
    }

    #[test]
    fn max_of_multiple_values() {
        assert_eq!(max_of([3.0_f64, 7.5, 1.2, 5.0].into_iter()), 7.5);
    }

    #[test]
    fn max_of_all_equal_values() {
        assert_eq!(max_of([4.0_f64, 4.0, 4.0].into_iter()), 4.0);
    }
}
