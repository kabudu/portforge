use crate::models::{HealthResult, HealthStatus};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tracing::debug;

/// Health check protocol types.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthCheckType {
    /// Standard HTTP health check.
    Http,
    /// gRPC health check (TCP connection check).
    Grpc,
    /// WebSocket health check (TCP connection check).
    WebSocket,
}

/// Perform an HTTP health check on a given port and endpoint.
pub async fn check_health(port: u16, endpoint: &str, timeout_ms: u64) -> HealthResult {
    let url = format!("http://127.0.0.1:{}{}", port, endpoint);
    debug!("Health check: {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .danger_accept_invalid_certs(true)
        .no_proxy()
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_e) => {
            return HealthResult {
                status: HealthStatus::Unknown,
                status_code: None,
                latency_ms: 0,
                endpoint: endpoint.to_string(),
            };
        }
    };

    let start = Instant::now();

    match client.get(&url).send().await {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;
            let status_code = response.status().as_u16();

            let health_status = if (200..400).contains(&status_code) {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            };

            debug!("Health check {}: {} ({}ms)", url, status_code, latency);

            HealthResult {
                status: health_status,
                status_code: Some(status_code),
                latency_ms: latency,
                endpoint: endpoint.to_string(),
            }
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!("Health check {} failed: {} ({}ms)", url, e, latency);

            // Connection refused is different from timeout
            let status = if e.is_connect() {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Unknown
            };

            HealthResult {
                status,
                status_code: None,
                latency_ms: latency,
                endpoint: endpoint.to_string(),
            }
        }
    }
}

/// Perform a gRPC health check (TCP connection check).
/// This checks if the gRPC service is accepting connections.
pub async fn check_grpc_health(port: u16, timeout_ms: u64) -> HealthResult {
    debug!("gRPC health check on port {}", port);
    let start = Instant::now();

    let addr = format!("127.0.0.1:{}", port);
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!("gRPC health check port {}: healthy ({}ms)", port, latency);
            HealthResult {
                status: HealthStatus::Healthy,
                status_code: None,
                latency_ms: latency,
                endpoint: "gRPC".to_string(),
            }
        }
        Ok(Err(e)) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!(
                "gRPC health check port {}: failed {} ({}ms)",
                port, e, latency
            );
            HealthResult {
                status: HealthStatus::Unhealthy,
                status_code: None,
                latency_ms: latency,
                endpoint: "gRPC".to_string(),
            }
        }
        Err(_) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!("gRPC health check port {}: timeout ({}ms)", port, latency);
            HealthResult {
                status: HealthStatus::Unknown,
                status_code: None,
                latency_ms: latency,
                endpoint: "gRPC".to_string(),
            }
        }
    }
}

/// Perform a WebSocket health check (TCP connection check).
/// This checks if the WebSocket service is accepting connections.
pub async fn check_websocket_health(port: u16, timeout_ms: u64) -> HealthResult {
    debug!("WebSocket health check on port {}", port);
    let start = Instant::now();

    let addr = format!("127.0.0.1:{}", port);
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!(
                "WebSocket health check port {}: healthy ({}ms)",
                port, latency
            );
            HealthResult {
                status: HealthStatus::Healthy,
                status_code: None,
                latency_ms: latency,
                endpoint: "WebSocket".to_string(),
            }
        }
        Ok(Err(e)) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!(
                "WebSocket health check port {}: failed {} ({}ms)",
                port, e, latency
            );
            HealthResult {
                status: HealthStatus::Unhealthy,
                status_code: None,
                latency_ms: latency,
                endpoint: "WebSocket".to_string(),
            }
        }
        Err(_) => {
            let latency = start.elapsed().as_millis() as u64;
            debug!(
                "WebSocket health check port {}: timeout ({}ms)",
                port, latency
            );
            HealthResult {
                status: HealthStatus::Unknown,
                status_code: None,
                latency_ms: latency,
                endpoint: "WebSocket".to_string(),
            }
        }
    }
}

/// Perform a health check based on the specified type.
pub async fn check_health_typed(
    port: u16,
    check_type: HealthCheckType,
    endpoint: &str,
    timeout_ms: u64,
) -> HealthResult {
    match check_type {
        HealthCheckType::Http => check_health(port, endpoint, timeout_ms).await,
        HealthCheckType::Grpc => check_grpc_health(port, timeout_ms).await,
        HealthCheckType::WebSocket => check_websocket_health(port, timeout_ms).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_nonexistent_port() {
        let result = check_health(59999, "/health", 500).await;
        assert_ne!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_grpc_health_check_nonexistent_port() {
        let result = check_grpc_health(59998, 500).await;
        assert_ne!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_websocket_health_check_nonexistent_port() {
        let result = check_websocket_health(59997, 500).await;
        assert_ne!(result.status, HealthStatus::Healthy);
    }
}
