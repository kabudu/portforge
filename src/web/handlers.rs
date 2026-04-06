use crate::models::Status;
use crate::process;
use crate::web::assets::StaticAssets;
#[allow(unused_imports)]
use crate::web::server::SharedState;
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Json},
};
use serde::Serialize;

// ─── Dashboard Page ───

pub async fn dashboard_page(State(state): State<SharedState>) -> Html<String> {
    let state = state.lock().await;
    let total = state.entries.len();
    let healthy = state
        .entries
        .iter()
        .filter(|e| e.status == Status::Healthy)
        .count();
    let docker_count = state.entries.iter().filter(|e| e.docker.is_some()).count();
    let total_mem: f64 = state
        .entries
        .iter()
        .map(|e| e.memory_mb)
        .sum::<f64>()
        .max(0.0);

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PortForge Dashboard</title>
    <meta name="description" content="PortForge — Modern port inspector and manager web dashboard">
    <meta name="theme-color" content="#09101c">
    <link rel="icon" type="image/png" href="/static/logo.png">
    <link rel="apple-touch-icon" href="/static/logo.png">
    <link rel="stylesheet" href="/static/style.css">
    <script src="https://unpkg.com/htmx.org@1.9.12"></script>
</head>
<body>
    <div class="app">
        <!-- Header -->
        <header class="header">
            <div class="header-left">
                <div class="logo">
                    <img src="/static/logo.png" alt="PortForge logo" class="logo-mark">
                    <h1>PortForge</h1>
                    <span class="version">v{version}</span>
                </div>
            </div>
            <div class="header-right">
                <div class="live-indicator">
                    <span class="pulse"></span>
                    <span>Live</span>
                </div>
            </div>
        </header>

        <!-- Stats Cards -->
        <div id="stats-section" hx-get="/partials/stats" hx-trigger="every 3s" hx-swap="innerHTML">
            {stats_html}
        </div>

        <!-- Port Table -->
        <main class="main-content">
            <div class="table-header">
                <h2>Active Ports</h2>
                <div class="table-actions">
                    <input type="search" id="port-search" placeholder="Search ports..." class="search-input"
                           onkeyup="filterTable(this.value)">
                </div>
            </div>
            <div id="port-table" hx-get="/partials/table" hx-trigger="every 3s" hx-swap="innerHTML">
                {table_html}
            </div>
        </main>
    </div>

    <!-- Detail Modal -->
    <div id="detail-modal" class="modal" style="display:none">
        <div class="modal-backdrop" onclick="closeModal()"></div>
        <div class="modal-content">
            <button class="modal-close" onclick="closeModal()">✕</button>
            <div id="modal-body"></div>
        </div>
    </div>

    <script src="/static/app.js"></script>
</body>
</html>"##,
        version = env!("CARGO_PKG_VERSION"),
        stats_html = render_stats_cards(total, healthy, docker_count, total_mem),
        table_html = render_port_table(&state.entries),
    );

    Html(html)
}

// ─── HTMX Partials ───

pub async fn partial_table(State(state): State<SharedState>) -> Html<String> {
    let state = state.lock().await;
    Html(render_port_table(&state.entries))
}

pub async fn partial_stats(State(state): State<SharedState>) -> Html<String> {
    let state = state.lock().await;
    let total = state.entries.len();
    let healthy = state
        .entries
        .iter()
        .filter(|e| e.status == Status::Healthy)
        .count();
    let docker_count = state.entries.iter().filter(|e| e.docker.is_some()).count();
    let total_mem: f64 = state
        .entries
        .iter()
        .map(|e| e.memory_mb)
        .sum::<f64>()
        .max(0.0);
    Html(render_stats_cards(total, healthy, docker_count, total_mem))
}

// ─── API Endpoints ───

pub async fn api_ports(State(state): State<SharedState>) -> Json<Vec<crate::models::PortEntry>> {
    let state = state.lock().await;
    Json(state.entries.clone())
}

pub async fn api_port_detail(
    State(state): State<SharedState>,
    Path(port): Path<u16>,
) -> impl IntoResponse {
    let state = state.lock().await;
    match state.entries.iter().find(|e| e.port == port) {
        Some(entry) => Json(entry.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn api_kill_port(
    State(state): State<SharedState>,
    Path(port): Path<u16>,
) -> impl IntoResponse {
    let state = state.lock().await;
    match state.entries.iter().find(|e| e.port == port) {
        Some(entry) => match process::kill_process(entry, false) {
            Ok(()) => Json(serde_json::json!({"status": "ok", "message": format!("Killed PID {} on port {}", entry.pid, port)})).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"status": "error", "message": "Port not found"}))).into_response(),
    }
}

#[derive(Serialize)]
pub struct StatsResponse {
    total: usize,
    healthy: usize,
    docker: usize,
    memory_mb: f64,
}

pub async fn api_stats(State(state): State<SharedState>) -> Json<StatsResponse> {
    let state = state.lock().await;
    Json(StatsResponse {
        total: state.entries.len(),
        healthy: state
            .entries
            .iter()
            .filter(|e| e.status == Status::Healthy)
            .count(),
        docker: state.entries.iter().filter(|e| e.docker.is_some()).count(),
        memory_mb: state
            .entries
            .iter()
            .map(|e| e.memory_mb)
            .sum::<f64>()
            .max(0.0),
    })
}

// ─── Static Assets ───

pub async fn static_asset(Path(path): Path<String>) -> impl IntoResponse {
    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else if path.ends_with(".png") {
                "image/png"
            } else {
                "application/octet-stream"
            };

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// ─── HTML Rendering Helpers ───

fn render_stats_cards(total: usize, healthy: usize, docker: usize, memory: f64) -> String {
    format!(
        r#"<div class="stats-grid">
    <div class="stat-card">
        <div class="stat-icon stat-icon-total">🔌</div>
        <div class="stat-info">
            <span class="stat-value">{total}</span>
            <span class="stat-label">Active Ports</span>
        </div>
    </div>
    <div class="stat-card">
        <div class="stat-icon stat-icon-healthy">✓</div>
        <div class="stat-info">
            <span class="stat-value stat-healthy">{healthy}</span>
            <span class="stat-label">Healthy</span>
        </div>
    </div>
    <div class="stat-card">
        <div class="stat-icon stat-icon-docker">🐳</div>
        <div class="stat-info">
            <span class="stat-value stat-docker">{docker}</span>
            <span class="stat-label">Docker</span>
        </div>
    </div>
    <div class="stat-card">
        <div class="stat-icon stat-icon-memory">📊</div>
        <div class="stat-info">
            <span class="stat-value">{memory:.0}</span>
            <span class="stat-label">MB Total</span>
        </div>
    </div>
</div>"#
    )
}

fn render_port_table(entries: &[crate::models::PortEntry]) -> String {
    if entries.is_empty() {
        return r#"<div class="empty-state">
            <span class="empty-icon">🔍</span>
            <p>No active ports detected</p>
            <p class="empty-hint">Start a dev server to see it here</p>
        </div>"#
            .to_string();
    }

    let mut rows = String::new();
    for entry in entries {
        let status_class = match &entry.status {
            Status::Healthy => "status-healthy",
            Status::Warning(_) => "status-warning",
            Status::Zombie => "status-error",
            Status::Orphaned => "status-warning",
            Status::Unknown => "status-unknown",
        };

        let git_class = entry
            .git
            .as_ref()
            .map(|g| if g.dirty { "git-dirty" } else { "git-clean" })
            .unwrap_or("");

        rows.push_str(&format!(
            r#"<tr class="port-row" onclick="showDetail({port})">
                <td class="port-cell">{port}<span class="protocol">/{protocol}</span></td>
                <td class="pid-cell">{pid}</td>
                <td class="process-cell">{process}</td>
                <td class="project-cell">{project}</td>
                <td class="git-cell {git_class}">{git}</td>
                <td class="docker-cell">{docker}</td>
                <td class="uptime-cell">{uptime}</td>
                <td class="mem-cell">{mem:.0} MB</td>
                <td><span class="status-badge {status_class}">{status}</span></td>
                <td class="actions-cell">
                    <button class="btn-kill" onclick="event.stopPropagation(); killPort({port})"
                            title="Kill process">✕</button>
                </td>
            </tr>"#,
            port = entry.port,
            protocol = entry.protocol,
            pid = entry.pid,
            process = html_escape(&entry.process_name),
            project = html_escape(&entry.project_display()),
            git = html_escape(&entry.git_display()),
            git_class = git_class,
            docker = html_escape(&entry.docker_display()),
            uptime = entry.uptime_display(),
            mem = entry.memory_mb,
            status = entry.status,
            status_class = status_class,
        ));
    }

    format!(
        r#"<table class="port-table">
            <thead>
                <tr>
                    <th>Port</th>
                    <th>PID</th>
                    <th>Process</th>
                    <th>Project</th>
                    <th>Git</th>
                    <th>Docker</th>
                    <th>Uptime</th>
                    <th>Memory</th>
                    <th>Status</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
