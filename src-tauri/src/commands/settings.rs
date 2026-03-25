use tauri::State;

use crate::db::DbPool;

#[tauri::command]
pub fn get_setting(pool: State<'_, DbPool>, key: String) -> Result<String, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .map_err(|e| format!("Setting '{key}' not found: {e}"))
}

#[tauri::command]
pub fn set_setting(pool: State<'_, DbPool>, key: String, value: String) -> Result<(), String> {
    let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| format!("Failed to save setting: {e}"))?;

    Ok(())
}
