//! Service Account authentication.

use super::{AccessToken, AuthProvider};
use async_trait::async_trait;
use base64::{engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD}, Engine};
use chrono::{Duration, Utc};
use ring::signature::{RsaKeyPair, RSA_PKCS1_SHA256};
use std::sync::Arc;
use tokio::sync::RwLock;
use tor_capture_core::GDriveError;

/// Service Account credentials.
#[derive(Debug, Clone)]
pub struct ServiceAccountCredentials {
    pub client_email: String,
    pub private_key: String,
    pub token_uri: String,
}

impl ServiceAccountCredentials {
    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, GDriveError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| GDriveError::AuthenticationFailed(format!("Invalid JSON: {}", e)))?;

        let client_email = value["client_email"]
            .as_str()
            .ok_or_else(|| GDriveError::AuthenticationFailed("Missing client_email".to_string()))?
            .to_string();

        let private_key = value["private_key"]
            .as_str()
            .ok_or_else(|| GDriveError::AuthenticationFailed("Missing private_key".to_string()))?
            .to_string();

        let token_uri = value["token_uri"]
            .as_str()
            .unwrap_or("https://oauth2.googleapis.com/token")
            .to_string();

        Ok(Self {
            client_email,
            private_key,
            token_uri,
        })
    }
}

/// Service Account authentication.
pub struct ServiceAccountAuth {
    credentials: ServiceAccountCredentials,
    current_token: Arc<RwLock<Option<AccessToken>>>,
}

impl ServiceAccountAuth {
    /// Create from credentials.
    pub fn new(credentials: ServiceAccountCredentials) -> Self {
        Self {
            credentials,
            current_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from JSON file content.
    pub fn from_json(json: &str) -> Result<Self, GDriveError> {
        let credentials = ServiceAccountCredentials::from_json(json)?;
        Ok(Self::new(credentials))
    }

    /// Create a signed JWT for token request.
    fn create_jwt(&self) -> Result<String, GDriveError> {
        let now = Utc::now().timestamp();
        let exp = now + 3600; // 1 hour

        let scopes = super::super::DRIVE_SCOPES.join(" ");

        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#);

        let claims = serde_json::json!({
            "iss": self.credentials.client_email,
            "scope": scopes,
            "aud": self.credentials.token_uri,
            "iat": now,
            "exp": exp,
        });

        let claims_str = serde_json::to_string(&claims)
            .map_err(|e| GDriveError::AuthenticationFailed(e.to_string()))?;
        let claims_b64 = URL_SAFE_NO_PAD.encode(&claims_str);

        let signing_input = format!("{}.{}", header, claims_b64);

        // Parse private key and sign
        let signature = self.sign_rs256(&signing_input)?;
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

        Ok(format!("{}.{}", signing_input, signature_b64))
    }

    /// Sign data with RS256.
    fn sign_rs256(&self, data: &str) -> Result<Vec<u8>, GDriveError> {
        // Parse PEM private key
        let pem = self.credentials.private_key.replace("\\n", "\n");
        let pem_lines: Vec<&str> = pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect();
        let der = STANDARD.decode(pem_lines.join(""))
            .map_err(|e| GDriveError::AuthenticationFailed(format!("Invalid key: {}", e)))?;

        let key_pair = RsaKeyPair::from_pkcs8(&der)
            .map_err(|e| GDriveError::AuthenticationFailed(format!("Key parse error: {}", e)))?;

        let mut signature = vec![0u8; key_pair.public().modulus_len()];
        let rng = ring::rand::SystemRandom::new();

        key_pair
            .sign(&RSA_PKCS1_SHA256, &rng, data.as_bytes(), &mut signature)
            .map_err(|e| GDriveError::AuthenticationFailed(format!("Sign error: {}", e)))?;

        Ok(signature)
    }

    /// Request an access token using JWT.
    async fn request_token(&self) -> Result<AccessToken, GDriveError> {
        let jwt = self.create_jwt()?;

        let client = reqwest::Client::new();

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ];

        let response = client
            .post(&self.credentials.token_uri)
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
}

#[async_trait]
impl AuthProvider for ServiceAccountAuth {
    async fn get_token(&self) -> Result<AccessToken, GDriveError> {
        let token = self.current_token.read().await;
        if let Some(t) = token.as_ref() {
            if !t.is_expired() {
                return Ok(t.clone());
            }
        }
        drop(token);

        self.request_token().await
    }

    async fn refresh_if_needed(&self) -> Result<AccessToken, GDriveError> {
        let token = self.current_token.read().await;
        if let Some(t) = token.as_ref() {
            let threshold = Utc::now() + Duration::minutes(5);
            if t.expires_at.map(|e| e > threshold).unwrap_or(true) {
                return Ok(t.clone());
            }
        }
        drop(token);

        self.request_token().await
    }

    fn is_configured(&self) -> bool {
        !self.credentials.client_email.is_empty() && !self.credentials.private_key.is_empty()
    }
}

// Note: Using async_trait requires adding it as a dependency.
// For now, we'll use a simple trait without async.
mod async_trait {
    pub use async_trait::async_trait;
}
