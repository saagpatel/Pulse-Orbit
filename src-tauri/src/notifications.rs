use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Send a macOS notification when a metric threshold is breached.
pub fn send_threshold_alert(app: &AppHandle, metric_type: &str, value: f64, threshold: f64) {
    let label = metric_type_label(metric_type);
    let body = format!("{label} at {value:.1}% — threshold is {threshold:.1}%");

    if let Err(e) = app
        .notification()
        .builder()
        .title("Pulse Orbit")
        .body(&body)
        .show()
    {
        eprintln!("[pulse-orbit] Notification error: {e}");
    }
}

fn metric_type_label(metric_type: &str) -> &str {
    match metric_type {
        "cpu" => "CPU",
        "memory" => "Memory",
        "disk_write" => "Disk Write",
        "disk_read" => "Disk Read",
        "net_in" => "Network In",
        "net_out" => "Network Out",
        _ => metric_type,
    }
}
