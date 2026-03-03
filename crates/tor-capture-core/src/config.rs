//! Application configuration types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub web: WebConfig,
    pub tor: TorConfig,
    pub capture: CaptureConfig,
    pub storage: StorageConfig,
    pub gdrive: GDriveSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            web: WebConfig::default(),
            tor: TorConfig::default(),
            capture: CaptureConfig::default(),
            storage: StorageConfig::default(),
            gdrive: GDriveSettings::default(),
        }
    }
}

/// Web server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub bind_address: String,
    pub port: u16,
    pub static_files_path: PathBuf,
    pub templates_path: PathBuf,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            static_files_path: PathBuf::from("./static"),
            templates_path: PathBuf::from("./templates"),
        }
    }
}

/// TOR configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConfig {
    pub enabled: bool,
    pub data_dir: PathBuf,
    pub new_circuit_per_capture: bool,
    pub bootstrap_timeout_secs: u64,
    pub socks_port: u16,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            data_dir: PathBuf::from("./data/tor"),
            new_circuit_per_capture: true,
            bootstrap_timeout_secs: 120,
            socks_port: 9150,
        }
    }
}

/// Capture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub storage_path: PathBuf,
    pub max_concurrent_captures: usize,
    pub default_timeout_ms: u64,
    pub chrome_executable_path: Option<PathBuf>,
    pub default_viewport_width: u32,
    pub default_viewport_height: u32,
    pub default_wait_after_load_ms: u64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./data/captures"),
            max_concurrent_captures: 3,
            default_timeout_ms: 30000,
            chrome_executable_path: None,
            default_viewport_width: 1920,
            default_viewport_height: 1080,
            default_wait_after_load_ms: 2000,
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: PathBuf,
    pub pool_size: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("./data/tor-capture.db"),
            pool_size: 5,
        }
    }
}

/// Google Drive settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GDriveSettings {
    pub enabled: bool,
    pub auto_upload: bool,
    pub upload_screenshots: bool,
    pub upload_html: bool,
}

impl Default for GDriveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_upload: false,
            upload_screenshots: true,
            upload_html: true,
        }
    }
}

/// TOR isolation configuration for security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorIsolationConfig {
    /// Force ALL DNS requests via TOR (no local resolution)
    pub force_tor_dns: bool,
    /// New circuit for each different domain
    pub isolate_by_destination: bool,
    /// New circuit for each capture
    pub isolate_by_capture: bool,
    /// Block direct connections (failsafe)
    pub block_direct_connections: bool,
}

impl Default for TorIsolationConfig {
    fn default() -> Self {
        Self {
            force_tor_dns: true,
            isolate_by_destination: true,
            isolate_by_capture: true,
            block_direct_connections: true,
        }
    }
}

/// IoT bot user agents collection.
pub struct IotUserAgents;

impl IotUserAgents {
    pub fn shodan() -> &'static [&'static str] {
        &[
            "Shodan",
            "Mozilla/5.0 (compatible; Shodan; +https://www.shodan.io)",
        ]
    }

    pub fn censys() -> &'static [&'static str] {
        &[
            "Mozilla/5.0 (compatible; CensysInspect/1.1; +https://about.censys.io/)",
            "censys/1.0",
        ]
    }

    pub fn zgrab() -> &'static [&'static str] {
        &["Mozilla/5.0 zgrab/0.x", "zgrab2/0.1"]
    }

    pub fn masscan() -> &'static [&'static str] {
        &["masscan/1.3 (https://github.com/robertdavidgraham/masscan)"]
    }

    pub fn nmap() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; Nmap Scripting Engine; https://nmap.org/book/nse.html)"]
    }

    pub fn binaryedge() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; BinaryEdge; +https://www.binaryedge.io)"]
    }

    pub fn fofa() -> &'static [&'static str] {
        &["FOFA"]
    }

    pub fn zoomeye() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; ZoomEye; +https://www.zoomeye.org)"]
    }

    pub fn greynoise() -> &'static [&'static str] {
        &["GreyNoise/1.0 (greynoise.io)"]
    }

    pub fn shadowserver() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; Shadowserver; +https://shadowserver.org)"]
    }

    pub fn securitytrails() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; SecurityTrails; +https://securitytrails.com)"]
    }

    pub fn onyphe() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; ONYPHE; +https://www.onyphe.io)"]
    }

    pub fn ipinfo() -> &'static [&'static str] {
        &["Mozilla/5.0 (compatible; IPinfo; +https://ipinfo.io)"]
    }

    pub fn all() -> Vec<&'static str> {
        let mut all = Vec::new();
        all.extend(Self::shodan());
        all.extend(Self::censys());
        all.extend(Self::zgrab());
        all.extend(Self::masscan());
        all.extend(Self::nmap());
        all.extend(Self::binaryedge());
        all.extend(Self::fofa());
        all.extend(Self::zoomeye());
        all.extend(Self::greynoise());
        all.extend(Self::shadowserver());
        all.extend(Self::securitytrails());
        all.extend(Self::onyphe());
        all.extend(Self::ipinfo());
        all
    }

    pub fn random() -> &'static str {
        use std::time::{SystemTime, UNIX_EPOCH};
        let all = Self::all();
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        all[seed % all.len()]
    }
}
