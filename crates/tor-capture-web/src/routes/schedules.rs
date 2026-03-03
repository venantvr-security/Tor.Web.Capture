//! Schedule routes.

use crate::{state::AppState, templates::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use tor_capture_core::Schedule;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ScheduleForm {
    pub target_id: Uuid,
    pub cron_expression: String,
    pub timezone: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let schedules = state.schedule_repo.list_enabled().unwrap_or_default();

    // Get target names for display
    let mut schedule_with_targets = Vec::new();
    for schedule in schedules {
        let target = state.target_repo.get(&schedule.target_id).ok().flatten();
        schedule_with_targets.push((schedule, target));
    }

    let template = SchedulesListTemplate {
        title: "Schedules".to_string(),
        schedules: schedule_with_targets,
    };

    Html(template.render())
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<ScheduleForm>,
) -> impl IntoResponse {
    // Validate cron expression
    if let Err(e) = tor_capture_scheduler::validate_cron(&form.cron_expression) {
        return (StatusCode::BAD_REQUEST, format!("Invalid cron: {}", e)).into_response();
    }

    // Check target exists
    if state.target_repo.get(&form.target_id).ok().flatten().is_none() {
        return (StatusCode::BAD_REQUEST, "Target not found").into_response();
    }

    let mut schedule = Schedule::new(form.target_id, form.cron_expression);
    if let Some(tz) = form.timezone {
        schedule.timezone = tz;
    }

    // Calculate next run
    if let Ok(next) = tor_capture_scheduler::next_run_time(&schedule.cron_expression) {
        schedule.next_run_at = Some(next);
    }

    match state.schedule_repo.create(&schedule) {
        Ok(()) => {
            let target = state.target_repo.get(&schedule.target_id).ok().flatten();
            let template = ScheduleRowTemplate {
                schedule,
                target,
            };
            (
                StatusCode::CREATED,
                [("HX-Trigger", "schedule-created")],
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
    match state.schedule_repo.get(&id) {
        Ok(Some(schedule)) => {
            let target = state.target_repo.get(&schedule.target_id).ok().flatten();
            let next_run = tor_capture_scheduler::format_next_run(&schedule.cron_expression);

            let template = ScheduleDetailTemplate {
                title: "Schedule Detail".to_string(),
                schedule,
                target,
                next_run,
            };

            Html(template.render()).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Schedule not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(form): Form<ScheduleForm>,
) -> impl IntoResponse {
    // Validate cron expression
    if let Err(e) = tor_capture_scheduler::validate_cron(&form.cron_expression) {
        return (StatusCode::BAD_REQUEST, format!("Invalid cron: {}", e)).into_response();
    }

    match state.schedule_repo.get(&id) {
        Ok(Some(mut schedule)) => {
            schedule.cron_expression = form.cron_expression;
            if let Some(tz) = form.timezone {
                schedule.timezone = tz;
            }

            // Recalculate next run
            if let Ok(next) = tor_capture_scheduler::next_run_time(&schedule.cron_expression) {
                schedule.next_run_at = Some(next);
            }

            match state.schedule_repo.update(&schedule) {
                Ok(()) => (
                    StatusCode::OK,
                    [("HX-Trigger", "schedule-updated")],
                    "Updated",
                )
                    .into_response(),
                Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Schedule not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.schedule_repo.delete(&id) {
        Ok(()) => (StatusCode::OK, [("HX-Trigger", "schedule-deleted")], "").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn toggle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.schedule_repo.get(&id) {
        Ok(Some(mut schedule)) => {
            schedule.enabled = !schedule.enabled;

            // Recalculate next run if enabling
            if schedule.enabled {
                if let Ok(next) = tor_capture_scheduler::next_run_time(&schedule.cron_expression) {
                    schedule.next_run_at = Some(next);
                }
            }

            match state.schedule_repo.update(&schedule) {
                Ok(()) => {
                    let status = if schedule.enabled { "enabled" } else { "disabled" };
                    (
                        StatusCode::OK,
                        [("HX-Trigger", "schedule-toggled")],
                        status,
                    )
                        .into_response()
                }
                Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Schedule not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
