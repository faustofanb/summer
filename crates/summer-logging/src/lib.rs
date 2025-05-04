//! summer-logging 日志模块主入口，提供日志初始化与常用宏导出。

mod config;
mod error;
mod init;
mod pattern;

pub use config::*;
pub use error::LoggingError;
pub use init::init;

/// 重新导出 tracing 的核心功能，让用户可以直接从 summer_logging 使用
pub use tracing::{debug, error, info, trace, warn};

/// Result type for summer-logging operations
pub type Result<T> = std::result::Result<T, LoggingError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::init::init_default;

    #[tokio::test]
    async fn  test_default_logging() {
        init_default().expect("Failed to initialize logging");

        info!(target: "test", "这是一条信息日志");
        warn!("这是一条警告日志");
        debug!("这是一条调试日志, 不会显示");
    }
    
    /// 测试基础日志输出，包含 info/warn/debug 级别
    #[tokio::test]
    async fn test_basic_logging() {
        // 创建测试配置
        let mut config = LoggingConfig::default();

        // 添加控制台 appender，使用自定义格式
        let console_appender = AppenderConfig::Console(ConsoleAppenderConfig {
            target: ConsoleTarget::Stdout,
            encoder: EncoderConfig::Pattern(PatternEncoderConfig {
                pattern: "%d{yyyy-MM-dd HH:mm:ss} [%t]  %c @ %M [ %p] %m %n".to_string(),
            }),
        });

        config
            .appenders
            .insert("console".to_string(), console_appender);

        // 设置日志级别，仅 target 为 "test" 的日志会输出 debug 及以上
        config
            .loggers
            .insert("test".to_string(), "debug".to_string());

        // 初始化日志系统
        init(&config).expect("Failed to initialize logging");

        // 输出一些测试日志
        info!("这是一条信息日志");
        warn!("这是一条警告日志");
        debug!("这是一条调试日志, 不会显示");
        debug!(target: "test", "这是一条调试日志"); // 需指定 target 才能输出
    }

    /// 测试配置校验逻辑，包括无效日志级别和滚动策略
    #[test]
    fn test_config_validation() {
        let mut config = LoggingConfig::default();

        // 测试无效的日志级别
        config
            .loggers
            .insert("test".to_string(), "INVALID_LEVEL".to_string());
        assert!(config.validate().is_err());

        // 测试有效的配置
        config.loggers.clear();
        config
            .loggers
            .insert("test".to_string(), "DEBUG".to_string());
        assert!(config.validate().is_ok());

        // 测试无效的滚动策略配置（缺少 %d）
        let file_appender = AppenderConfig::File(FileAppenderConfig {
            path: "test.log".to_string(),
            encoder: EncoderConfig::Pattern(PatternEncoderConfig {
                pattern: "%m%n".to_string(),
            }),
            rolling_policy: Some(RollingPolicyConfig::Time(TimeBasedRollingPolicy {
                file_name_pattern: "test.log".to_string(), // 缺少 %d
                max_history: 7,
            })),
        });
        config.appenders.insert("file".to_string(), file_appender);
        assert!(config.validate().is_err());
    }

    /// 测试 JSON 编码器和 stderr 输出
    #[tokio::test]
    async fn test_json_encoder_stderr() {
        // 创建测试配置
        let mut config = LoggingConfig {
            appenders: HashMap::new(),
            loggers: HashMap::new(),
        };

        // 添加使用 JSON 编码器并输出到 stderr 的控制台 appender
        let console_appender = AppenderConfig::Console(ConsoleAppenderConfig {
            target: ConsoleTarget::Stderr, // 输出到 stderr
            encoder: EncoderConfig::Json(JsonEncoderConfig {
                json_options: HashMap::new(),
            }), // 使用 JSON 编码器
        });
        config
            .appenders
            .insert("console_stderr_json".to_string(), console_appender);

        // 设置特定 logger 的日志级别为 trace
        config
            .loggers
            .insert("test_json".to_string(), "trace".to_string());

        // 初始化日志系统
        // 注意：由于 tracing 全局状态，并行测试可能会相互干扰。
        // 实际项目中可能需要使用 serial_test 或类似库来确保测试串行执行。
        init(&config).expect("Failed to initialize logging for json test");

        // 输出一些测试日志（应以 JSON 格式输出到 stderr）
        tracing::trace!(target: "test_json", "这是一条 JSON trace 日志");
        tracing::debug!(target: "test_json", "这是一条 JSON debug 日志");
        tracing::info!(target: "test_json", "这是一条 JSON info 日志");
        tracing::warn!(target: "test_json", "这是一条 JSON warn 日志");
        tracing::error!(target: "test_json", "这是一条 JSON error 日志");

        // 注意：此测试主要验证初始化和日志调用不 panic。
        // 验证 stderr 的具体 JSON 输出通常需要更复杂的测试设置（例如，捕获 stderr）。
    }
}
