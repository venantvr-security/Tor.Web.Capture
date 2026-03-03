//! Google Drive integration for uploading captures.

pub mod auth;
mod upload;
mod folder;

pub use auth::{AuthProvider, OAuth2Flow, ServiceAccountAuth};
pub use upload::*;
pub use folder::*;

/// Google Drive scopes required.
pub const DRIVE_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/drive.file",
    "https://www.googleapis.com/auth/drive.metadata.readonly",
];

/// Check if Google Drive is configured and available.
pub async fn is_configured(auth: &dyn AuthProvider) -> bool {
    auth.get_token().await.is_ok()
}
