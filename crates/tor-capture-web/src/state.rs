//! Application state shared across handlers.

use std::path::PathBuf;
use std::sync::Arc;
use tor_capture_browser::{CaptureEngine, SpiderEngine};
use tor_capture_network::TorNetworkClient;
use tor_capture_storage::{
    CaptureRepository, ConfigRepository, Database, ScheduleRepository, TargetRepository,
    UserAgentRepository,
};

/// Application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub target_repo: Arc<TargetRepository>,
    pub capture_repo: Arc<CaptureRepository>,
    pub schedule_repo: Arc<ScheduleRepository>,
    pub user_agent_repo: Arc<UserAgentRepository>,
    pub config_repo: Arc<ConfigRepository>,
    pub capture_engine: Arc<CaptureEngine>,
    pub spider_engine: Arc<SpiderEngine>,
    pub tor_client: Arc<Option<TorNetworkClient>>,
}

impl AppState {
    pub fn new(
        db: Database,
        capture_engine: CaptureEngine,
        tor_client: Option<TorNetworkClient>,
        data_dir: PathBuf,
    ) -> Self {
        let capture_engine = Arc::new(capture_engine);
        let spider_engine = Arc::new(SpiderEngine::new(capture_engine.clone(), data_dir));

        Self {
            target_repo: Arc::new(TargetRepository::new(db.clone())),
            capture_repo: Arc::new(CaptureRepository::new(db.clone())),
            schedule_repo: Arc::new(ScheduleRepository::new(db.clone())),
            user_agent_repo: Arc::new(UserAgentRepository::new(db.clone())),
            config_repo: Arc::new(ConfigRepository::new(db.clone())),
            db,
            capture_engine,
            spider_engine,
            tor_client: Arc::new(tor_client),
        }
    }

    /// Check if TOR is connected.
    pub fn is_tor_connected(&self) -> bool {
        self.tor_client
            .as_ref()
            .as_ref()
            .map(|c| c.is_ready())
            .unwrap_or(false)
    }

    /// Get TOR SOCKS address.
    pub fn tor_socks_addr(&self) -> Option<String> {
        self.tor_client
            .as_ref()
            .as_ref()
            .map(|c| c.socks_addr())
    }
}
