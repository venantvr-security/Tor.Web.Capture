//! Template definitions for Askama.
//! Note: These are simplified string templates. In production, use Askama.

use std::collections::HashMap;
use tor_capture_core::{Capture, Schedule, Target};

/// Base layout wrapper.
fn layout(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Tor.Web.Capture</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <link href="https://cdn.jsdelivr.net/npm/daisyui@4.4.19/dist/full.min.css" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="min-h-screen bg-base-200">
    <div class="navbar bg-base-100 shadow-lg">
        <div class="flex-1">
            <a href="/" class="btn btn-ghost text-xl">Tor.Web.Capture</a>
        </div>
        <div class="flex-none">
            <ul class="menu menu-horizontal px-1">
                <li><a href="/">Dashboard</a></li>
                <li><a href="/targets">Targets</a></li>
                <li><a href="/captures">Captures</a></li>
                <li><a href="/schedules">Schedules</a></li>
                <li><a href="/settings">Settings</a></li>
            </ul>
        </div>
    </div>
    <main class="container mx-auto p-4">
        {}
    </main>
</body>
</html>"#,
        title, content
    )
}

/// Dashboard template.
pub struct DashboardTemplate {
    pub title: String,
    pub tor_connected: bool,
    pub target_count: usize,
    pub capture_count: usize,
    pub running_captures: usize,
    pub recent_captures: Vec<Capture>,
}

impl DashboardTemplate {
    pub fn render(&self) -> String {
        let tor_status = if self.tor_connected {
            r#"<div class="badge badge-success">TOR Connected</div>"#
        } else {
            r#"<div class="badge badge-error">TOR Disconnected</div>"#
        };

        let captures_html: String = self
            .recent_captures
            .iter()
            .map(|c| {
                format!(
                    r#"<tr>
                    <td>{}</td>
                    <td><span class="badge badge-{}">{}</span></td>
                    <td>{}</td>
                    <td><a href="/captures/{}" class="btn btn-xs btn-ghost">View</a></td>
                </tr>"#,
                    c.id,
                    match c.status {
                        tor_capture_core::CaptureStatus::Success => "success",
                        tor_capture_core::CaptureStatus::Failed => "error",
                        tor_capture_core::CaptureStatus::Running => "warning",
                        tor_capture_core::CaptureStatus::Pending => "ghost",
                    },
                    c.status,
                    c.created_at.format("%Y-%m-%d %H:%M"),
                    c.id
                )
            })
            .collect();

        let content = format!(
            r#"<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
            <div class="stat bg-base-100 rounded-lg shadow">
                <div class="stat-title">TOR Status</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat bg-base-100 rounded-lg shadow">
                <div class="stat-title">Targets</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat bg-base-100 rounded-lg shadow">
                <div class="stat-title">Total Captures</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat bg-base-100 rounded-lg shadow">
                <div class="stat-title">Running</div>
                <div class="stat-value">{}</div>
            </div>
        </div>
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">Recent Captures</h2>
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead><tr><th>ID</th><th>Status</th><th>Date</th><th>Actions</th></tr></thead>
                        <tbody>{}</tbody>
                    </table>
                </div>
            </div>
        </div>"#,
            tor_status,
            self.target_count,
            self.capture_count,
            self.running_captures,
            captures_html
        );

        layout(&self.title, &content)
    }
}

/// Targets list template.
pub struct TargetsListTemplate {
    pub title: String,
    pub targets: Vec<Target>,
}

impl TargetsListTemplate {
    pub fn render(&self) -> String {
        let targets_html: String = self
            .targets
            .iter()
            .map(|t| {
                format!(
                    r##"<div id="target-{}" class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h3 class="card-title">{}</h3>
                        <p class="text-sm text-gray-500 truncate">{}</p>
                        <div class="flex gap-2 mt-2">
                            {}{}
                            <span class="badge badge-ghost">{}</span>
                        </div>
                        <div class="card-actions justify-end mt-4">
                            <button hx-post="/targets/{}/capture" hx-target="#captures-list" hx-swap="afterbegin" class="btn btn-sm btn-accent">Capture</button>
                            <a href="/targets/{}" class="btn btn-sm btn-ghost">Edit</a>
                            <button hx-delete="/targets/{}" hx-target="#target-{}" hx-swap="outerHTML" hx-confirm="Delete this target?" class="btn btn-sm btn-error btn-outline">Delete</button>
                        </div>
                    </div>
                </div>"##,
                    t.id, t.name, t.url,
                    if t.capture_screenshot { r#"<span class="badge badge-info">Screenshot</span>"# } else { "" },
                    if t.capture_html { r#"<span class="badge badge-success">HTML</span>"# } else { "" },
                    t.user_agent_type,
                    t.id, t.id, t.id, t.id
                )
            })
            .collect();

        let content = format!(
            r#"<div class="flex justify-between items-center mb-6">
            <h2 class="text-2xl font-bold">Targets</h2>
            <a href="/targets/new" class="btn btn-primary">+ New Target</a>
        </div>
        <div id="targets-list" class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">{}</div>"#,
            targets_html
        );

        layout(&self.title, &content)
    }
}

/// Target card partial.
pub struct TargetCardTemplate {
    pub target: Target,
}

impl TargetCardTemplate {
    pub fn render(&self) -> String {
        let t = &self.target;
        format!(
            r#"<div id="target-{}" class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h3 class="card-title">{}</h3>
                <p class="text-sm text-gray-500 truncate">{}</p>
                <div class="card-actions justify-end mt-4">
                    <button hx-post="/targets/{}/capture" class="btn btn-sm btn-accent">Capture</button>
                    <a href="/targets/{}" class="btn btn-sm btn-ghost">Edit</a>
                </div>
            </div>
        </div>"#,
            t.id, t.name, t.url, t.id, t.id
        )
    }
}

/// Target form template.
pub struct TargetFormTemplate {
    pub title: String,
    pub target: Option<Target>,
    pub user_agent_types: Vec<(&'static str, &'static str)>,
}

impl TargetFormTemplate {
    pub fn render(&self) -> String {
        let (action, method, name, url) = match &self.target {
            Some(t) => (
                format!("/targets/{}", t.id),
                "put",
                t.name.as_str(),
                t.url.as_str(),
            ),
            None => ("/targets".to_string(), "post", "", ""),
        };

        let options: String = self
            .user_agent_types
            .iter()
            .map(|(value, label)| format!(r#"<option value="{}">{}</option>"#, value, label))
            .collect();

        let content = format!(
            r##"<div class="card bg-base-100 shadow-xl max-w-2xl mx-auto">
            <div class="card-body">
                <h2 class="card-title">{}</h2>
                <form hx-{}="{}" hx-target="#targets-list" hx-swap="beforeend">
                    <div class="form-control">
                        <label class="label"><span class="label-text">Name</span></label>
                        <input type="text" name="name" value="{}" class="input input-bordered" required>
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text">URL</span></label>
                        <input type="url" name="url" value="{}" class="input input-bordered" required>
                    </div>
                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">Capture Screenshot</span>
                            <input type="checkbox" name="capture_screenshot" checked class="checkbox">
                        </label>
                    </div>
                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">Capture HTML</span>
                            <input type="checkbox" name="capture_html" checked class="checkbox">
                        </label>
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text">User Agent Type</span></label>
                        <select name="user_agent_type" class="select select-bordered">{}</select>
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text">Viewport Width</span></label>
                        <input type="number" name="viewport_width" value="1920" class="input input-bordered">
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text">Viewport Height</span></label>
                        <input type="number" name="viewport_height" value="1080" class="input input-bordered">
                    </div>
                    <div class="form-control mt-6">
                        <button type="submit" class="btn btn-primary">Save Target</button>
                    </div>
                </form>
            </div>
        </div>"##,
            self.title, method, action, name, url, options
        );

        layout(&self.title, &content)
    }
}

/// Target detail template.
pub struct TargetDetailTemplate {
    pub title: String,
    pub target: Target,
    pub captures: Vec<Capture>,
    pub schedules: Vec<Schedule>,
    pub user_agent_types: Vec<(&'static str, &'static str)>,
}

impl TargetDetailTemplate {
    pub fn render(&self) -> String {
        let form = TargetFormTemplate {
            title: self.title.clone(),
            target: Some(self.target.clone()),
            user_agent_types: self.user_agent_types.clone(),
        };
        layout(&self.title, &form.render())
    }
}

/// Captures list template.
pub struct CapturesListTemplate {
    pub title: String,
    pub captures: Vec<Capture>,
}

impl CapturesListTemplate {
    pub fn render(&self) -> String {
        let rows: String = self
            .captures
            .iter()
            .map(|c| {
                format!(
                    r#"<tr>
                    <td>{}</td>
                    <td><span class="badge badge-{}">{}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>
                        <a href="/captures/{}" class="btn btn-xs btn-ghost">View</a>
                        {}
                        {}
                    </td>
                </tr>"#,
                    &c.id.to_string()[..8],
                    match c.status {
                        tor_capture_core::CaptureStatus::Success => "success",
                        tor_capture_core::CaptureStatus::Failed => "error",
                        tor_capture_core::CaptureStatus::Running => "warning",
                        tor_capture_core::CaptureStatus::Pending => "ghost",
                    },
                    c.status,
                    c.page_title.as_deref().unwrap_or("-"),
                    c.created_at.format("%Y-%m-%d %H:%M"),
                    c.id,
                    if c.screenshot_path.is_some() {
                        format!(r#"<a href="/captures/{}/screenshot" class="btn btn-xs btn-info">Screenshot</a>"#, c.id)
                    } else { String::new() },
                    if c.html_path.is_some() {
                        format!(r#"<a href="/captures/{}/html" class="btn btn-xs btn-success">HTML</a>"#, c.id)
                    } else { String::new() }
                )
            })
            .collect();

        let content = format!(
            r#"<h2 class="text-2xl font-bold mb-6">Captures</h2>
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead><tr><th>ID</th><th>Status</th><th>Title</th><th>Date</th><th>Actions</th></tr></thead>
                        <tbody>{}</tbody>
                    </table>
                </div>
            </div>
        </div>"#,
            rows
        );

        layout(&self.title, &content)
    }
}

/// Capture detail template.
pub struct CaptureDetailTemplate {
    pub title: String,
    pub capture: Capture,
    pub target: Option<Target>,
}

impl CaptureDetailTemplate {
    pub fn render(&self) -> String {
        let c = &self.capture;
        let content = format!(
            r#"<div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">Capture: {}</h2>
                <div class="grid grid-cols-2 gap-4">
                    <div><strong>Status:</strong> <span class="badge">{}</span></div>
                    <div><strong>Created:</strong> {}</div>
                    <div><strong>Title:</strong> {}</div>
                    <div><strong>Final URL:</strong> {}</div>
                    <div><strong>Exit Node:</strong> {}</div>
                    <div><strong>User Agent:</strong> {}</div>
                </div>
                {}
                <div class="card-actions justify-end mt-4">
                    {}
                    {}
                    <button hx-delete="/captures/{}" hx-confirm="Delete this capture?" class="btn btn-error">Delete</button>
                </div>
            </div>
        </div>"#,
            c.id,
            c.status,
            c.created_at.format("%Y-%m-%d %H:%M:%S"),
            c.page_title.as_deref().unwrap_or("-"),
            c.final_url.as_deref().unwrap_or("-"),
            c.exit_node_ip.as_deref().unwrap_or("-"),
            c.user_agent_used.as_deref().unwrap_or("-"),
            if let Some(err) = &c.error_message {
                format!(r#"<div class="alert alert-error mt-4"><span>{}</span></div>"#, err)
            } else { String::new() },
            if c.screenshot_path.is_some() {
                format!(r#"<a href="/captures/{}/screenshot" class="btn btn-info">Download Screenshot</a>"#, c.id)
            } else { String::new() },
            if c.html_path.is_some() {
                format!(r#"<a href="/captures/{}/html" class="btn btn-success">Download HTML</a>"#, c.id)
            } else { String::new() },
            c.id
        );

        layout(&self.title, &content)
    }
}

/// Capture status partial.
pub struct CaptureStatusTemplate {
    pub capture_id: uuid::Uuid,
    pub status: String,
    pub message: String,
}

impl CaptureStatusTemplate {
    pub fn render(&self) -> String {
        format!(
            r#"<div class="alert alert-info" hx-get="/captures/{}" hx-trigger="every 2s" hx-swap="outerHTML">
            <span class="loading loading-spinner"></span>
            <span>{}</span>
        </div>"#,
            self.capture_id, self.message
        )
    }
}

/// Schedules list template.
pub struct SchedulesListTemplate {
    pub title: String,
    pub schedules: Vec<(Schedule, Option<Target>)>,
}

impl SchedulesListTemplate {
    pub fn render(&self) -> String {
        let rows: String = self
            .schedules
            .iter()
            .map(|(s, t)| {
                let target_name = t.as_ref().map(|t| t.name.as_str()).unwrap_or("-");
                format!(
                    r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td>
                        <button hx-post="/schedules/{}/toggle" class="btn btn-xs {}">{}</button>
                        <button hx-delete="/schedules/{}" hx-confirm="Delete?" class="btn btn-xs btn-error">Delete</button>
                    </td>
                </tr>"#,
                    target_name,
                    s.cron_expression,
                    s.next_run_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or("-".to_string()),
                    s.run_count,
                    s.id,
                    if s.enabled { "btn-success" } else { "btn-ghost" },
                    if s.enabled { "Enabled" } else { "Disabled" },
                    s.id
                )
            })
            .collect();

        let content = format!(
            r#"<h2 class="text-2xl font-bold mb-6">Schedules</h2>
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead><tr><th>Target</th><th>Cron</th><th>Next Run</th><th>Runs</th><th>Actions</th></tr></thead>
                        <tbody>{}</tbody>
                    </table>
                </div>
            </div>
        </div>"#,
            rows
        );

        layout(&self.title, &content)
    }
}

/// Schedule row partial.
pub struct ScheduleRowTemplate {
    pub schedule: Schedule,
    pub target: Option<Target>,
}

impl ScheduleRowTemplate {
    pub fn render(&self) -> String {
        let s = &self.schedule;
        let target_name = self.target.as_ref().map(|t| t.name.as_str()).unwrap_or("-");
        format!(
            r#"<tr>
            <td>{}</td>
            <td><code>{}</code></td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <button hx-post="/schedules/{}/toggle" class="btn btn-xs btn-success">Enabled</button>
            </td>
        </tr>"#,
            target_name,
            s.cron_expression,
            s.next_run_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or("-".to_string()),
            s.run_count,
            s.id
        )
    }
}

/// Schedule detail template.
pub struct ScheduleDetailTemplate {
    pub title: String,
    pub schedule: Schedule,
    pub target: Option<Target>,
    pub next_run: String,
}

impl ScheduleDetailTemplate {
    pub fn render(&self) -> String {
        let content = format!(
            r#"<div class="card bg-base-100 shadow-xl max-w-2xl mx-auto">
            <div class="card-body">
                <h2 class="card-title">Schedule Details</h2>
                <div class="grid grid-cols-2 gap-4">
                    <div><strong>Target:</strong> {}</div>
                    <div><strong>Cron:</strong> <code>{}</code></div>
                    <div><strong>Next Run:</strong> {}</div>
                    <div><strong>Run Count:</strong> {}</div>
                    <div><strong>Failures:</strong> {}</div>
                    <div><strong>Status:</strong> {}</div>
                </div>
            </div>
        </div>"#,
            self.target.as_ref().map(|t| t.name.as_str()).unwrap_or("-"),
            self.schedule.cron_expression,
            self.next_run,
            self.schedule.run_count,
            self.schedule.failure_count,
            if self.schedule.enabled { "Enabled" } else { "Disabled" }
        );

        layout(&self.title, &content)
    }
}

/// Settings template.
pub struct SettingsTemplate {
    pub title: String,
    pub config: HashMap<String, String>,
    pub tor_connected: bool,
    pub gdrive_configured: bool,
}

impl SettingsTemplate {
    pub fn render(&self) -> String {
        let tor_enabled = self.config.get("tor_enabled").map(|v| v == "true").unwrap_or(true);

        let content = format!(
            r#"<h2 class="text-2xl font-bold mb-6">Settings</h2>
        <div class="grid gap-6 md:grid-cols-2">
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <h3 class="card-title">TOR Configuration</h3>
                    <div class="mb-4">
                        <span class="badge {}">{}</span>
                    </div>
                    <form hx-post="/settings/tor" hx-swap="none">
                        <div class="form-control">
                            <label class="label cursor-pointer">
                                <span class="label-text">Enable TOR</span>
                                <input type="checkbox" name="enabled" {} class="checkbox">
                            </label>
                        </div>
                        <div class="form-control">
                            <label class="label cursor-pointer">
                                <span class="label-text">New circuit per capture</span>
                                <input type="checkbox" name="new_circuit_per_capture" checked class="checkbox">
                            </label>
                        </div>
                        <div class="form-control mt-4">
                            <button type="submit" class="btn btn-primary">Save</button>
                        </div>
                    </form>
                </div>
            </div>
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                    <h3 class="card-title">Google Drive</h3>
                    <div class="mb-4">
                        <span class="badge {}">{}</span>
                    </div>
                    <a href="/settings/gdrive" class="btn btn-outline">Configure Google Drive</a>
                </div>
            </div>
        </div>"#,
            if self.tor_connected { "badge-success" } else { "badge-error" },
            if self.tor_connected { "Connected" } else { "Disconnected" },
            if tor_enabled { "checked" } else { "" },
            if self.gdrive_configured { "badge-success" } else { "badge-ghost" },
            if self.gdrive_configured { "Configured" } else { "Not Configured" }
        );

        layout(&self.title, &content)
    }
}

/// Google Drive settings template.
pub struct GDriveSettingsTemplate {
    pub title: String,
    pub config: HashMap<String, String>,
    pub configured: bool,
    pub auth_url: Option<String>,
}

impl GDriveSettingsTemplate {
    pub fn render(&self) -> String {
        let content = format!(
            r#"<div class="card bg-base-100 shadow-xl max-w-2xl mx-auto">
            <div class="card-body">
                <h2 class="card-title">Google Drive Configuration</h2>
                <form hx-post="/settings/gdrive" hx-swap="none">
                    <div class="form-control">
                        <label class="label"><span class="label-text">Client ID</span></label>
                        <input type="text" name="client_id" value="{}" class="input input-bordered">
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text">Client Secret</span></label>
                        <input type="password" name="client_secret" class="input input-bordered">
                    </div>
                    <div class="form-control">
                        <label class="label cursor-pointer">
                            <span class="label-text">Auto-upload captures</span>
                            <input type="checkbox" name="auto_upload" class="checkbox">
                        </label>
                    </div>
                    <div class="form-control mt-4">
                        <button type="submit" class="btn btn-primary">Save</button>
                    </div>
                </form>
                <div class="divider"></div>
                <a href="/settings/gdrive/oauth" class="btn btn-outline">Connect with Google</a>
            </div>
        </div>"#,
            self.config.get("gdrive_client_id").unwrap_or(&String::new())
        );

        layout(&self.title, &content)
    }
}
