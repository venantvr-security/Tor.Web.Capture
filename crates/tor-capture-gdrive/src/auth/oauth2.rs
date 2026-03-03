//! OAuth2 authentication flow.

use super::{AccessToken, AuthProvider};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tor_capture_core::GDriveError;

/// OAuth2 credentials.
#[derive(Debug, Clone)]
pub struct OAuth2Credentials {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
}

/// OAuth2 authentication flow.
pub struct OAuth2Flow {
    credentials: OAuth2Credentials,
    current_token: Arc<RwLock<Option<AccessToken>>>,
    redirect_uri: String,
}

impl OAuth2Flow {
    /// Create a new OAuth2 flow.
    pub fn new(credentials: OAuth2Credentials, redirect_uri: &str) -> Self {
        Self {
            credentials,
            current_token: Arc::new(RwLock::new(None)),
            redirect_uri: redirect_uri.to_string(),
        }
    }

    /// Get the authorization URL for user consent.
    pub fn get_auth_url(&self) -> String {
        let scopes = super::super::DRIVE_SCOPES.join(" ");
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
            client_id={}&\
            redirect_uri={}&\
            response_type=code&\
            scope={}&\
            access_type=offline&\
            prompt=consent",
            urlencoding::encode(&self.credentials.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
        )
    }

    /// Exchange authorization code for tokens.
    pub async fn exchange_code(&self, code: &str) -> Result<String, GDriveError> {
        let client = reqwest::Client::new();

        let params = [
            ("client_id", self.credentials.client_id.as_str()),
            ("client_secret", self.credentials.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| GDriveError::AuthenticationFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GDriveError::AuthenticationFailed(error_text));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GDriveError::AuthenticationFailed(e.to_string()))?;

        // Store access token
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| GDriveError::AuthenticationFailed("No access token".to_string()))?;

        let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

        let token = AccessToken {
            token: access_token.to_string(),
            expires_at: Some(Utc::now() + Duration::seconds(expires_in)),
        };

        *self.current_token.write().await = Some(token);

        // Return refresh token
        json["refresh_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| GDriveError::AuthenticationFailed("No refresh token".to_string()))
    }

    /// Refresh the access token using refresh token.
    async fn refresh_token(&self) -> Result<AccessToken, GDriveError> {
        let refresh_token = self
            .credentials
            .refresh_token
            .as_ref()
            .ok_or(GDriveError::TokenExpired)?;

        let client = reqwest::Client::new();

        let params = [
            ("client_id", self.credentials.client_id.as_str()),
            ("client_secret", self.credentials.client_secret.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ];

        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| GDriveError::AuthenticationFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GDriveError::AuthenticationFailed(error_text));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GDriveError::AuthenticationFailed(e.to_string()))?;

        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| GDriveError::AuthenticationFailed("No access token".to_string()))?;

        let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

        let token = AccessToken {
            token: access_token.to_string(),
            expires_at: Some(Utc::now() + Duration::seconds(expires_in)),
        };

        *self.current_token.write().await = Some(token.clone());

        Ok(token)
    }

    /// Set the refresh token (after initial authorization).
    pub fn set_refresh_token(&mut self, refresh_token: String) {
        self.credentials.refresh_token = Some(refresh_token);
    }
}

#[async_trait]
impl AuthProvider for OAuth2Flow {
    async fn get_token(&self) -> Result<AccessToken, GDriveError> {
        let token = self.current_token.read().await;
        if let Some(t) = token.as_ref() {
            if !t.is_expired() {
                return Ok(t.clone());
            }
        }
        drop(token);

        self.refresh_token().await
    }

    async fn refresh_if_needed(&self) -> Result<AccessToken, GDriveError> {
        let token = self.current_token.read().await;
        if let Some(t) = token.as_ref() {
            // Refresh if expiring in less than 5 minutes
            let threshold = Utc::now() + Duration::minutes(5);
            if t.expires_at.map(|e| e > threshold).unwrap_or(true) {
                return Ok(t.clone());
            }
        }
        drop(token);

        self.refresh_token().await
    }

    fn is_configured(&self) -> bool {
        self.credentials.refresh_token.is_some()
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
