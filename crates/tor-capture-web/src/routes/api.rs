//! JSON API routes.

use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tor_capture_core::{CaptureStatus, Target};
use uuid::Uuid;

#[derive(Serialize)]
pub struct StatusResponse {
    pub tor_connected: bool,
    pub captures_running: usize,
    pub total_targets: usize,
    pub total_captures: usize,
}

#[derive(Deserialize)]
pub struct CreateTargetRequest {
    pub name: String,
    pub url: String,
    pub capture_screenshot: Option<bool>,
    pub capture_html: Option<bool>,
    pub user_agent_type: Option<String>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let targets = state.target_repo.list_all().unwrap_or_default();
    let running = state
        .capture_repo
        .count_by_status(&CaptureStatus::Running)
        .unwrap_or(0);
    let captures = state.capture_repo.list_recent(1000).unwrap_or_default();

    Json(StatusResponse {
        tor_connected: state.is_tor_connected(),
        captures_running: running,
        total_targets: targets.len(),
        total_captures: captures.len(),
    })
}

pub async fn list_targets(State(state): State<AppState>) -> impl IntoResponse {
    match state.target_repo.list_all() {
        Ok(targets) => Json(targets).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn create_target(
    State(state): State<AppState>,
    Json(req): Json<CreateTargetRequest>,
) -> impl IntoResponse {
    let mut target = Target::new(req.name, req.url);
    target.capture_screenshot = req.capture_screenshot.unwrap_or(true);
    target.capture_html = req.capture_html.unwrap_or(true);

    if let Some(ua_type) = req.user_agent_type {
        target.user_agent_type = match ua_type.to_lowercase().as_str() {
            "shodan" => tor_capture_core::UserAgentType::Shodan,
            "censys" => tor_capture_core::UserAgentType::Censys,
            "zgrab" => tor_capture_core::UserAgentType::ZGrab,
            _ => tor_capture_core::UserAgentType::Random,
        };
    }

    match state.target_repo.create(&target) {
        Ok(()) => (StatusCode::CREATED, Json(target)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn get_target(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.target_repo.get(&id) {
        Ok(Some(target)) => Json(target).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Target not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn update_target(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateTargetRequest>,
) -> impl IntoResponse {
    match state.target_repo.get(&id) {
        Ok(Some(mut target)) => {
            target.name = req.name;
            target.url = req.url;
            if let Some(ss) = req.capture_screenshot {
                target.capture_screenshot = ss;
            }
            if let Some(html) = req.capture_html {
                target.capture_html = html;
            }

            match state.target_repo.update(&target) {
                Ok(()) => Json(target).into_response(),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        error: e.to_string(),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Target not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn delete_target(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.target_repo.delete(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_captures(State(state): State<AppState>) -> impl IntoResponse {
    match state.capture_repo.list_recent(100) {
        Ok(captures) => Json(captures).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn get_capture(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.capture_repo.get(&id) {
        Ok(Some(capture)) => Json(capture).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Capture not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_user_agents(State(state): State<AppState>) -> impl IntoResponse {
    match state.user_agent_repo.list_all() {
        Ok(agents) => Json(agents).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}
