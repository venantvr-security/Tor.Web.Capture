//! Screenshot capture functionality.

use chrono::Utc;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::Tab;
use std::sync::Arc;
use std::time::Duration;
use tor_capture_core::{CaptureError, CaptureResult};

/// Viewport configuration.
#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

/// Screenshot options.
#[derive(Debug, Clone)]
pub struct ScreenshotOptions {
    pub viewport: Viewport,
    pub full_page: bool,
    pub format: ScreenshotFormat,
    pub quality: Option<u32>,
    pub wait_after_load_ms: u64,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            full_page: true,
            format: ScreenshotFormat::Png,
            quality: None,
            wait_after_load_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenshotFormat {
    Png,
    Jpeg,
    Webp,
}

impl ScreenshotFormat {
    fn to_cdp(&self) -> CaptureScreenshotFormatOption {
        match self {
            Self::Png => CaptureScreenshotFormatOption::Png,
            Self::Jpeg => CaptureScreenshotFormatOption::Jpeg,
            Self::Webp => CaptureScreenshotFormatOption::Webp,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }
}

/// Take a screenshot of a page.
pub fn take_screenshot(
    tab: &Arc<Tab>,
    options: &ScreenshotOptions,
) -> Result<Vec<u8>, CaptureError> {
    // Set viewport size via emulation
    tab.call_method(headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
        width: options.viewport.width,
        height: options.viewport.height,
        device_scale_factor: 1.0,
        mobile: false,
        scale: None,
        screen_width: None,
        screen_height: None,
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        screen_orientation: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    })
    .map_err(|e| CaptureError::ScreenshotFailed(format!("Viewport error: {}", e)))?;

    // Wait for any dynamic content
    std::thread::sleep(Duration::from_millis(options.wait_after_load_ms));

    // Capture screenshot
    let screenshot = tab
        .capture_screenshot(options.format.to_cdp(), options.quality, None, options.full_page)
        .map_err(|e| CaptureError::ScreenshotFailed(format!("Capture error: {}", e)))?;

    Ok(screenshot)
}

/// Capture a page (screenshot + HTML).
pub fn capture_page(
    tab: &Arc<Tab>,
    url: &str,
    user_agent: &str,
    options: &ScreenshotOptions,
    capture_screenshot: bool,
    capture_html: bool,
) -> Result<CaptureResult, CaptureError> {
    // Set user agent
    tab.set_user_agent(user_agent, None, None)
        .map_err(|e| CaptureError::NavigationFailed(format!("User agent error: {}", e)))?;

    // Navigate to URL
    tab.navigate_to(url)
        .map_err(|e| CaptureError::NavigationFailed(format!("Navigation error: {}", e)))?;

    // Wait for page to load
    tab.wait_until_navigated()
        .map_err(|e| CaptureError::NavigationFailed(format!("Wait error: {}", e)))?;

    // Additional wait for dynamic content
    std::thread::sleep(Duration::from_millis(options.wait_after_load_ms));

    // Get final URL (after redirects)
    let final_url = tab.get_url();

    // Get page title
    let title = tab.get_title().ok();

    // Capture screenshot if requested
    let screenshot_data = if capture_screenshot {
        Some(take_screenshot(tab, options)?)
    } else {
        None
    };

    // Capture HTML if requested
    let html_content = if capture_html {
        Some(
            tab.get_content()
                .map_err(|e| CaptureError::HtmlCaptureFailed(format!("HTML error: {}", e)))?,
        )
    } else {
        None
    };

    Ok(CaptureResult {
        screenshot_data,
        html_content,
        final_url,
        page_title: title,
        http_status_code: None, // Would need to hook into network events
        captured_at: Utc::now(),
    })
}
