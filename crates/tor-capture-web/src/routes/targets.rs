//! Target routes.

use crate::{state::AppState, templates::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use tor_capture_core::{Capture, SpiderConfig, Target, UserAgentType};
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TargetForm {
    pub name: String,
    pub url: String,
    pub capture_screenshot: Option<String>,
    pub capture_html: Option<String>,
    pub user_agent_type: String,
    pub custom_user_agent: Option<String>,
    pub viewport_width: Option<i32>,
    pub viewport_height: Option<i32>,
    pub wait_after_load_ms: Option<i64>,
    // Spider options
    pub spider_enabled: Option<String>,
    pub spider_max_depth: Option<u32>,
    pub spider_max_urls: Option<usize>,
    pub spider_delay_ms: Option<u64>,
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let targets = state.target_repo.list_all().unwrap_or_default();

    let template = TargetsListTemplate {
        title: "Targets".to_string(),
        targets,
    };

    Html(template.render())
}

pub async fn new_form() -> impl IntoResponse {
    let template = TargetFormTemplate {
        title: "New Target".to_string(),
        target: None,
        user_agent_types: get_user_agent_types(),
    };

    Html(template.render())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<TargetForm>,
) -> impl IntoResponse {
    let mut target = Target::new(form.name, form.url);
    target.capture_screenshot = form.capture_screenshot.is_some();
    target.capture_html = form.capture_html.is_some();
    target.user_agent_type = parse_user_agent_type(&form.user_agent_type);
    target.custom_user_agent = form.custom_user_agent;
    target.viewport_width = form.viewport_width.unwrap_or(1920);
    target.viewport_height = form.viewport_height.unwrap_or(1080);
    target.wait_after_load_ms = form.wait_after_load_ms.unwrap_or(2000);

    match state.target_repo.create(&target) {
        Ok(()) => {
            // Save spider config if enabled
            if form.spider_enabled.is_some() {
                let spider_config = SpiderConfig {
                    enabled: true,
                    max_depth: form.spider_max_depth.unwrap_or(2),
                    same_domain_only: true,
                    max_urls: form.spider_max_urls.unwrap_or(100),
                    delay_ms: form.spider_delay_ms.unwrap_or(1000),
                };
                let _ = state.spider_engine.set_config(&target.id, spider_config).await;
            }

            let template = TargetCardTemplate { target };
            (
                StatusCode::CREATED,
                [("HX-Trigger", "target-created")],
                Html(template.render()),
            )
                .into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.target_repo.get(&id) {
        Ok(Some(target)) => {
            let captures = state.capture_repo.list_by_target(&id, 20).unwrap_or_default();
            let schedules = state.schedule_repo.list_by_target(&id).unwrap_or_default();

            let template = TargetDetailTemplate {
                title: format!("Target: {}", target.name),
                target,
                captures,
                schedules,
                user_agent_types: get_user_agent_types(),
            };

            Html(template.render()).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Target not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<TargetForm>,
) -> impl IntoResponse {
    match state.target_repo.get(&id) {
        Ok(Some(mut target)) => {
            target.name = form.name;
            target.url = form.url;
            target.capture_screenshot = form.capture_screenshot.is_some();
            target.capture_html = form.capture_html.is_some();
            target.user_agent_type = parse_user_agent_type(&form.user_agent_type);
            target.custom_user_agent = form.custom_user_agent;
            target.viewport_width = form.viewport_width.unwrap_or(target.viewport_width);
            target.viewport_height = form.viewport_height.unwrap_or(target.viewport_height);
            target.wait_after_load_ms = form.wait_after_load_ms.unwrap_or(target.wait_after_load_ms);

            match state.target_repo.update(&target) {
                Ok(()) => {
                    // Update spider config
                    if form.spider_enabled.is_some() {
                        let spider_config = SpiderConfig {
                            enabled: true,
                            max_depth: form.spider_max_depth.unwrap_or(2),
                            same_domain_only: true,
                            max_urls: form.spider_max_urls.unwrap_or(100),
                            delay_ms: form.spider_delay_ms.unwrap_or(1000),
                        };
                        let _ = state.spider_engine.set_config(&target.id, spider_config).await;
                    } else {
                        // Disable spider if checkbox not checked
                        let _ = state.spider_engine.remove_config(&target.id).await;
                    }

                    (
                        StatusCode::OK,
                        [("HX-Trigger", "target-updated")],
                        "Updated",
                    )
                        .into_response()
                }
                Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Target not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.target_repo.delete(&id) {
        Ok(()) => (StatusCode::OK, [("HX-Trigger", "target-deleted")], "").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn trigger_capture(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let target = match state.target_repo.get(&id) {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "Target not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Check if spider mode is enabled
    if let Some(spider_config) = state.spider_engine.get_config(&target.id).await {
        if spider_config.enabled {
            return trigger_spider_capture(state, target, spider_config).await;
        }
    }

    // Create capture record
    let mut capture = Capture::new(target.id, None);
    if let Err(e) = state.capture_repo.create(&capture) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Start capture in background
    let capture_engine = state.capture_engine.clone();
    let capture_repo = state.capture_repo.clone();
    let capture_id = capture.id;
    let target_clone = target.clone();

    tokio::spawn(async move {
        capture.mark_started();
        let _ = capture_repo.update(&capture);

        match capture_engine.execute_capture(&target_clone).await {
            Ok(result) => {
                if let Err(e) = capture_engine
                    .save_capture(&target_clone, &result, &mut capture)
                    .await
                {
                    capture.mark_failed("save_error", &e.to_string());
                } else {
                    capture.mark_success();
                }
            }
            Err(e) => {
                capture.mark_failed("capture_error", &e.to_string());
            }
        }

        let _ = capture_repo.update(&capture);
    });

    let template = CaptureStatusTemplate {
        capture_id,
        status: "running".to_string(),
        message: "Capture started...".to_string(),
    };

    (
        StatusCode::ACCEPTED,
        [("HX-Trigger", "capture-started")],
        Html(template.render()),
    )
        .into_response()
}

async fn trigger_spider_capture(
    state: AppState,
    target: Target,
    config: SpiderConfig,
) -> axum::response::Response {
    info!(
        "Starting spider capture for target {} (max_depth: {}, max_urls: {})",
        target.id, config.max_depth, config.max_urls
    );

    let spider_engine = state.spider_engine.clone();
    let capture_engine = state.capture_engine.clone();
    let capture_repo = state.capture_repo.clone();
    let target_clone = target.clone();
    let config_clone = config.clone();

    // Start spider in background
    tokio::spawn(async move {
        let _ = spider_engine
            .execute_spider(&target_clone, &config_clone, |result, url, depth| {
                // Create capture record for each spidered URL
                let mut capture = Capture::new(target_clone.id, None);
                capture.mark_started();
                capture.final_url = Some(url.to_string());
                capture.page_title = result.page_title.clone();

                // Save capture to DB
                if capture_repo.create(&capture).is_ok() {
                    // Save files
                    let capture_engine_clone = capture_engine.clone();
                    let target_for_save = target_clone.clone();
                    let capture_repo_clone = capture_repo.clone();

                    // Note: This is sync context, so we use blocking approach
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        if let Err(e) = capture_engine_clone
                            .save_capture(&target_for_save, &result, &mut capture)
                            .await
                        {
                            capture.mark_failed("save_error", &e.to_string());
                        } else {
                            capture.mark_success();
                        }
                        let _ = capture_repo_clone.update(&capture);
                    });

                    info!("Spider captured: {} (depth {})", url, depth);
                }
            })
            .await;
    });

    let template = CaptureStatusTemplate {
        capture_id: Uuid::new_v4(),
        status: "running".to_string(),
        message: format!(
            "Spider crawl started (max depth: {}, max URLs: {})",
            config.max_depth, config.max_urls
        ),
    };

    (
        StatusCode::ACCEPTED,
        [("HX-Trigger", "spider-started")],
        Html(template.render()),
    )
        .into_response()
}

fn parse_user_agent_type(s: &str) -> UserAgentType {
    match s.to_lowercase().as_str() {
        "shodan" => UserAgentType::Shodan,
        "censys" => UserAgentType::Censys,
        "zgrab" => UserAgentType::ZGrab,
        "masscan" => UserAgentType::Masscan,
        "nmap" => UserAgentType::Nmap,
        "binaryedge" => UserAgentType::BinaryEdge,
        "fofa" => UserAgentType::Fofa,
        "zoomeye" => UserAgentType::ZoomEye,
        "greynoise" => UserAgentType::GreyNoise,
        "custom" => UserAgentType::Custom,
        _ => UserAgentType::Random,
    }
}

fn get_user_agent_types() -> Vec<(&'static str, &'static str)> {
    vec![
        ("random", "Random"),
        ("shodan", "Shodan"),
        ("censys", "Censys"),
        ("zgrab", "ZGrab"),
        ("masscan", "Masscan"),
        ("nmap", "Nmap"),
        ("binaryedge", "BinaryEdge"),
        ("fofa", "FOFA"),
        ("zoomeye", "ZoomEye"),
        ("greynoise", "GreyNoise"),
        ("custom", "Custom"),
    ]
}
