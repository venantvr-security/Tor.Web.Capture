//! Tor.Web.Capture - Main entry point.
//!
//! A Rust application for capturing web pages via TOR with:
//! - Integrated TOR client (Arti)
//! - Dynamic web interface (HTMX)
//! - Screenshot + HTML capture
//! - IoT bot user agents
//! - Google Drive upload
//! - SQLite storage

use anyhow::Result;
use std::path::PathBuf;
use tor_capture_browser::CaptureEngine;
use tor_capture_core::{AppConfig, CaptureConfig, TorConfig, WebConfig};
use tor_capture_network::TorNetworkClient;
use tor_capture_storage::Database;
use tor_capture_web::{start_server, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tor_web_capture=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Tor.Web.Capture...");

    // Load configuration
    let config = load_config()?;

    // Initialize database
    tracing::info!("Initializing database...");
    let db = Database::new(&config.storage.database_path, config.storage.pool_size)?;

    // Initialize TOR client (optional - can run without TOR for testing)
    let tor_client = if config.tor.enabled {
        tracing::info!("Bootstrapping TOR client...");
        match TorNetworkClient::new(&config.tor).await {
            Ok(client) => {
                tracing::info!("TOR client ready");
                Some(client)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize TOR: {}. Running without TOR.", e);
                None
            }
        }
    } else {
        tracing::info!("TOR disabled in configuration");
        None
    };

    // Get SOCKS address
    let socks_addr = tor_client
        .as_ref()
        .map(|c| c.socks_addr())
        .unwrap_or_else(|| "127.0.0.1:9050".to_string());

    // Initialize capture engine
    let capture_engine = CaptureEngine::new(&socks_addr, config.capture.clone());

    // Data directory for spider config and other state
    let data_dir = config
        .capture
        .storage_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"));

    // Create application state
    let state = AppState::new(db, capture_engine, tor_client, data_dir);

    // Start scheduler in background
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_scheduler(scheduler_state).await {
            tracing::error!("Scheduler error: {}", e);
        }
    });

    // Start web server
    tracing::info!(
        "Starting web server on http://{}:{}",
        config.web.bind_address,
        config.web.port
    );

    start_server(state, &config.web).await?;

    Ok(())
}

/// Load configuration from file or defaults.
fn load_config() -> Result<AppConfig> {
    // Try to load from config file
    let config_path = PathBuf::from("config/default.toml");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        return Ok(config);
    }

    // Use defaults
    Ok(AppConfig {
        web: WebConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            static_files_path: PathBuf::from("./static"),
            templates_path: PathBuf::from("./templates"),
        },
        tor: TorConfig {
            enabled: true,
            data_dir: PathBuf::from("./data/tor"),
            new_circuit_per_capture: true,
            bootstrap_timeout_secs: 120,
            socks_port: 9150,
        },
        capture: CaptureConfig {
            storage_path: PathBuf::from("./data/captures"),
            max_concurrent_captures: 3,
            default_timeout_ms: 30000,
            chrome_executable_path: None,
            default_viewport_width: 1920,
            default_viewport_height: 1080,
            default_wait_after_load_ms: 2000,
        },
        storage: tor_capture_core::StorageConfig {
            database_path: PathBuf::from("./data/tor-capture.db"),
            pool_size: 5,
        },
        gdrive: tor_capture_core::GDriveSettings::default(),
    })
}

/// Run the capture scheduler.
async fn run_scheduler(state: AppState) -> Result<()> {
    use parking_lot::RwLock;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tor_capture_scheduler::{run_scheduler_loop, Scheduler};

    // Load enabled schedules
    let schedules = state.schedule_repo.list_enabled().unwrap_or_default();

    let (scheduler, rx) = Scheduler::new();
    scheduler.load_schedules(schedules);

    let jobs = Arc::new(RwLock::new(HashMap::new()));
    let running = Arc::new(RwLock::new(false));

    // Clone state for callback
    let callback_state = state.clone();

    run_scheduler_loop(jobs, rx, running, move |schedule_id, target_id| {
        let state = callback_state.clone();

        // Spawn capture task
        tokio::spawn(async move {
            tracing::info!("Scheduled capture triggered: target={}", target_id);

            let target = match state.target_repo.get(&target_id) {
                Ok(Some(t)) => t,
                _ => {
                    tracing::error!("Target not found: {}", target_id);
                    return;
                }
            };

            // Create capture
            let mut capture = tor_capture_core::Capture::new(target.id, Some(schedule_id));
            if let Err(e) = state.capture_repo.create(&capture) {
                tracing::error!("Failed to create capture: {}", e);
                return;
            }

            capture.mark_started();
            let _ = state.capture_repo.update(&capture);

            // Execute capture
            match state.capture_engine.execute_capture(&target).await {
                Ok(result) => {
                    if let Err(e) = state
                        .capture_engine
                        .save_capture(&target, &result, &mut capture)
                        .await
                    {
                        capture.mark_failed("save_error", &e.to_string());
                    } else {
                        capture.mark_success();
                    }
                }
                Err(e) => {
                    capture.mark_failed("capture_error", &e.to_string());
                }
            }

            let _ = state.capture_repo.update(&capture);

            // Update schedule stats
            let success = capture.status == tor_capture_core::CaptureStatus::Success;
            let _ = state.schedule_repo.increment_run_count(&schedule_id, success);
        });
    })
    .await;

    Ok(())
}
