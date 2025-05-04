use thiserror::Error;

/// Configuration Error Types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration property not found: {0}")]
    NotFound(String),
    #[error("Configuration parsing error: {0}")]
    ParseError(String),
    #[error("I/O error reading configuration: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Configuration source error: {0}")]
    SourceError(String),
    #[error("Other configuration error: {0}")]
    Other(String),
}
