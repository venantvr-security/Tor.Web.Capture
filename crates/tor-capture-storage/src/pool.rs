//! Database connection pool management.

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use tor_capture_core::StorageError;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

/// Create a new database connection pool.
pub fn create_pool(database_path: &Path, pool_size: u32) -> Result<DbPool, StorageError> {
    // Ensure parent directory exists
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            StorageError::FileSystemError(format!("Failed to create database directory: {}", e))
        })?;
    }

    let manager = SqliteConnectionManager::file(database_path);
    let pool = Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .map_err(|e| StorageError::DatabaseError(format!("Failed to create pool: {}", e)))?;

    // Initialize database with pragmas
    let conn = pool
        .get()
        .map_err(|e| StorageError::DatabaseError(format!("Failed to get connection: {}", e)))?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        ",
    )
    .map_err(|e| StorageError::DatabaseError(format!("Failed to set pragmas: {}", e)))?;

    Ok(pool)
}

/// Database wrapper with pool.
#[derive(Clone)]
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(database_path: &Path, pool_size: u32) -> Result<Self, StorageError> {
        let pool = create_pool(database_path, pool_size)?;
        let db = Self { pool };

        // Run migrations
        db.migrate()?;

        Ok(db)
    }

    pub fn get_connection(&self) -> Result<DbConnection, StorageError> {
        self.pool
            .get()
            .map_err(|e| StorageError::DatabaseError(format!("Failed to get connection: {}", e)))
    }

    /// Run database migrations.
    pub fn migrate(&self) -> Result<(), StorageError> {
        let conn = self.get_connection()?;
        crate::migrations::run_migrations(&conn)
    }
}
