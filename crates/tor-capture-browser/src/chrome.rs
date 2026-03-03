//! Chrome browser configuration and launcher.

use headless_chrome::{Browser, LaunchOptions, Tab};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tor_capture_core::CaptureError;

/// Chrome browser wrapper configured for TOR.
pub struct ChromeBrowser {
    browser: Browser,
}

impl ChromeBrowser {
    /// Launch Chrome with TOR proxy configuration.
    pub fn new(
        socks_addr: &str,
        chrome_path: Option<&PathBuf>,
        sandbox: bool,
    ) -> Result<Self, CaptureError> {
        let mut args = build_chrome_args(socks_addr);

        // Add no-sandbox if requested (needed in containers)
        if !sandbox {
            args.push("--no-sandbox".to_string());
        }

        let args_refs: Vec<&OsStr> = args.iter().map(|s| OsStr::new(s)).collect();

        let options = LaunchOptions::default_builder()
            .args(args_refs)
            .idle_browser_timeout(Duration::from_secs(300))
            .enable_logging(false)
            .path(chrome_path.cloned())
            .build()
            .map_err(|e| CaptureError::BrowserLaunchFailed(format!("Options error: {}", e)))?;

        let browser = Browser::new(options)
            .map_err(|e| CaptureError::BrowserLaunchFailed(format!("Launch error: {}", e)))?;

        Ok(Self { browser })
    }

    /// Get a reference to the browser.
    pub fn inner(&self) -> &Browser {
        &self.browser
    }

    /// Create a new tab.
    pub fn new_tab(&self) -> Result<Arc<Tab>, CaptureError> {
        self.browser
            .new_tab()
            .map_err(|e| CaptureError::BrowserLaunchFailed(format!("Tab error: {}", e)))
    }
}

/// Build Chrome command line arguments for TOR proxy.
fn build_chrome_args(socks_addr: &str) -> Vec<String> {
    vec![
        // SOCKS5 proxy via TOR
        format!("--proxy-server=socks5://{}", socks_addr),
        // Force DNS through proxy (CRITICAL for anonymity)
        "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost".to_string(),
        // Disable WebRTC (prevents IP leaks)
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
        // Disable GPU (not needed)
        "--disable-gpu".to_string(),
        "--disable-software-rasterizer".to_string(),
        // Headless mode
        "--headless=new".to_string(),
        // Sandbox settings
        "--disable-setuid-sandbox".to_string(),
        // Disable background networking
        "--disable-background-networking".to_string(),
        "--disable-sync".to_string(),
        "--disable-translate".to_string(),
        "--disable-default-apps".to_string(),
        // Disable client-side phishing detection
        "--disable-client-side-phishing-detection".to_string(),
        // Disable hang monitor
        "--disable-hang-monitor".to_string(),
        // Disable popup blocking
        "--disable-popup-blocking".to_string(),
        // Disable prompt on repost
        "--disable-prompt-on-repost".to_string(),
        // Deterministic rendering for consistent screenshots
        "--deterministic-mode".to_string(),
        "--disable-threaded-scrolling".to_string(),
        "--disable-threaded-animation".to_string(),
        // Disable infobars
        "--disable-infobars".to_string(),
        // Mute audio
        "--mute-audio".to_string(),
        // Disable dev shm usage (for containers)
        "--disable-dev-shm-usage".to_string(),
    ]
}

/// Find Chrome/Chromium executable.
pub fn find_chrome() -> Option<PathBuf> {
    let candidates = [
        // Linux
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/snap/bin/chromium",
        // macOS
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        // Windows
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    // Try to find via which command
    if let Ok(output) = std::process::Command::new("which")
        .arg("chromium")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    if let Ok(output) = std::process::Command::new("which")
        .arg("google-chrome")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}
