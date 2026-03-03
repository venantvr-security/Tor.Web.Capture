//! Capture routes.

use crate::{state::AppState, templates::*};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let captures = state.capture_repo.list_recent(50).unwrap_or_default();

    let template = CapturesListTemplate {
        title: "Captures".to_string(),
        captures,
    };

    Html(template.render())
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.capture_repo.get(&id) {
        Ok(Some(capture)) => {
            let target = state.target_repo.get(&capture.target_id).ok().flatten();

            let template = CaptureDetailTemplate {
                title: "Capture Detail".to_string(),
                capture,
                target,
            };

            Html(template.render()).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Capture not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Get capture to delete files
    if let Ok(Some(capture)) = state.capture_repo.get(&id) {
        // Delete screenshot file
        if let Some(path) = &capture.screenshot_path {
            let _ = tokio::fs::remove_file(path).await;
        }
        // Delete HTML file
        if let Some(path) = &capture.html_path {
            let _ = tokio::fs::remove_file(path).await;
        }
    }

    match state.capture_repo.delete(&id) {
        Ok(()) => (StatusCode::OK, [("HX-Trigger", "capture-deleted")], "").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn download_screenshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.capture_repo.get(&id) {
        Ok(Some(capture)) => {
            if let Some(path) = capture.screenshot_path {
                match tokio::fs::File::open(&path).await {
                    Ok(file) => {
                        let stream = ReaderStream::new(file);
                        let body = Body::from_stream(stream);

                        Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "image/png")
                            .header(
                                header::CONTENT_DISPOSITION,
                                format!("attachment; filename=\"capture_{}.png\"", id),
                            )
                            .body(body)
                            .unwrap()
                            .into_response()
                    }
                    Err(_) => (StatusCode::NOT_FOUND, "Screenshot file not found").into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "No screenshot available").into_response()
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Capture not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn download_html(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.capture_repo.get(&id) {
        Ok(Some(capture)) => {
            if let Some(path) = capture.html_path {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .header(
                            header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"capture_{}.html\"", id),
                        )
                        .body(Body::from(content))
                        .unwrap()
                        .into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "HTML file not found").into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "No HTML available").into_response()
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Capture not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn upload_to_gdrive(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement Google Drive upload
    (StatusCode::NOT_IMPLEMENTED, "Google Drive upload not yet implemented").into_response()
}
