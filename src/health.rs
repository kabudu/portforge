use crate::models::{HealthResult, HealthStatus};
use std::time::Instant;
use tracing::debug;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_nonexistent_port() {
        let result = check_health(59999, "/health", 500).await;
        assert_ne!(result.status, HealthStatus::Healthy);
    }
}
