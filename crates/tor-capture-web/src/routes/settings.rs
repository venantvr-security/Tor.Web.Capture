//! Settings routes.

use crate::{state::AppState, templates::*};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TorSettingsForm {
    pub enabled: Option<String>,
    pub new_circuit_per_capture: Option<String>,
}

#[derive(Deserialize)]
pub struct GDriveSettingsForm {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub auto_upload: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config_repo.get_all().unwrap_or_default();
    let tor_connected = state.is_tor_connected();

    let template = SettingsTemplate {
        title: "Settings".to_string(),
        config,
        tor_connected,
        gdrive_configured: false, // TODO: Check from gdrive config table
    };

    Html(template.render())
}

pub async fn update_tor(
    State(state): State<AppState>,
    Form(form): Form<TorSettingsForm>,
) -> impl IntoResponse {
    let enabled = form.enabled.is_some();
    let new_circuit = form.new_circuit_per_capture.is_some();

    if let Err(e) = state.config_repo.set_bool("tor_enabled", enabled) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    if let Err(e) = state
        .config_repo
        .set_bool("tor_new_circuit_per_capture", new_circuit)
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (
        StatusCode::OK,
        [("HX-Trigger", "settings-updated")],
        "TOR settings updated",
    )
        .into_response()
}

pub async fn gdrive(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config_repo.get_all().unwrap_or_default();

    let template = GDriveSettingsTemplate {
        title: "Google Drive Settings".to_string(),
        config,
        configured: false,
        auth_url: None,
    };

    Html(template.render())
}

pub async fn update_gdrive(
    State(state): State<AppState>,
    Form(form): Form<GDriveSettingsForm>,
) -> impl IntoResponse {
    if let Some(client_id) = form.client_id {
        if let Err(e) = state.config_repo.set_string("gdrive_client_id", &client_id) {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    // Note: In production, client_secret should be encrypted before storage
    if let Some(client_secret) = form.client_secret {
        if let Err(e) = state
            .config_repo
            .set_string("gdrive_client_secret", &client_secret)
        {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    let auto_upload = form.auto_upload.is_some();
    if let Err(e) = state.config_repo.set_bool("gdrive_auto_upload", auto_upload) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (
        StatusCode::OK,
        [("HX-Trigger", "settings-updated")],
        "Google Drive settings updated",
    )
        .into_response()
}

pub async fn gdrive_oauth_start(State(state): State<AppState>) -> impl IntoResponse {
    let client_id = state
        .config_repo
        .get("gdrive_client_id")
        .ok()
        .flatten()
        .unwrap_or_default();

    if client_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "Client ID not configured").into_response();
    }

    let redirect_uri = "http://127.0.0.1:8080/settings/gdrive/callback";
    let scopes = "https://www.googleapis.com/auth/drive.file";

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope={}&\
        access_type=offline&\
        prompt=consent",
        urlencoding::encode(&client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(scopes),
    );

    Redirect::to(&auth_url).into_response()
}

pub async fn gdrive_oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallback>,
) -> impl IntoResponse {
    if let Some(error) = params.error {
        return Html(format!(
            "<h1>OAuth Error</h1><p>{}</p><a href='/settings/gdrive'>Back to settings</a>",
            error
        ))
        .into_response();
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            return Html("<h1>Error</h1><p>No authorization code received</p>").into_response()
        }
    };

    // Exchange code for tokens
    // In production, this would call the OAuth2 token endpoint
    // For now, just store the code (demo purposes)
    if let Err(e) = state.config_repo.set_string("gdrive_auth_code", &code) {
        return Html(format!("<h1>Error</h1><p>{}</p>", e)).into_response();
    }

    Html("<h1>Success!</h1><p>Google Drive connected.</p><a href='/settings/gdrive'>Back to settings</a>").into_response()
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
