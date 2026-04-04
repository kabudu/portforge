use thiserror::Error;

/// Central error type for PortForge operations.
#[derive(Error, Debug)]
pub enum PortForgeError {
    #[error("Port scan failed: {0}")]
    ScanError(String),

    #[error("Process operation failed: {0}")]
    ProcessError(String),

    #[error("Docker connection failed: {0}")]
    DockerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Health check failed for port {port}: {message}")]
    HealthCheckError { port: u16, message: String },

    #[error("Export failed: {0}")]
    ExportError(String),

    #[error("TUI error: {0}")]
    TuiError(String),

    #[cfg(feature = "web")]
    #[error("Web server error: {0}")]
    WebError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenience Result alias for PortForge operations.
pub type Result<T> = std::result::Result<T, PortForgeError>;
