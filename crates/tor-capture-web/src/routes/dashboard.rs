//! Dashboard routes.

use crate::{state::AppState, templates::DashboardTemplate};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let targets = state.target_repo.list_all().unwrap_or_default();
    let recent_captures = state.capture_repo.list_recent(10).unwrap_or_default();
    let tor_connected = state.is_tor_connected();

    let running_count = state
        .capture_repo
        .count_by_status(&tor_capture_core::CaptureStatus::Running)
        .unwrap_or(0);

    let template = DashboardTemplate {
        title: "Dashboard".to_string(),
        tor_connected,
        target_count: targets.len(),
        capture_count: recent_captures.len(),
        running_captures: running_count,
        recent_captures,
    };

    Html(template.render())
}
