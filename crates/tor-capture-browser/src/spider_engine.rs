//! Spider engine for recursive web crawling.

use crate::{extract_links, filter_same_domain, CaptureEngine};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tor_capture_core::{
    CaptureError, CaptureResult, SpiderConfig, SpiderConfigFile, SpiderState, SpiderStatus,
    SpiderUrl, Target,
};
use tracing::{info, warn};
use uuid::Uuid;

/// Spider engine that orchestrates recursive crawling.
pub struct SpiderEngine {
    capture_engine: Arc<CaptureEngine>,
    config_path: PathBuf,
    config_file: Arc<RwLock<SpiderConfigFile>>,
}

impl SpiderEngine {
    /// Create a new spider engine.
    pub fn new(capture_engine: Arc<CaptureEngine>, data_dir: PathBuf) -> Self {
        let config_path = data_dir.join("spider_config.json");
        let config_file = SpiderConfigFile::load(&config_path).unwrap_or_default();

        Self {
            capture_engine,
            config_path,
            config_file: Arc::new(RwLock::new(config_file)),
        }
    }

    /// Execute spider crawl for a target.
    ///
    /// Crawls pages starting from the target URL, following links up to max_depth
    /// and max_urls limits. Calls the callback for each captured page.
    pub async fn execute_spider<F>(
        &self,
        target: &Target,
        config: &SpiderConfig,
        mut on_capture: F,
    ) -> Result<SpiderState, CaptureError>
    where
        F: FnMut(CaptureResult, &str, u32),
    {
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: Vec<SpiderUrl> = vec![SpiderUrl {
            url: target.url.clone(),
            depth: 0,
            discovered_from: None,
        }];

        let mut state = SpiderState::new(target.id);
        state.status = SpiderStatus::Running;

        info!(
            "Starting spider crawl for target {} (max_depth: {}, max_urls: {})",
            target.id, config.max_depth, config.max_urls
        );

        while let Some(spider_url) = queue.pop() {
            // Check limits
            if visited.len() >= config.max_urls {
                info!("Reached max URLs limit: {}", config.max_urls);
                break;
            }

            if spider_url.depth > config.max_depth {
                continue;
            }

            // Skip if already visited
            if visited.contains(&spider_url.url) {
                continue;
            }

            visited.insert(spider_url.url.clone());

            info!(
                "Spider capturing: {} (depth: {}, visited: {})",
                spider_url.url,
                spider_url.depth,
                visited.len()
            );

            // Create modified target for this URL
            let mut url_target = target.clone();
            url_target.url = spider_url.url.clone();

            // Execute capture
            match self.capture_engine.execute_capture(&url_target).await {
                Ok(result) => {
                    // Call the callback with the capture result
                    on_capture(result.clone(), &spider_url.url, spider_url.depth);
                    state.total_captured += 1;

                    // Extract and queue new links if we have HTML and haven't reached max depth
                    if spider_url.depth < config.max_depth {
                        if let Some(html) = &result.html_content {
                            let mut links = extract_links(html, &spider_url.url);

                            if config.same_domain_only {
                                links = filter_same_domain(links, &target.url);
                            }

                            let new_links: Vec<_> = links
                                .into_iter()
                                .filter(|link| !visited.contains(link))
                                .collect();

                            info!(
                                "Found {} new links at depth {}",
                                new_links.len(),
                                spider_url.depth
                            );

                            for link in new_links {
                                queue.push(SpiderUrl {
                                    url: link,
                                    depth: spider_url.depth + 1,
                                    discovered_from: Some(spider_url.url.clone()),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Spider capture failed for {}: {}", spider_url.url, e);
                }
            }

            state.last_activity_at = chrono::Utc::now();

            // Delay between requests
            if config.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(config.delay_ms)).await;
            }
        }

        state.visited_urls = visited.into_iter().collect();
        state.pending_urls = queue;
        state.status = if state.pending_urls.is_empty() {
            SpiderStatus::Completed
        } else {
            SpiderStatus::Paused
        };

        info!(
            "Spider completed for target {}: {} URLs captured",
            target.id, state.total_captured
        );

        // Save state
        self.save_state(&target.id, state.clone()).await?;

        Ok(state)
    }

    /// Save spider state to config file.
    async fn save_state(
        &self,
        target_id: &Uuid,
        state: SpiderState,
    ) -> Result<(), CaptureError> {
        let mut config_file = self.config_file.write().await;
        config_file.set_state(target_id, state);
        config_file
            .save(&self.config_path)
            .map_err(|e| CaptureError::ScreenshotFailed(format!("Failed to save spider state: {}", e)))
    }

    /// Get spider config for a target.
    pub async fn get_config(&self, target_id: &Uuid) -> Option<SpiderConfig> {
        self.config_file.read().await.get_config(target_id).cloned()
    }

    /// Set spider config for a target and persist to JSON.
    pub async fn set_config(
        &self,
        target_id: &Uuid,
        config: SpiderConfig,
    ) -> Result<(), CaptureError> {
        let mut config_file = self.config_file.write().await;
        config_file.set_config(target_id, config);
        config_file
            .save(&self.config_path)
            .map_err(|e| CaptureError::ScreenshotFailed(format!("Failed to save spider config: {}", e)))
    }

    /// Remove spider config for a target.
    pub async fn remove_config(&self, target_id: &Uuid) -> Result<(), CaptureError> {
        let mut config_file = self.config_file.write().await;
        config_file.remove_config(target_id);
        config_file.remove_state(target_id);
        config_file
            .save(&self.config_path)
            .map_err(|e| CaptureError::ScreenshotFailed(format!("Failed to save spider config: {}", e)))
    }

    /// Get spider state for a target.
    pub async fn get_state(&self, target_id: &Uuid) -> Option<SpiderState> {
        self.config_file.read().await.get_state(target_id).cloned()
    }

    /// Check if spider is enabled for a target.
    pub async fn is_enabled(&self, target_id: &Uuid) -> bool {
        self.config_file.read().await.is_enabled(target_id)
    }

    /// Reload config from disk.
    pub async fn reload(&self) -> Result<(), CaptureError> {
        let new_config = SpiderConfigFile::load(&self.config_path)
            .map_err(|e| CaptureError::ScreenshotFailed(format!("Failed to reload spider config: {}", e)))?;

        let mut config_file = self.config_file.write().await;
        *config_file = new_config;
        Ok(())
    }
}
