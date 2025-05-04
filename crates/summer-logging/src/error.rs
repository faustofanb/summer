//! 日志模块错误类型定义。
//!
//! 统一封装日志配置、IO、全局订阅器等相关错误。

use thiserror::Error;

/// 日志相关错误类型。
#[derive(Debug, Error)]
pub enum LoggingError {
    /// 日志配置解析失败
    #[error("Failed to parse logging configuration: {0}")]
    ConfigParse(String),

    /// 日志写入器创建失败
    #[error("Failed to create log writer: {0}")]
    WriterCreation(String),

    /// 设置全局 tracing 订阅器失败
    #[error("Failed to set global subscriber: {0}")]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),

    /// 日志级别无效
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),

    /// 滚动策略配置无效
    #[error("Invalid rolling policy configuration: {0}")]
    InvalidRollingPolicy(String),

    /// 文件系统相关错误
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// 其他内部错误
    #[error("Internal error: {0}")]
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

