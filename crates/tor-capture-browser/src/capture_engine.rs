//! High-level capture engine.

use crate::{capture_page, ChromeBrowser, ScreenshotOptions, Viewport};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tor_capture_core::{
    Capture, CaptureConfig, CaptureError, CaptureResult, IotUserAgents, Target, UserAgentType,
};
use uuid::Uuid;

/// Capture engine that manages browser instances and captures.
pub struct CaptureEngine {
    socks_addr: String,
    config: CaptureConfig,
    semaphore: Arc<Semaphore>,
}

impl CaptureEngine {
    /// Create a new capture engine.
    pub fn new(socks_addr: &str, config: CaptureConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_captures));

        Self {
            socks_addr: socks_addr.to_string(),
            config,
            semaphore,
        }
    }

    /// Execute a capture for a target.
    pub async fn execute_capture(&self, target: &Target) -> Result<CaptureResult, CaptureError> {
        // Acquire semaphore permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| CaptureError::BrowserLaunchFailed(format!("Semaphore error: {}", e)))?;

        // Get user agent
        let user_agent = self.get_user_agent(target);

        // Create screenshot options
        let options = ScreenshotOptions {
            viewport: Viewport {
                width: target.viewport_width as u32,
                height: target.viewport_height as u32,
            },
            full_page: true,
            wait_after_load_ms: target.wait_after_load_ms as u64,
            ..Default::default()
        };

        // Run capture in blocking task (Chrome is not async)
        let url = target.url.clone();
        let capture_screenshot = target.capture_screenshot;
        let capture_html = target.capture_html;
        let socks_addr = self.socks_addr.clone();
        let chrome_path = self.config.chrome_executable_path.clone();

        tokio::task::spawn_blocking(move || {
            // Launch browser
            let browser = ChromeBrowser::new(&socks_addr, chrome_path.as_ref(), false)?;

            // Create new tab
            let tab = browser.new_tab()?;

            // Capture page
            capture_page(
                &tab,
                &url,
                &user_agent,
                &options,
                capture_screenshot,
                capture_html,
            )
        })
        .await
        .map_err(|e| CaptureError::BrowserLaunchFailed(format!("Task error: {}", e)))?
    }

    /// Save capture result to disk.
    pub async fn save_capture(
        &self,
        target: &Target,
        result: &CaptureResult,
        capture: &mut Capture,
    ) -> Result<(), CaptureError> {
        let base_path = &self.config.storage_path;

        // Create directory structure: captures/target_id/YYYY-MM-DD/
        let date_str = result.captured_at.format("%Y-%m-%d").to_string();
        let capture_dir = base_path
            .join(target.id.to_string())
            .join(&date_str);

        tokio::fs::create_dir_all(&capture_dir)
            .await
            .map_err(|e| CaptureError::ScreenshotFailed(format!("Directory error: {}", e)))?;

        let timestamp = result.captured_at.format("%H-%M-%S").to_string();

        // Save screenshot
        if let Some(screenshot) = &result.screenshot_data {
            let filename = format!("{}.png", timestamp);
            let path = capture_dir.join(&filename);

            tokio::fs::write(&path, screenshot)
                .await
                .map_err(|e| CaptureError::ScreenshotFailed(format!("Write error: {}", e)))?;

            capture.screenshot_path = Some(path.to_string_lossy().to_string());
            capture.screenshot_size_bytes = Some(screenshot.len() as i64);
        }

        // Save HTML
        if let Some(html) = &result.html_content {
            let filename = format!("{}.html", timestamp);
            let path = capture_dir.join(&filename);

            tokio::fs::write(&path, html)
                .await
                .map_err(|e| CaptureError::HtmlCaptureFailed(format!("Write error: {}", e)))?;

            capture.html_path = Some(path.to_string_lossy().to_string());
            capture.html_size_bytes = Some(html.len() as i64);
        }

        // Update capture metadata
        capture.final_url = Some(result.final_url.clone());
        capture.page_title = result.page_title.clone();
        capture.http_status_code = result.http_status_code;

        Ok(())
    }

    /// Get user agent string for target.
    fn get_user_agent(&self, target: &Target) -> String {
        match &target.user_agent_type {
            UserAgentType::Custom => target
                .custom_user_agent
                .clone()
                .unwrap_or_else(|| IotUserAgents::random().to_string()),
            UserAgentType::Shodan => IotUserAgents::shodan()[0].to_string(),
            UserAgentType::Censys => IotUserAgents::censys()[0].to_string(),
            UserAgentType::ZGrab => IotUserAgents::zgrab()[0].to_string(),
            UserAgentType::Masscan => IotUserAgents::masscan()[0].to_string(),
            UserAgentType::Nmap => IotUserAgents::nmap()[0].to_string(),
            UserAgentType::BinaryEdge => IotUserAgents::binaryedge()[0].to_string(),
            UserAgentType::Fofa => IotUserAgents::fofa()[0].to_string(),
            UserAgentType::ZoomEye => IotUserAgents::zoomeye()[0].to_string(),
            UserAgentType::GreyNoise => IotUserAgents::greynoise()[0].to_string(),
            UserAgentType::Random => IotUserAgents::random().to_string(),
        }
    }

    /// Get the storage path.
    pub fn storage_path(&self) -> &Path {
        &self.config.storage_path
    }
}

/// Generate a unique capture filename.
pub fn generate_capture_filename(target_id: &Uuid, extension: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    format!("{}_{}.{}", target_id, timestamp, extension)
}
