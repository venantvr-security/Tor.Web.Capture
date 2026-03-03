//! Target repository.

use crate::Database;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tor_capture_core::{StorageError, Target, UserAgentType};
use uuid::Uuid;

pub struct TargetRepository {
    db: Database,
}

impl TargetRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, target: &Target) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        let tags_json = serde_json::to_string(&target.tags).unwrap_or_default();

        conn.execute(
            "INSERT INTO targets (id, name, url, enabled, capture_screenshot, capture_html,
             user_agent_type, custom_user_agent, viewport_width, viewport_height,
             wait_after_load_ms, tags, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                target.id.to_string(),
                target.name,
                target.url,
                target.enabled,
                target.capture_screenshot,
                target.capture_html,
                target.user_agent_type.to_string(),
                target.custom_user_agent,
                target.viewport_width,
                target.viewport_height,
                target.wait_after_load_ms,
                tags_json,
                target.notes,
                target.created_at.to_rfc3339(),
                target.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Target>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, url, enabled, capture_screenshot, capture_html,
                 user_agent_type, custom_user_agent, viewport_width, viewport_height,
                 wait_after_load_ms, tags, notes, created_at, updated_at
                 FROM targets WHERE id = ?1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(row_to_target(row))
            })
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(target) => Ok(Some(target.map_err(|e| StorageError::DatabaseError(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn list_all(&self) -> Result<Vec<Target>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, url, enabled, capture_screenshot, capture_html,
                 user_agent_type, custom_user_agent, viewport_width, viewport_height,
                 wait_after_load_ms, tags, notes, created_at, updated_at
                 FROM targets ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let targets = stmt
            .query_map([], |row| Ok(row_to_target(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(targets)
    }

    pub fn list_enabled(&self) -> Result<Vec<Target>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, url, enabled, capture_screenshot, capture_html,
                 user_agent_type, custom_user_agent, viewport_width, viewport_height,
                 wait_after_load_ms, tags, notes, created_at, updated_at
                 FROM targets WHERE enabled = 1 ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let targets = stmt
            .query_map([], |row| Ok(row_to_target(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(targets)
    }

    pub fn update(&self, target: &Target) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        let tags_json = serde_json::to_string(&target.tags).unwrap_or_default();

        let affected = conn
            .execute(
                "UPDATE targets SET name = ?1, url = ?2, enabled = ?3,
                 capture_screenshot = ?4, capture_html = ?5, user_agent_type = ?6,
                 custom_user_agent = ?7, viewport_width = ?8, viewport_height = ?9,
                 wait_after_load_ms = ?10, tags = ?11, notes = ?12, updated_at = ?13
                 WHERE id = ?14",
                params![
                    target.name,
                    target.url,
                    target.enabled,
                    target.capture_screenshot,
                    target.capture_html,
                    target.user_agent_type.to_string(),
                    target.custom_user_agent,
                    target.viewport_width,
                    target.viewport_height,
                    target.wait_after_load_ms,
                    tags_json,
                    target.notes,
                    Utc::now().to_rfc3339(),
                    target.id.to_string(),
                ],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(target.id.to_string()));
        }

        Ok(())
    }

    pub fn delete(&self, id: &Uuid) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        let affected = conn
            .execute("DELETE FROM targets WHERE id = ?1", params![id.to_string()])
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }
}

fn row_to_target(row: &rusqlite::Row) -> Result<Target, rusqlite::Error> {
    let id_str: String = row.get(0)?;
    let user_agent_type_str: String = row.get(6)?;
    let tags_json: String = row.get(11)?;
    let created_str: String = row.get(13)?;
    let updated_str: String = row.get(14)?;

    let user_agent_type = match user_agent_type_str.as_str() {
        "shodan" => UserAgentType::Shodan,
        "censys" => UserAgentType::Censys,
        "zgrab" => UserAgentType::ZGrab,
        "masscan" => UserAgentType::Masscan,
        "nmap" => UserAgentType::Nmap,
        "binaryedge" => UserAgentType::BinaryEdge,
        "fofa" => UserAgentType::Fofa,
        "zoomeye" => UserAgentType::ZoomEye,
        "greynoise" => UserAgentType::GreyNoise,
        "custom" => UserAgentType::Custom,
        _ => UserAgentType::Random,
    };

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

    Ok(Target {
        id: Uuid::parse_str(&id_str).unwrap_or_default(),
        name: row.get(1)?,
        url: row.get(2)?,
        enabled: row.get(3)?,
        capture_screenshot: row.get(4)?,
        capture_html: row.get(5)?,
        user_agent_type,
        custom_user_agent: row.get(7)?,
        viewport_width: row.get(8)?,
        viewport_height: row.get(9)?,
        wait_after_load_ms: row.get(10)?,
        tags,
        notes: row.get(12)?,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
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
