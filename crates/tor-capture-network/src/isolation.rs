//! TOR isolation and security configuration.

use tor_capture_core::TorIsolationConfig;

/// Verify that a URL is safe to access via TOR.
pub fn verify_url_safety(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    // Check scheme
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("Unsupported scheme: {}", scheme)),
    }

    // Check for local/private IPs
    if let Some(host) = parsed.host_str() {
        if is_local_or_private(host) {
            return Err(format!("Local/private address not allowed: {}", host));
        }
    }

    Ok(())
}

/// Check if a host is a local or private address.
fn is_local_or_private(host: &str) -> bool {
    // localhost variations
    if host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.ends_with(".localhost")
    {
        return true;
    }

    // Private IPv4 ranges
    if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
        let octets = ip.octets();
        return
            // 10.0.0.0/8
            octets[0] == 10 ||
            // 172.16.0.0/12
            (octets[0] == 172 && (16..=31).contains(&octets[1])) ||
            // 192.168.0.0/16
            (octets[0] == 192 && octets[1] == 168) ||
            // 127.0.0.0/8
            octets[0] == 127 ||
            // 169.254.0.0/16 (link-local)
            (octets[0] == 169 && octets[1] == 254);
    }

    // Private IPv6
    if let Ok(ip) = host.parse::<std::net::Ipv6Addr>() {
        return ip.is_loopback()
            || is_private_ipv6(&ip)
            || is_link_local_ipv6(&ip);
    }

    false
}

fn is_private_ipv6(ip: &std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    // fc00::/7 (unique local)
    (segments[0] & 0xfe00) == 0xfc00
}

fn is_link_local_ipv6(ip: &std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    // fe80::/10
    (segments[0] & 0xffc0) == 0xfe80
}

/// Generate Chrome command line arguments for TOR proxy.
pub fn chrome_tor_args(socks_addr: &str, config: &TorIsolationConfig) -> Vec<String> {
    let mut args = vec![
        // SOCKS5 proxy
        format!("--proxy-server=socks5://{}", socks_addr),
    ];

    if config.force_tor_dns {
        // Force all DNS through the proxy
        args.push("--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost".to_string());
    }

    // Security hardening
    args.extend(vec![
        // Disable WebRTC (IP leak prevention)
        "--disable-webrtc".to_string(),
        "--disable-features=WebRTC".to_string(),
        // Disable geolocation
        "--disable-features=Geolocation".to_string(),
        // Incognito mode
        "--incognito".to_string(),
        // Disable cache
        "--disable-application-cache".to_string(),
        "--aggressive-cache-discard".to_string(),
        // Disable extensions
        "--disable-extensions".to_string(),
        // Disable GPU (not needed for screenshots)
        "--disable-gpu".to_string(),
        "--disable-software-rasterizer".to_string(),
        // Headless mode
        "--headless=new".to_string(),
        // Sandbox settings (may need --no-sandbox in containers)
        "--disable-setuid-sandbox".to_string(),
        // Disable background networking
        "--disable-background-networking".to_string(),
        "--disable-sync".to_string(),
        "--disable-translate".to_string(),
        "--disable-default-apps".to_string(),
        // Disable features that might leak
        "--disable-client-side-phishing-detection".to_string(),
        "--disable-hang-monitor".to_string(),
        "--disable-popup-blocking".to_string(),
        "--disable-prompt-on-repost".to_string(),
        // Deterministic rendering
        "--deterministic-mode".to_string(),
        "--disable-threaded-scrolling".to_string(),
        "--disable-threaded-animation".to_string(),
    ]);

    args
}

/// Pre-capture security check.
pub struct SecurityCheck {
    pub url_safe: bool,
    pub tor_connected: bool,
    pub exit_ip: Option<String>,
    pub warnings: Vec<String>,
}

impl SecurityCheck {
    pub fn is_ok(&self) -> bool {
        self.url_safe && self.tor_connected
    }
}

/// Perform pre-capture security verification.
pub async fn pre_capture_check(url: &str, socks_addr: &str) -> SecurityCheck {
    let mut check = SecurityCheck {
        url_safe: false,
        tor_connected: false,
        exit_ip: None,
        warnings: Vec::new(),
    };

    // Verify URL safety
    match verify_url_safety(url) {
        Ok(()) => check.url_safe = true,
        Err(e) => check.warnings.push(format!("URL safety: {}", e)),
    }

    // Check TOR connectivity
    check.tor_connected = super::tor_client::check_tor_connectivity(socks_addr).await;
    if !check.tor_connected {
        check.warnings.push("TOR not connected".to_string());
    }

    // Get exit IP
    check.exit_ip = super::tor_client::get_exit_ip(socks_addr).await;

    check
}
