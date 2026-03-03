//! Capture repository.

use crate::Database;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tor_capture_core::{Capture, CaptureStatus, StorageError};
use uuid::Uuid;

pub struct CaptureRepository {
    db: Database,
}

impl CaptureRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, capture: &Capture) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        conn.execute(
            "INSERT INTO captures (id, target_id, schedule_id, status, started_at,
             completed_at, duration_ms, screenshot_path, screenshot_size_bytes,
             html_path, html_size_bytes, page_title, final_url, http_status_code,
             tor_circuit_id, exit_node_ip, exit_node_country, user_agent_used,
             error_message, error_type, gdrive_screenshot_id, gdrive_html_id,
             gdrive_uploaded_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                     ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
            params![
                capture.id.to_string(),
                capture.target_id.to_string(),
                capture.schedule_id.map(|id| id.to_string()),
                capture.status.to_string(),
                capture.started_at.map(|dt| dt.to_rfc3339()),
                capture.completed_at.map(|dt| dt.to_rfc3339()),
                capture.duration_ms,
                capture.screenshot_path,
                capture.screenshot_size_bytes,
                capture.html_path,
                capture.html_size_bytes,
                capture.page_title,
                capture.final_url,
                capture.http_status_code,
                capture.tor_circuit_id,
                capture.exit_node_ip,
                capture.exit_node_country,
                capture.user_agent_used,
                capture.error_message,
                capture.error_type,
                capture.gdrive_screenshot_id,
                capture.gdrive_html_id,
                capture.gdrive_uploaded_at.map(|dt| dt.to_rfc3339()),
                capture.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Capture>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, schedule_id, status, started_at, completed_at,
                 duration_ms, screenshot_path, screenshot_size_bytes, html_path,
                 html_size_bytes, page_title, final_url, http_status_code,
                 tor_circuit_id, exit_node_ip, exit_node_country, user_agent_used,
                 error_message, error_type, gdrive_screenshot_id, gdrive_html_id,
                 gdrive_uploaded_at, created_at
                 FROM captures WHERE id = ?1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(params![id.to_string()], |row| Ok(row_to_capture(row)))
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(capture) => Ok(Some(capture.map_err(|e| StorageError::DatabaseError(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn list_by_target(&self, target_id: &Uuid, limit: usize) -> Result<Vec<Capture>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, schedule_id, status, started_at, completed_at,
                 duration_ms, screenshot_path, screenshot_size_bytes, html_path,
                 html_size_bytes, page_title, final_url, http_status_code,
                 tor_circuit_id, exit_node_ip, exit_node_country, user_agent_used,
                 error_message, error_type, gdrive_screenshot_id, gdrive_html_id,
                 gdrive_uploaded_at, created_at
                 FROM captures WHERE target_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let captures = stmt
            .query_map(params![target_id.to_string(), limit], |row| {
                Ok(row_to_capture(row))
            })
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(captures)
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<Capture>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, schedule_id, status, started_at, completed_at,
                 duration_ms, screenshot_path, screenshot_size_bytes, html_path,
                 html_size_bytes, page_title, final_url, http_status_code,
                 tor_circuit_id, exit_node_ip, exit_node_country, user_agent_used,
                 error_message, error_type, gdrive_screenshot_id, gdrive_html_id,
                 gdrive_uploaded_at, created_at
                 FROM captures ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let captures = stmt
            .query_map(params![limit], |row| Ok(row_to_capture(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(captures)
    }

    pub fn update(&self, capture: &Capture) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        let affected = conn
            .execute(
                "UPDATE captures SET status = ?1, started_at = ?2, completed_at = ?3,
                 duration_ms = ?4, screenshot_path = ?5, screenshot_size_bytes = ?6,
                 html_path = ?7, html_size_bytes = ?8, page_title = ?9, final_url = ?10,
                 http_status_code = ?11, tor_circuit_id = ?12, exit_node_ip = ?13,
                 exit_node_country = ?14, user_agent_used = ?15, error_message = ?16,
                 error_type = ?17, gdrive_screenshot_id = ?18, gdrive_html_id = ?19,
                 gdrive_uploaded_at = ?20
                 WHERE id = ?21",
                params![
                    capture.status.to_string(),
                    capture.started_at.map(|dt| dt.to_rfc3339()),
                    capture.completed_at.map(|dt| dt.to_rfc3339()),
                    capture.duration_ms,
                    capture.screenshot_path,
                    capture.screenshot_size_bytes,
                    capture.html_path,
                    capture.html_size_bytes,
                    capture.page_title,
                    capture.final_url,
                    capture.http_status_code,
                    capture.tor_circuit_id,
                    capture.exit_node_ip,
                    capture.exit_node_country,
                    capture.user_agent_used,
                    capture.error_message,
                    capture.error_type,
                    capture.gdrive_screenshot_id,
                    capture.gdrive_html_id,
                    capture.gdrive_uploaded_at.map(|dt| dt.to_rfc3339()),
                    capture.id.to_string(),
                ],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(capture.id.to_string()));
        }

        Ok(())
    }

    pub fn delete(&self, id: &Uuid) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        let affected = conn
            .execute("DELETE FROM captures WHERE id = ?1", params![id.to_string()])
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub fn count_by_status(&self, status: &CaptureStatus) -> Result<usize, StorageError> {
        let conn = self.db.get_connection()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM captures WHERE status = ?1",
                params![status.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(count as usize)
    }
}

fn row_to_capture(row: &rusqlite::Row) -> Result<Capture, rusqlite::Error> {
    let id_str: String = row.get(0)?;
    let target_id_str: String = row.get(1)?;
    let schedule_id_str: Option<String> = row.get(2)?;
    let status_str: String = row.get(3)?;
    let started_at_str: Option<String> = row.get(4)?;
    let completed_at_str: Option<String> = row.get(5)?;
    let gdrive_uploaded_str: Option<String> = row.get(22)?;
    let created_str: String = row.get(23)?;

    let status = match status_str.as_str() {
        "pending" => CaptureStatus::Pending,
        "running" => CaptureStatus::Running,
        "success" => CaptureStatus::Success,
        "failed" => CaptureStatus::Failed,
        _ => CaptureStatus::Pending,
    };

    Ok(Capture {
        id: Uuid::parse_str(&id_str).unwrap_or_default(),
        target_id: Uuid::parse_str(&target_id_str).unwrap_or_default(),
        schedule_id: schedule_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
        status,
        started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        completed_at: completed_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        duration_ms: row.get(6)?,
        screenshot_path: row.get(7)?,
        screenshot_size_bytes: row.get(8)?,
        html_path: row.get(9)?,
        html_size_bytes: row.get(10)?,
        page_title: row.get(11)?,
        final_url: row.get(12)?,
        http_status_code: row.get(13)?,
        tor_circuit_id: row.get(14)?,
        exit_node_ip: row.get(15)?,
        exit_node_country: row.get(16)?,
        user_agent_used: row.get(17)?,
        error_message: row.get(18)?,
        error_type: row.get(19)?,
        gdrive_screenshot_id: row.get(20)?,
        gdrive_html_id: row.get(21)?,
        gdrive_uploaded_at: gdrive_uploaded_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
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
