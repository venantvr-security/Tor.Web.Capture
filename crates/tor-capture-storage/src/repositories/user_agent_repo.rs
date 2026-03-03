//! User agent repository.

use crate::Database;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tor_capture_core::{StorageError, UserAgent};

pub struct UserAgentRepository {
    db: Database,
}

impl UserAgentRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn list_all(&self) -> Result<Vec<UserAgent>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, user_agent_string, category, enabled, usage_count,
                 last_used_at, created_at FROM user_agents ORDER BY name",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let agents = stmt
            .query_map([], |row| Ok(row_to_user_agent(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(agents)
    }

    pub fn list_by_category(&self, category: &str) -> Result<Vec<UserAgent>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, user_agent_string, category, enabled, usage_count,
                 last_used_at, created_at FROM user_agents WHERE category = ?1 AND enabled = 1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let agents = stmt
            .query_map(params![category], |row| Ok(row_to_user_agent(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(agents)
    }

    pub fn list_enabled(&self) -> Result<Vec<UserAgent>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, user_agent_string, category, enabled, usage_count,
                 last_used_at, created_at FROM user_agents WHERE enabled = 1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let agents = stmt
            .query_map([], |row| Ok(row_to_user_agent(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(agents)
    }

    pub fn get_random_enabled(&self) -> Result<Option<UserAgent>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, user_agent_string, category, enabled, usage_count,
                 last_used_at, created_at FROM user_agents WHERE enabled = 1
                 ORDER BY RANDOM() LIMIT 1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row([], |row| Ok(row_to_user_agent(row)))
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(agent) => Ok(Some(agent.map_err(|e| StorageError::DatabaseError(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<Option<UserAgent>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, user_agent_string, category, enabled, usage_count,
                 last_used_at, created_at FROM user_agents WHERE name = ?1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(params![name], |row| Ok(row_to_user_agent(row)))
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(agent) => Ok(Some(agent.map_err(|e| StorageError::DatabaseError(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn increment_usage(&self, id: i64) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        conn.execute(
            "UPDATE user_agents SET usage_count = usage_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn toggle_enabled(&self, id: i64, enabled: bool) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        conn.execute(
            "UPDATE user_agents SET enabled = ?1 WHERE id = ?2",
            params![enabled, id],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

fn row_to_user_agent(row: &rusqlite::Row) -> Result<UserAgent, rusqlite::Error> {
    let last_used_str: Option<String> = row.get(6)?;
    let created_str: String = row.get(7)?;

    Ok(UserAgent {
        id: row.get(0)?,
        name: row.get(1)?,
        user_agent_string: row.get(2)?,
        category: row.get(3)?,
        enabled: row.get(4)?,
        usage_count: row.get(5)?,
        last_used_at: last_used_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
