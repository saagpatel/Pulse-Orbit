use serde::Serialize;
use tauri::State;

use crate::db::DbPool;

#[derive(Clone, Debug, Serialize)]
pub struct HistoryRow {
    pub window_start: String,
    pub value_avg: f64,
    pub value_max: f64,
}

#[tauri::command]
pub fn get_history(
    pool: State<'_, DbPool>,
    metric_type: String,
    interface: Option<String>,
    hours: u32,
) -> Result<Vec<HistoryRow>, String> {
    let conn = pool
        .get()
        .map_err(|e| format!("DB connection error: {e}"))?;

    let hours_str = format!("-{hours} hours");
    let mut stmt = conn
        .prepare(
            "SELECT window_start, value_avg, value_max \
             FROM metric_snapshots \
             WHERE metric_type = ?1 \
               AND (?2 IS NULL OR interface = ?2) \
               AND window_start > datetime('now', ?3) \
             ORDER BY window_start ASC",
        )
        .map_err(|e| format!("Query prepare error: {e}"))?;

    let rows = stmt
        .query_map(
            rusqlite::params![metric_type, interface, hours_str],
            |row| {
                Ok(HistoryRow {
                    window_start: row.get(0)?,
                    value_avg: row.get(1)?,
                    value_max: row.get(2)?,
                })
            },
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        match row {
            Ok(r) => result.push(r),
            Err(e) => eprintln!("[pulse-orbit] Row read error: {e}"),
        }
    }

    Ok(result)
}
