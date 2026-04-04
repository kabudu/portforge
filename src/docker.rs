use crate::models::DockerInfo;
use std::collections::HashMap;
use tracing::debug;

/// Get a map of host port → DockerInfo for all running containers.
pub async fn get_container_port_map() -> Result<HashMap<u16, DockerInfo>, String> {
    let docker = match bollard::Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            debug!("Docker not available: {}", e);
            return Err(format!("Docker connection failed: {}", e));
        }
    };

    // Verify Docker is reachable
    if let Err(e) = docker.ping().await {
        debug!("Docker ping failed: {}", e);
        return Err(format!("Docker not responding: {}", e));
    }

    let mut map = HashMap::new();

    let options = bollard::container::ListContainersOptions::<String> {
        all: false, // only running containers
        ..Default::default()
    };

    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    for container in containers {
        let container_name = container
            .names
            .as_ref()
            .and_then(|n| n.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let image = container.image.clone().unwrap_or_default();

        let container_id = container
            .id
            .clone()
            .unwrap_or_default();

        // Extract compose project from labels
        let compose_project = container
            .labels
            .as_ref()
            .and_then(|labels| labels.get("com.docker.compose.project"))
            .cloned();

        // Map all published ports
        if let Some(ports) = &container.ports {
            for port in ports {
                if let Some(public_port) = port.public_port {
                    let info = DockerInfo {
                        container_name: container_name.clone(),
                        image: image.clone(),
                        compose_project: compose_project.clone(),
                        container_id: container_id.clone(),
                    };
                    map.insert(public_port, info);
                }
            }
        }
    }

    debug!("Found {} Docker port mappings", map.len());
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_connection_graceful_failure() {
        // This test verifies that Docker integration fails gracefully
        // when Docker is not available (common in CI environments).
        let result = get_container_port_map().await;
        // We don't assert success/failure — just that it doesn't panic
        match result {
            Ok(map) => {
                println!("Docker available, found {} mappings", map.len());
            }
            Err(e) => {
                println!("Docker not available (expected in CI): {}", e);
            }
        }
    }
}
