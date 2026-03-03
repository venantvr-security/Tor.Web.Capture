//! Scheduled job definitions.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A scheduled capture job.
#[derive(Debug, Clone)]
pub struct CaptureJob {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub target_id: Uuid,
    pub cron_expression: String,
    pub next_run: DateTime<Utc>,
    pub enabled: bool,
}

impl CaptureJob {
    pub fn new(schedule_id: Uuid, target_id: Uuid, cron_expression: String) -> Self {
        let next_run = super::next_run_time(&cron_expression).unwrap_or_else(|_| Utc::now());

        Self {
            id: Uuid::new_v4(),
            schedule_id,
            target_id,
            cron_expression,
            next_run,
            enabled: true,
        }
    }

    /// Update the next run time.
    pub fn update_next_run(&mut self) {
        if let Ok(next) = super::next_run_time(&self.cron_expression) {
            self.next_run = next;
        }
    }

    /// Check if job should run now.
    pub fn should_run(&self) -> bool {
        self.enabled && Utc::now() >= self.next_run
    }
}

/// Job execution result.
#[derive(Debug)]
pub enum JobResult {
    Success {
        capture_id: Uuid,
        duration_ms: u64,
    },
    Failed {
        error: String,
    },
    Skipped {
        reason: String,
    },
}
