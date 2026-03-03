//! Authentication providers for Google Drive.

mod oauth2;
mod service_account;

pub use oauth2::*;
pub use service_account::*;

use async_trait::async_trait;
use tor_capture_core::GDriveError;

/// Token for Google Drive API.
#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => chrono::Utc::now() >= expires,
            None => false,
        }
    }
}

/// Trait for authentication providers.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get a valid access token.
    async fn get_token(&self) -> Result<AccessToken, GDriveError>;

    /// Refresh the token if needed.
    async fn refresh_if_needed(&self) -> Result<AccessToken, GDriveError>;

    /// Check if authentication is configured.
    fn is_configured(&self) -> bool;
}
