use crate::error::{PortForgeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// PortForge configuration loaded from `~/.config/portforge.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PortForgeConfig {
    /// General settings.
    pub general: GeneralConfig,

    /// Health check configuration.
    pub health: HealthConfig,

    /// Custom project detectors.
    pub detectors: Vec<CustomDetector>,

    /// Per-port overrides.
    pub ports: HashMap<u16, PortOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default refresh interval for watch mode (seconds).
    pub refresh_interval: u64,

    /// Show all ports including non-dev (default: false).
    pub show_all: bool,

    /// Enable Docker integration.
    pub docker_enabled: bool,

    /// Enable health checks.
    pub health_checks_enabled: bool,

    /// Maximum concurrent health checks.
    pub max_concurrent_health_checks: usize,

    /// Color theme: "dark" or "light".
    pub theme: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 2,
            show_all: false,
            docker_enabled: true,
            health_checks_enabled: true,
            max_concurrent_health_checks: 10,
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthConfig {
    /// Default timeout for health checks (milliseconds).
    pub timeout_ms: u64,

    /// Default health check endpoints to try.
    pub default_endpoints: Vec<String>,

    /// Framework-specific health endpoints.
    pub framework_endpoints: HashMap<String, String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        let mut framework_endpoints = HashMap::new();
        framework_endpoints.insert("next.js".to_string(), "/api/health".to_string());
        framework_endpoints.insert("express".to_string(), "/health".to_string());
        framework_endpoints.insert("actix".to_string(), "/health".to_string());
        framework_endpoints.insert("axum".to_string(), "/health".to_string());
        framework_endpoints.insert("fastapi".to_string(), "/health".to_string());
        framework_endpoints.insert("rails".to_string(), "/up".to_string());
        framework_endpoints.insert("django".to_string(), "/health/".to_string());
        framework_endpoints.insert("spring".to_string(), "/actuator/health".to_string());

        Self {
            timeout_ms: 2000,
            default_endpoints: vec![
                "/health".to_string(),
                "/healthz".to_string(),
                "/api/health".to_string(),
                "/".to_string(),
            ],
            framework_endpoints,
        }
    }
}

/// Custom project detector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDetector {
    /// Name of the project kind.
    pub kind: String,
    /// Framework name.
    pub framework: String,
    /// Files to look for in the project directory.
    pub detect_files: Vec<String>,
    /// Optional health endpoint.
    pub health_endpoint: Option<String>,
}

/// Per-port configuration override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortOverride {
    /// Custom label for this port.
    pub label: Option<String>,
    /// Custom health endpoint.
    pub health_endpoint: Option<String>,
    /// Whether to hide this port from default view.
    pub hidden: bool,
}

impl PortForgeConfig {
    /// Load configuration from the default path.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| {
                PortForgeError::ConfigError(format!(
                    "Failed to read config at {}: {}",
                    path.display(),
                    e
                ))
            })?;
            let config: PortForgeConfig = toml::from_str(&content).map_err(|e| {
                PortForgeError::ConfigError(format!("Failed to parse config: {}", e))
            })?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Write default configuration to disk.
    pub fn write_default() -> Result<PathBuf> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PortForgeError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let default_config = Self::default();
        let content = toml::to_string_pretty(&default_config).map_err(|e| {
            PortForgeError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        let header = r#"# PortForge Configuration
# Location: ~/.config/portforge.toml
# Documentation: https://github.com/kabudu/portforge#configuration

"#;

        std::fs::write(&path, format!("{}{}", header, content))
            .map_err(|e| PortForgeError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(path)
    }

    /// Get the configuration file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("portforge.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PortForgeConfig::default();
        assert_eq!(config.general.refresh_interval, 2);
        assert!(!config.general.show_all);
        assert!(config.general.docker_enabled);
        assert_eq!(config.health.timeout_ms, 2000);
        assert!(!config.health.default_endpoints.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = PortForgeConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: PortForgeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            parsed.general.refresh_interval,
            config.general.refresh_interval
        );
    }
}
