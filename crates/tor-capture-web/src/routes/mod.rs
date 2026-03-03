//! Route definitions.

mod dashboard;
mod targets;
mod captures;
mod schedules;
mod settings;
mod api;

use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

/// Create the application router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Dashboard
        .route("/", get(dashboard::index))
        // Targets
        .route("/targets", get(targets::list).post(targets::create))
        .route("/targets/new", get(targets::new_form))
        .route(
            "/targets/{id}",
            get(targets::show).put(targets::update).delete(targets::delete),
        )
        .route("/targets/{id}/capture", post(targets::trigger_capture))
        // Captures
        .route("/captures", get(captures::list))
        .route("/captures/{id}", get(captures::show).delete(captures::delete))
        .route("/captures/{id}/screenshot", get(captures::download_screenshot))
        .route("/captures/{id}/html", get(captures::download_html))
        .route("/captures/{id}/upload", post(captures::upload_to_gdrive))
        // Schedules
        .route("/schedules", get(schedules::list).post(schedules::create))
        .route(
            "/schedules/{id}",
            get(schedules::show).put(schedules::update).delete(schedules::delete),
        )
        .route("/schedules/{id}/toggle", post(schedules::toggle))
        // Settings
        .route("/settings", get(settings::index))
        .route("/settings/tor", post(settings::update_tor))
        .route("/settings/gdrive", get(settings::gdrive).post(settings::update_gdrive))
        .route("/settings/gdrive/oauth", get(settings::gdrive_oauth_start))
        .route("/settings/gdrive/callback", get(settings::gdrive_oauth_callback))
        // API
        .route("/api/v1/status", get(api::status))
        .route("/api/v1/targets", get(api::list_targets).post(api::create_target))
        .route(
            "/api/v1/targets/{id}",
            get(api::get_target).put(api::update_target).delete(api::delete_target),
        )
        .route("/api/v1/captures", get(api::list_captures))
        .route("/api/v1/captures/{id}", get(api::get_capture))
        .route("/api/v1/user-agents", get(api::list_user_agents))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // State
        .with_state(state)
}
