//! Error types for the application.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("Capture not found: {0}")]
    CaptureNotFound(String),

    #[error("Schedule not found: {0}")]
    ScheduleNotFound(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Error, Debug)]
pub enum TorError {
    #[error("TOR not bootstrapped")]
    NotBootstrapped,

    #[error("TOR connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Circuit creation failed: {0}")]
    CircuitFailed(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),
}

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Browser launch failed: {0}")]
    BrowserLaunchFailed(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    #[error("HTML capture failed: {0}")]
    HtmlCaptureFailed(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("TOR error: {0}")]
    TorError(#[from] TorError),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Duplicate record: {0}")]
    Duplicate(String),

    #[error("File system error: {0}")]
    FileSystemError(String),
}

#[derive(Error, Debug)]
pub enum GDriveError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Folder creation failed: {0}")]
    FolderCreationFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Not configured")]
    NotConfigured,
}

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Scheduler not running")]
    NotRunning,
}
