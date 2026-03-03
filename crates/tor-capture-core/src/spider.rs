//! Spider configuration and state management with JSON persistence.

use crate::{SpiderConfig, SpiderState, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// All spider configurations and states stored in JSON file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpiderConfigFile {
    /// Spider configuration per target (target_id -> config)
    pub configs: HashMap<String, SpiderConfig>,
    /// Spider state per target (target_id -> state)
    pub states: HashMap<String, SpiderState>,
}

impl SpiderConfigFile {
    /// Load from JSON file or create default if file doesn't exist.
    pub fn load(path: &PathBuf) -> Result<Self, StorageError> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| StorageError::FileSystemError(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| StorageError::FileSystemError(format!("JSON parse error: {}", e)))
        } else {
            Ok(Self::default())
        }
    }

    /// Save to JSON file (creates parent directories if needed).
    pub fn save(&self, path: &PathBuf) -> Result<(), StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError::FileSystemError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| StorageError::FileSystemError(format!("JSON serialize error: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| StorageError::FileSystemError(e.to_string()))
    }

    /// Get spider config for a target.
    pub fn get_config(&self, target_id: &Uuid) -> Option<&SpiderConfig> {
        self.configs.get(&target_id.to_string())
    }

    /// Set spider config for a target.
    pub fn set_config(&mut self, target_id: &Uuid, config: SpiderConfig) {
        self.configs.insert(target_id.to_string(), config);
    }

    /// Remove spider config for a target.
    pub fn remove_config(&mut self, target_id: &Uuid) {
        self.configs.remove(&target_id.to_string());
    }

    /// Get spider state for a target.
    pub fn get_state(&self, target_id: &Uuid) -> Option<&SpiderState> {
        self.states.get(&target_id.to_string())
    }

    /// Get mutable spider state for a target.
    pub fn get_state_mut(&mut self, target_id: &Uuid) -> Option<&mut SpiderState> {
        self.states.get_mut(&target_id.to_string())
    }

    /// Set spider state for a target.
    pub fn set_state(&mut self, target_id: &Uuid, state: SpiderState) {
        self.states.insert(target_id.to_string(), state);
    }

    /// Remove spider state for a target.
    pub fn remove_state(&mut self, target_id: &Uuid) {
        self.states.remove(&target_id.to_string());
    }

    /// Check if spider is enabled for a target.
    pub fn is_enabled(&self, target_id: &Uuid) -> bool {
        self.get_config(target_id)
            .map(|c| c.enabled)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpiderStatus;
    use tempfile::tempdir;

    #[test]
    fn test_spider_config_file_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("spider_config.json");

        let mut file = SpiderConfigFile::default();
        let target_id = Uuid::new_v4();

        // Add config
        file.set_config(
            &target_id,
            SpiderConfig {
                enabled: true,
                max_depth: 3,
                same_domain_only: true,
                max_urls: 50,
                delay_ms: 500,
            },
        );

        // Add state
        let mut state = SpiderState::new(target_id);
        state.visited_urls.push("https://example.com".to_string());
        state.status = SpiderStatus::Running;
        file.set_state(&target_id, state);

        // Save
        file.save(&path).unwrap();

        // Load
        let loaded = SpiderConfigFile::load(&path).unwrap();

        assert!(loaded.is_enabled(&target_id));
        assert_eq!(loaded.get_config(&target_id).unwrap().max_depth, 3);
        assert_eq!(
            loaded.get_state(&target_id).unwrap().status,
            SpiderStatus::Running
        );
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/path/spider.json");
        let file = SpiderConfigFile::load(&path).unwrap();
        assert!(file.configs.is_empty());
        assert!(file.states.is_empty());
    }
}
