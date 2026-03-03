//! Schedule repository.

use crate::Database;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tor_capture_core::{Schedule, StorageError};
use uuid::Uuid;

pub struct ScheduleRepository {
    db: Database,
}

impl ScheduleRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, schedule: &Schedule) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        conn.execute(
            "INSERT INTO schedules (id, target_id, cron_expression, timezone, enabled,
             last_run_at, next_run_at, run_count, failure_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                schedule.id.to_string(),
                schedule.target_id.to_string(),
                schedule.cron_expression,
                schedule.timezone,
                schedule.enabled,
                schedule.last_run_at.map(|dt| dt.to_rfc3339()),
                schedule.next_run_at.map(|dt| dt.to_rfc3339()),
                schedule.run_count,
                schedule.failure_count,
                schedule.created_at.to_rfc3339(),
                schedule.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Schedule>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, cron_expression, timezone, enabled,
                 last_run_at, next_run_at, run_count, failure_count, created_at, updated_at
                 FROM schedules WHERE id = ?1",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(params![id.to_string()], |row| Ok(row_to_schedule(row)))
            .optional()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match result {
            Some(schedule) => Ok(Some(schedule.map_err(|e| StorageError::DatabaseError(e.to_string()))?)),
            None => Ok(None),
        }
    }

    pub fn list_by_target(&self, target_id: &Uuid) -> Result<Vec<Schedule>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, cron_expression, timezone, enabled,
                 last_run_at, next_run_at, run_count, failure_count, created_at, updated_at
                 FROM schedules WHERE target_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let schedules = stmt
            .query_map(params![target_id.to_string()], |row| {
                Ok(row_to_schedule(row))
            })
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(schedules)
    }

    pub fn list_enabled(&self) -> Result<Vec<Schedule>, StorageError> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, target_id, cron_expression, timezone, enabled,
                 last_run_at, next_run_at, run_count, failure_count, created_at, updated_at
                 FROM schedules WHERE enabled = 1 ORDER BY next_run_at ASC",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let schedules = stmt
            .query_map([], |row| Ok(row_to_schedule(row)))
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(schedules)
    }

    pub fn update(&self, schedule: &Schedule) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        let affected = conn
            .execute(
                "UPDATE schedules SET cron_expression = ?1, timezone = ?2, enabled = ?3,
                 last_run_at = ?4, next_run_at = ?5, run_count = ?6, failure_count = ?7,
                 updated_at = ?8 WHERE id = ?9",
                params![
                    schedule.cron_expression,
                    schedule.timezone,
                    schedule.enabled,
                    schedule.last_run_at.map(|dt| dt.to_rfc3339()),
                    schedule.next_run_at.map(|dt| dt.to_rfc3339()),
                    schedule.run_count,
                    schedule.failure_count,
                    Utc::now().to_rfc3339(),
                    schedule.id.to_string(),
                ],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(schedule.id.to_string()));
        }

        Ok(())
    }

    pub fn delete(&self, id: &Uuid) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;
        let affected = conn
            .execute(
                "DELETE FROM schedules WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub fn increment_run_count(&self, id: &Uuid, success: bool) -> Result<(), StorageError> {
        let conn = self.db.get_connection()?;

        let query = if success {
            "UPDATE schedules SET run_count = run_count + 1, last_run_at = ?1, updated_at = ?1 WHERE id = ?2"
        } else {
            "UPDATE schedules SET failure_count = failure_count + 1, last_run_at = ?1, updated_at = ?1 WHERE id = ?2"
        };

        conn.execute(query, params![Utc::now().to_rfc3339(), id.to_string()])
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

fn row_to_schedule(row: &rusqlite::Row) -> Result<Schedule, rusqlite::Error> {
    let id_str: String = row.get(0)?;
    let target_id_str: String = row.get(1)?;
    let last_run_str: Option<String> = row.get(5)?;
    let next_run_str: Option<String> = row.get(6)?;
    let created_str: String = row.get(9)?;
    let updated_str: String = row.get(10)?;

    Ok(Schedule {
        id: Uuid::parse_str(&id_str).unwrap_or_default(),
        target_id: Uuid::parse_str(&target_id_str).unwrap_or_default(),
        cron_expression: row.get(2)?,
        timezone: row.get(3)?,
        enabled: row.get(4)?,
        last_run_at: last_run_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        next_run_at: next_run_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        run_count: row.get(7)?,
        failure_count: row.get(8)?,
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
