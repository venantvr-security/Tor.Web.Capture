//! Scheduler for automated captures using cron expressions.

mod job;
mod executor;

pub use job::*;
pub use executor::*;

use chrono::{DateTime, Utc};
use croner::Cron;
use std::str::FromStr;
use tor_capture_core::SchedulerError;

/// Parse and validate a cron expression.
pub fn parse_cron(expression: &str) -> Result<Cron, SchedulerError> {
    Cron::from_str(expression)
        .map_err(|e| SchedulerError::InvalidCron(format!("{}: {}", expression, e)))
}

/// Calculate the next run time for a cron expression.
pub fn next_run_time(expression: &str) -> Result<DateTime<Utc>, SchedulerError> {
    let cron = parse_cron(expression)?;

    cron.find_next_occurrence(&Utc::now(), false)
        .map_err(|e| SchedulerError::InvalidCron(format!("Next run error: {}", e)))
}

/// Validate a cron expression without calculating next run.
pub fn validate_cron(expression: &str) -> Result<(), SchedulerError> {
    parse_cron(expression)?;
    Ok(())
}

/// Format next run time as human-readable string.
pub fn format_next_run(expression: &str) -> String {
    match next_run_time(expression) {
        Ok(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        Err(_) => "Invalid".to_string(),
    }
}
