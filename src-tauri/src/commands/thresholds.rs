use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbPool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub metric_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub cooldown_seconds: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DbInfo {
    pub row_count: u64,
    pub file_size_bytes: u64,
    pub path: String,
}

#[tauri::command]
pub fn get_thresholds(pool: State<'_, DbPool>) -> Result<Vec<AlertThreshold>, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT metric_type, threshold, enabled, cooldown_seconds \
             FROM alert_thresholds ORDER BY metric_type",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AlertThreshold {
                metric_type: row.get(0)?,
                threshold: row.get(1)?,
                enabled: row.get::<_, i32>(2)? != 0,
                cooldown_seconds: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    let mut result = Vec::new();
    for r in rows.flatten() {
        result.push(r);
    }
    Ok(result)
}

#[tauri::command]
pub fn set_threshold(pool: State<'_, DbPool>, threshold: AlertThreshold) -> Result<(), String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO alert_thresholds \
         (metric_type, threshold, enabled, cooldown_seconds, updated_at) \
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        rusqlite::params![
            threshold.metric_type,
            threshold.threshold,
            threshold.enabled as i32,
            threshold.cooldown_seconds,
        ],
    )
    .map_err(|e| format!("Insert error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_db_info(pool: State<'_, DbPool>) -> Result<DbInfo, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

    let row_count: u64 = conn
        .query_row("SELECT COUNT(*) FROM metric_snapshots", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let path: String = conn
        .query_row("PRAGMA database_list", [], |row| row.get::<_, String>(2))
        .unwrap_or_default();

    let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(DbInfo {
        row_count,
        file_size_bytes,
        path,
    })
}
