use crate::config::PortForgeConfig;
use crate::error::{PortForgeError, Result};
use crate::web::handlers;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state for the web server.
pub struct AppState {
    pub config: PortForgeConfig,
    pub entries: Vec<crate::models::PortEntry>,
}

pub type SharedState = Arc<Mutex<AppState>>;

/// Start the web dashboard server.
pub async fn start_server(bind: &str, port: u16, config: PortForgeConfig) -> Result<()> {
    if !is_loopback_bind(bind) {
        return Err(PortForgeError::WebError(format!(
            "Refusing to bind web dashboard to non-loopback address '{bind}'. Bind to 127.0.0.1, ::1, or localhost."
        )));
    }

    let state: SharedState = Arc::new(Mutex::new(AppState {
        config: config.clone(),
        entries: Vec::new(),
    }));

    // Initial scan
    {
        let mut state_lock = state.lock().await;
        match crate::scanner::scan_ports(&state_lock.config, false).await {
            Ok(entries) => state_lock.entries = entries,
            Err(e) => tracing::warn!("Initial scan failed: {}", e),
        }
    }

    // Background scanner task
    let scanner_state = state.clone();
    let scanner_config = config.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            scanner_config.general.refresh_interval.max(2),
        ));
        loop {
            interval.tick().await;
            match crate::scanner::scan_ports(&scanner_config, false).await {
                Ok(entries) => {
                    let mut state_lock = scanner_state.lock().await;
                    state_lock.entries = entries;
                }
                Err(e) => tracing::warn!("Background scan failed: {}", e),
            }
        }
    });

    let app = Router::new()
        // Pages
        .route("/", get(handlers::dashboard_page))
        // API endpoints
        .route("/api/ports", get(handlers::api_ports))
        .route("/api/ports/{port}", get(handlers::api_port_detail))
        .route("/api/ports/{port}/kill", post(handlers::api_kill_port))
        .route("/api/stats", get(handlers::api_stats))
        // HTMX partials
        .route("/partials/table", get(handlers::partial_table))
        .route("/partials/stats", get(handlers::partial_stats))
        // Static assets
        .route("/static/{*path}", get(handlers::static_asset))
        .with_state(state);

    let addr = format!("{}:{}", bind, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| PortForgeError::WebError(format!("Failed to bind to {}: {}", addr, e)))?;

    tracing::info!("Web dashboard running at http://{}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| PortForgeError::WebError(e.to_string()))?;

    Ok(())
}

fn is_loopback_bind(bind: &str) -> bool {
    bind.eq_ignore_ascii_case("localhost")
        || bind
            .parse::<IpAddr>()
            .map(|address| address.is_loopback())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::is_loopback_bind;

    #[test]
    fn test_loopback_bind_allows_local_addresses() {
        assert!(is_loopback_bind("127.0.0.1"));
        assert!(is_loopback_bind("::1"));
        assert!(is_loopback_bind("localhost"));
    }

    #[test]
    fn test_loopback_bind_rejects_remote_addresses() {
        assert!(!is_loopback_bind("0.0.0.0"));
        assert!(!is_loopback_bind("::"));
        assert!(!is_loopback_bind("192.168.1.10"));
    }
}
