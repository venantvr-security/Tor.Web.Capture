//! Configuration repository.

use crate::Database;
use rusqlite::params;
use std::collections::HashMap;
use tor_capture_core::StorageError;

pub struct ConfigRepository {
    db: Database,
}

impl ConfigRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT value FROM app_config WHERE key = ?1")
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result: Result<String, _> = stmt.query_row(params![key], |row| row.get(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<Option<bool>, StorageError> {
        self.get(key).map(|opt| {
            opt.map(|v| v.to_lowercase() == "true" || v == "1")
        })
    }

    pub fn get_int(&self, key: &str) -> Result<Option<i64>, StorageError> {
        self.get(key).map(|opt| {
            opt.and_then(|v| v.parse().ok())
        })
    }

    pub fn set(&self, key: &str, value: &str, value_type: &str) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        conn.execute(
            "INSERT INTO app_config (key, value, value_type, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
            params![key, value, value_type],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.set(key, value, "string")
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), StorageError> {
        self.set(key, if value { "true" } else { "false" }, "bool")
    }

    pub fn set_int(&self, key: &str, value: i64) -> Result<(), StorageError> {
        self.set(key, &value.to_string(), "int")
    }

    pub fn get_all(&self) -> Result<HashMap<String, String>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM app_config")
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let config = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(config)
    }

    pub fn delete(&self, key: &str) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        conn.execute("DELETE FROM app_config WHERE key = ?1", params![key])
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
