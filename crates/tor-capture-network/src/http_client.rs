//! HTTP client that routes all traffic through TOR.

use reqwest::{Client, Response};
use std::time::Duration;
use tor_capture_core::TorError;

/// HTTP client configured to use TOR SOCKS5 proxy.
pub struct TorHttpClient {
    client: Client,
    socks_addr: String,
}

impl TorHttpClient {
    /// Create a new HTTP client using the TOR SOCKS5 proxy.
    pub fn new(socks_addr: &str, timeout_ms: u64) -> Result<Self, TorError> {
        let proxy = reqwest::Proxy::all(format!("socks5://{}", socks_addr))
            .map_err(|e| TorError::ConnectionFailed(format!("Proxy config error: {}", e)))?;

        let client = Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_millis(timeout_ms))
            // Disable connection pooling for better anonymity
            .pool_max_idle_per_host(0)
            // Use rustls for TLS (no OpenSSL dependency)
            .use_rustls_tls()
            // Don't follow redirects automatically (we want to track them)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| TorError::ConnectionFailed(format!("Client build error: {}", e)))?;

        Ok(Self {
            client,
            socks_addr: socks_addr.to_string(),
        })
    }

    /// Make a GET request with custom user agent.
    pub async fn get(&self, url: &str, user_agent: &str) -> Result<Response, TorError> {
        self.client
            .get(url)
            .header("User-Agent", user_agent)
            .send()
            .await
            .map_err(|e| TorError::ConnectionFailed(format!("Request failed: {}", e)))
    }

    /// Make a GET request and follow redirects manually.
    pub async fn get_with_redirects(
        &self,
        url: &str,
        user_agent: &str,
        max_redirects: usize,
    ) -> Result<(Response, String), TorError> {
        let mut current_url = url.to_string();
        let mut redirects = 0;

        loop {
            let response = self.get(&current_url, user_agent).await?;

            if response.status().is_redirection() && redirects < max_redirects {
                if let Some(location) = response.headers().get("location") {
                    if let Ok(loc_str) = location.to_str() {
                        // Handle relative URLs
                        current_url = if loc_str.starts_with("http") {
                            loc_str.to_string()
                        } else if loc_str.starts_with('/') {
                            let url_parsed =
                                url::Url::parse(&current_url).map_err(|e| {
                                    TorError::ConnectionFailed(format!("URL parse error: {}", e))
                                })?;
                            format!(
                                "{}://{}{}",
                                url_parsed.scheme(),
                                url_parsed.host_str().unwrap_or(""),
                                loc_str
                            )
                        } else {
                            loc_str.to_string()
                        };
                        redirects += 1;
                        continue;
                    }
                }
            }

            return Ok((response, current_url));
        }
    }

    /// Get the SOCKS address being used.
    pub fn socks_addr(&self) -> &str {
        &self.socks_addr
    }

    /// Fetch page content as string.
    pub async fn fetch_text(&self, url: &str, user_agent: &str) -> Result<String, TorError> {
        let response = self.get(url, user_agent).await?;
        response
            .text()
            .await
            .map_err(|e| TorError::ConnectionFailed(format!("Failed to read response: {}", e)))
    }

    /// Fetch page content as bytes.
    pub async fn fetch_bytes(&self, url: &str, user_agent: &str) -> Result<Vec<u8>, TorError> {
        let response = self.get(url, user_agent).await?;
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| TorError::ConnectionFailed(format!("Failed to read response: {}", e)))
    }
}

/// Create a new TOR HTTP client with default settings.
pub fn create_tor_http_client(socks_addr: &str) -> Result<TorHttpClient, TorError> {
    TorHttpClient::new(socks_addr, 30000)
}
