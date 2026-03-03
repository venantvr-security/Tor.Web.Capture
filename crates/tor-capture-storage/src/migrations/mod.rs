//! Database migrations.

use rusqlite::Connection;
use tor_capture_core::StorageError;

/// Run all migrations.
pub fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| StorageError::MigrationFailed(e.to_string()))?;

    // Run each migration
    run_migration_v001(conn)?;

    Ok(())
}

fn run_migration_v001(conn: &Connection) -> Result<(), StorageError> {
    // Check if migration already applied
    let applied: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 1)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if applied {
        return Ok(());
    }

    tracing::info!("Running migration v001: Initial schema");

    conn.execute_batch(include_str!("v001_initial.sql"))
        .map_err(|e| StorageError::MigrationFailed(format!("v001: {}", e)))?;

    conn.execute("INSERT INTO schema_migrations (version) VALUES (1)", [])
        .map_err(|e| StorageError::MigrationFailed(e.to_string()))?;

    Ok(())
}
