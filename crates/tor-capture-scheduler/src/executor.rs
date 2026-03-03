//! Job executor and scheduler runtime.

use crate::{CaptureJob, JobResult};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;
use tor_capture_core::{Schedule, SchedulerError};
use uuid::Uuid;

/// Message types for the scheduler.
#[derive(Debug)]
pub enum SchedulerMessage {
    AddJob(CaptureJob),
    RemoveJob(Uuid),
    EnableJob(Uuid),
    DisableJob(Uuid),
    Shutdown,
}

/// Callback type for job execution.
pub type JobCallback = Arc<dyn Fn(Uuid, Uuid) -> JobResult + Send + Sync>;

/// Scheduler that manages and executes capture jobs.
pub struct Scheduler {
    jobs: Arc<RwLock<HashMap<Uuid, CaptureJob>>>,
    tx: mpsc::Sender<SchedulerMessage>,
    running: Arc<RwLock<bool>>,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new() -> (Self, mpsc::Receiver<SchedulerMessage>) {
        let (tx, rx) = mpsc::channel(100);

        let scheduler = Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            tx,
            running: Arc::new(RwLock::new(false)),
        };

        (scheduler, rx)
    }

    /// Add a job to the scheduler.
    pub async fn add_job(&self, job: CaptureJob) -> Result<(), SchedulerError> {
        self.tx
            .send(SchedulerMessage::AddJob(job))
            .await
            .map_err(|_| SchedulerError::NotRunning)?;
        Ok(())
    }

    /// Remove a job from the scheduler.
    pub async fn remove_job(&self, schedule_id: &Uuid) -> Result<(), SchedulerError> {
        self.tx
            .send(SchedulerMessage::RemoveJob(*schedule_id))
            .await
            .map_err(|_| SchedulerError::NotRunning)?;
        Ok(())
    }

    /// Enable a job.
    pub async fn enable_job(&self, schedule_id: &Uuid) -> Result<(), SchedulerError> {
        self.tx
            .send(SchedulerMessage::EnableJob(*schedule_id))
            .await
            .map_err(|_| SchedulerError::NotRunning)?;
        Ok(())
    }

    /// Disable a job.
    pub async fn disable_job(&self, schedule_id: &Uuid) -> Result<(), SchedulerError> {
        self.tx
            .send(SchedulerMessage::DisableJob(*schedule_id))
            .await
            .map_err(|_| SchedulerError::NotRunning)?;
        Ok(())
    }

    /// Shutdown the scheduler.
    pub async fn shutdown(&self) -> Result<(), SchedulerError> {
        self.tx
            .send(SchedulerMessage::Shutdown)
            .await
            .map_err(|_| SchedulerError::NotRunning)?;
        Ok(())
    }

    /// Get all jobs.
    pub fn get_jobs(&self) -> Vec<CaptureJob> {
        self.jobs.read().values().cloned().collect()
    }

    /// Check if scheduler is running.
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Load jobs from schedules.
    pub fn load_schedules(&self, schedules: Vec<Schedule>) {
        let mut jobs = self.jobs.write();
        for schedule in schedules {
            if schedule.enabled {
                let job = CaptureJob::new(
                    schedule.id,
                    schedule.target_id,
                    schedule.cron_expression,
                );
                jobs.insert(schedule.id, job);
            }
        }
    }
}

/// Run the scheduler loop.
pub async fn run_scheduler_loop<F>(
    jobs: Arc<RwLock<HashMap<Uuid, CaptureJob>>>,
    mut rx: mpsc::Receiver<SchedulerMessage>,
    running: Arc<RwLock<bool>>,
    on_job_due: F,
) where
    F: Fn(Uuid, Uuid) + Send + Sync + 'static,
{
    *running.write() = true;
    let mut check_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = check_interval.tick() => {
                // Check for due jobs
                let now = Utc::now();
                let due_jobs: Vec<_> = {
                    let jobs_guard = jobs.read();
                    jobs_guard
                        .values()
                        .filter(|job| job.enabled && job.next_run <= now)
                        .cloned()
                        .collect()
                };

                for job in due_jobs {
                    tracing::info!("Job due: schedule_id={}, target_id={}", job.schedule_id, job.target_id);
                    on_job_due(job.schedule_id, job.target_id);

                    // Update next run time
                    let mut jobs_guard = jobs.write();
                    if let Some(j) = jobs_guard.get_mut(&job.schedule_id) {
                        j.update_next_run();
                    }
                }
            }

            Some(msg) = rx.recv() => {
                match msg {
                    SchedulerMessage::AddJob(job) => {
                        tracing::info!("Adding job: schedule_id={}", job.schedule_id);
                        jobs.write().insert(job.schedule_id, job);
                    }
                    SchedulerMessage::RemoveJob(id) => {
                        tracing::info!("Removing job: schedule_id={}", id);
                        jobs.write().remove(&id);
                    }
                    SchedulerMessage::EnableJob(id) => {
                        if let Some(job) = jobs.write().get_mut(&id) {
                            job.enabled = true;
                            job.update_next_run();
                        }
                    }
                    SchedulerMessage::DisableJob(id) => {
                        if let Some(job) = jobs.write().get_mut(&id) {
                            job.enabled = false;
                        }
                    }
                    SchedulerMessage::Shutdown => {
                        tracing::info!("Scheduler shutting down");
                        break;
                    }
                }
            }
        }
    }

    *running.write() = false;
}
