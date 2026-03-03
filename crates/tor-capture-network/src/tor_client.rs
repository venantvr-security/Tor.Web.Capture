//! TOR client wrapper using Arti.

use arti_client::{TorClient, TorClientConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tor_capture_core::{TorCircuitInfo, TorConfig, TorError};

/// Isolation token for circuit separation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsolationToken(u64);

impl IsolationToken {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self(nanos)
    }
}

impl Default for IsolationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// TOR network client wrapper.
pub struct TorNetworkClient {
    client: Option<Arc<TorClient<tor_rtcompat::PreferredRuntime>>>,
    current_isolation: RwLock<Option<IsolationToken>>,
    socks_port: u16,
    _bootstrapped: bool,
}

impl TorNetworkClient {
    /// Create and bootstrap a new TOR client.
    pub async fn new(config: &TorConfig) -> Result<Self, TorError> {
        let data_dir = &config.data_dir;

        // Ensure directories exist
        std::fs::create_dir_all(data_dir.join("state"))
            .map_err(|e| TorError::ConnectionFailed(e.to_string()))?;
        std::fs::create_dir_all(data_dir.join("cache"))
            .map_err(|e| TorError::ConnectionFailed(e.to_string()))?;

        // Set environment variables for Arti data directories
        std::env::set_var("ARTI_LOCAL_DATA", data_dir.join("state"));
        std::env::set_var("ARTI_CACHE", data_dir.join("cache"));

        let tor_config = TorClientConfig::default();

        tracing::info!("Bootstrapping TOR client...");

        let client = TorClient::create_bootstrapped(tor_config)
            .await
            .map_err(|e| TorError::ConnectionFailed(format!("Bootstrap failed: {}", e)))?;

        tracing::info!("TOR client bootstrapped successfully");

        Ok(Self {
            client: Some(Arc::new(client)),
            current_isolation: RwLock::new(None),
            socks_port: config.socks_port,
            _bootstrapped: true,
        })
    }

    /// Create a mock client for testing without TOR.
    pub fn new_mock(socks_port: u16) -> Self {
        // This creates a placeholder - actual TOR connection won't work
        // Used only for testing the application without TOR
        Self {
            client: None,
            current_isolation: RwLock::new(None),
            socks_port,
            _bootstrapped: false,
        }
    }

    /// Get a new isolation token for circuit separation.
    pub async fn new_circuit(&self) -> IsolationToken {
        let token = IsolationToken::new();
        *self.current_isolation.write().await = Some(token.clone());
        token
    }

    /// Get the SOCKS5 proxy address.
    pub fn socks_addr(&self) -> String {
        format!("127.0.0.1:{}", self.socks_port)
    }

    /// Check if TOR is ready.
    pub fn is_ready(&self) -> bool {
        self._bootstrapped
    }

    /// Get current circuit information.
    pub async fn get_circuit_info(&self) -> Result<TorCircuitInfo, TorError> {
        // In a real implementation, we would query Arti for circuit details
        // For now, return a placeholder
        let isolation = self.current_isolation.read().await;

        Ok(TorCircuitInfo {
            circuit_id: isolation
                .as_ref()
                .map(|t| t.0.to_string())
                .unwrap_or_else(|| "default".to_string()),
            exit_node_ip: None, // Would need to be fetched via control port
            exit_node_country: None,
            is_ready: self._bootstrapped,
        })
    }

    /// Get the inner Arti client for direct usage.
    pub fn inner(&self) -> Option<&Arc<TorClient<tor_rtcompat::PreferredRuntime>>> {
        self.client.as_ref()
    }
}

/// Check if TOR is available by testing connectivity.
pub async fn check_tor_connectivity(socks_addr: &str) -> bool {
    // Try to connect to check.torproject.org via the SOCKS proxy
    let proxy = reqwest::Proxy::all(format!("socks5://{}", socks_addr));

    if proxy.is_err() {
        return false;
    }

    let client = reqwest::Client::builder()
        .proxy(proxy.unwrap())
        .timeout(std::time::Duration::from_secs(30))
        .build();

    if client.is_err() {
        return false;
    }

    match client
        .unwrap()
        .get("https://check.torproject.org/api/ip")
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Get the exit node IP via TOR.
pub async fn get_exit_ip(socks_addr: &str) -> Option<String> {
    let proxy = reqwest::Proxy::all(format!("socks5://{}", socks_addr)).ok()?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .ok()?;

    let response = client
        .get("https://check.torproject.org/api/ip")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = response.json().await.ok()?;
    json.get("IP")?.as_str().map(|s| s.to_string())
}
