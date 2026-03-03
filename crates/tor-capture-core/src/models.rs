//! Core data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Target website to capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub capture_screenshot: bool,
    pub capture_html: bool,
    pub user_agent_type: UserAgentType,
    pub custom_user_agent: Option<String>,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub wait_after_load_ms: u64,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Target {
    pub fn new(name: String, url: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            url,
            enabled: true,
            capture_screenshot: true,
            capture_html: true,
            user_agent_type: UserAgentType::Random,
            custom_user_agent: None,
            viewport_width: 1920,
            viewport_height: 1080,
            wait_after_load_ms: 2000,
            tags: Vec::new(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Type of user agent to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserAgentType {
    Shodan,
    Censys,
    ZGrab,
    Masscan,
    Nmap,
    BinaryEdge,
    Fofa,
    ZoomEye,
    GreyNoise,
    Random,
    Custom,
}

impl Default for UserAgentType {
    fn default() -> Self {
        Self::Random
    }
}

impl std::fmt::Display for UserAgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shodan => write!(f, "shodan"),
            Self::Censys => write!(f, "censys"),
            Self::ZGrab => write!(f, "zgrab"),
            Self::Masscan => write!(f, "masscan"),
            Self::Nmap => write!(f, "nmap"),
            Self::BinaryEdge => write!(f, "binaryedge"),
            Self::Fofa => write!(f, "fofa"),
            Self::ZoomEye => write!(f, "zoomeye"),
            Self::GreyNoise => write!(f, "greynoise"),
            Self::Random => write!(f, "random"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Schedule for automatic captures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub target_id: Uuid,
    pub cron_expression: String,
    pub timezone: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub run_count: u32,
    pub failure_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Schedule {
    pub fn new(target_id: Uuid, cron_expression: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            target_id,
            cron_expression,
            timezone: "UTC".to_string(),
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            run_count: 0,
            failure_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Status of a capture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl Default for CaptureStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for CaptureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Result of a capture operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capture {
    pub id: Uuid,
    pub target_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub status: CaptureStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,

    // Capture data
    pub screenshot_path: Option<String>,
    pub screenshot_size_bytes: Option<u64>,
    pub html_path: Option<String>,
    pub html_size_bytes: Option<u64>,
    pub page_title: Option<String>,
    pub final_url: Option<String>,
    pub http_status_code: Option<u16>,

    // TOR metadata
    pub tor_circuit_id: Option<String>,
    pub exit_node_ip: Option<String>,
    pub exit_node_country: Option<String>,

    // User agent used
    pub user_agent_used: Option<String>,

    // Errors
    pub error_message: Option<String>,
    pub error_type: Option<String>,

    // Google Drive upload
    pub gdrive_screenshot_id: Option<String>,
    pub gdrive_html_id: Option<String>,
    pub gdrive_uploaded_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

impl Capture {
    pub fn new(target_id: Uuid, schedule_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            target_id,
            schedule_id,
            status: CaptureStatus::Pending,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            screenshot_path: None,
            screenshot_size_bytes: None,
            html_path: None,
            html_size_bytes: None,
            page_title: None,
            final_url: None,
            http_status_code: None,
            tor_circuit_id: None,
            exit_node_ip: None,
            exit_node_country: None,
            user_agent_used: None,
            error_message: None,
            error_type: None,
            gdrive_screenshot_id: None,
            gdrive_html_id: None,
            gdrive_uploaded_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn mark_started(&mut self) {
        self.status = CaptureStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn mark_success(&mut self) {
        self.status = CaptureStatus::Success;
        self.completed_at = Some(Utc::now());
        if let Some(started) = self.started_at {
            self.duration_ms = Some((Utc::now() - started).num_milliseconds() as u64);
        }
    }

    pub fn mark_failed(&mut self, error_type: &str, error_message: &str) {
        self.status = CaptureStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_type = Some(error_type.to_string());
        self.error_message = Some(error_message.to_string());
        if let Some(started) = self.started_at {
            self.duration_ms = Some((Utc::now() - started).num_milliseconds() as u64);
        }
    }
}

/// Stored user agent string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgent {
    pub id: i64,
    pub name: String,
    pub user_agent_string: String,
    pub category: String,
    pub enabled: bool,
    pub usage_count: u32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Google Drive configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GDriveConfig {
    pub auth_type: GDriveAuthType,
    pub client_id: Option<String>,
    pub client_secret_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub service_account_json_encrypted: Option<String>,
    pub target_folder_id: Option<String>,
    pub auto_upload: bool,
    pub upload_screenshots: bool,
    pub upload_html: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GDriveAuthType {
    OAuth2,
    ServiceAccount,
}

/// Operation log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub operation_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub message: String,
    pub level: LogLevel,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Raw capture result from browser.
#[derive(Debug)]
pub struct CaptureResult {
    pub screenshot_data: Option<Vec<u8>>,
    pub html_content: Option<String>,
    pub final_url: String,
    pub page_title: Option<String>,
    pub http_status_code: Option<u16>,
    pub captured_at: DateTime<Utc>,
}

/// TOR circuit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorCircuitInfo {
    pub circuit_id: String,
    pub exit_node_ip: Option<String>,
    pub exit_node_country: Option<String>,
    pub is_ready: bool,
}
