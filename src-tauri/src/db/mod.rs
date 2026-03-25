pub mod writer;

use std::sync::Arc;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::{AppHandle, Manager};

pub type DbPool = Arc<Pool<SqliteConnectionManager>>;

const MIGRATION_001: &str = "
CREATE TABLE IF NOT EXISTS metric_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_type TEXT    NOT NULL,
    interface   TEXT,
    value_avg   REAL    NOT NULL,
    value_max   REAL    NOT NULL,
    window_start DATETIME NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_snapshots_type_time ON metric_snapshots(metric_type, window_start DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_window    ON metric_snapshots(window_start DESC);
";

/// Initialize the SQLite connection pool and run migrations.
pub fn init(app: &AppHandle) -> DbPool {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");

    std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");

    let db_path = app_data_dir.join("metrics.db");
    eprintln!("[pulse-orbit] DB path: {}", db_path.display());

    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(3)
        .build(manager)
        .expect("failed to create connection pool");

    // Run migrations and set WAL mode
    {
        let conn = pool.get().expect("failed to get connection for migrations");
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("failed to set WAL mode");
        conn.execute_batch(MIGRATION_001)
            .expect("failed to run migrations");
    }

    // Initial purge of stale rows
    purge_old_rows(&pool);

    Arc::new(pool)
}

/// Delete metric rows older than 24 hours.
pub fn purge_old_rows(pool: &Pool<SqliteConnectionManager>) {
    if let Ok(conn) = pool.get() {
        let deleted = conn
            .execute(
                "DELETE FROM metric_snapshots WHERE window_start < datetime('now', '-24 hours')",
                [],
            )
            .unwrap_or(0);
        if deleted > 0 {
            eprintln!("[pulse-orbit] Purged {deleted} stale metric rows");
        }
    }
}
