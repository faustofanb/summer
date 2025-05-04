//! 日志初始化与控制台输出实现。
//!
//! 提供 init() 入口，支持多种格式和目标，自动适配 tracing-subscriber。

use crate::{config::*, LoggingError};
use std::io::{self, Write};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::SystemTime},
    prelude::*,
    registry::Registry,
    EnvFilter,
};

/// 初始化日志系统，根据配置自动选择控制台输出格式和目标。
///
/// - 支持 Pattern 和 Json 编码器
/// - 支持 stdout/stderr 目标
/// - 自动启用 ANSI 颜色
///
/// # 参数
/// * `config` - 日志配置对象
///
/// # 返回
/// * `Result<(), LoggingError>` - 初始化成功或失败
pub fn init(config: &LoggingConfig) -> Result<(), LoggingError> {
    // 构建环境过滤器，优先使用 RUST_LOG 环境变量，否则默认 info
    let mut env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| LoggingError::ConfigParse(e.to_string()))?;
    for (target, level) in &config.loggers {
        let directive = format!("{}={}", target, level);
        env_filter = env_filter.add_directive(directive.parse().map_err(|e| {
            LoggingError::ConfigParse(format!("Invalid log directive '{}': {}", directive, e))
        })?);
    }

    // 构建基础 subscriber
    let subscriber = Registry::default().with(env_filter);

    // 查找第一个控制台 appender 配置，若无则使用默认
    let console_config = config
        .appenders
        .values()
        .find_map(|appender| match appender {
            AppenderConfig::Console(config) => Some(config),
            AppenderConfig::File(_) => None,
        })
        .cloned()
        .unwrap_or_default();

    // 构造 writer 闭包，支持 stdout/stderr
    let target = console_config.target;
    let make_writer = move || match target {
        ConsoleTarget::Stdout => ConsoleWriter::Stdout(io::stdout()),
        ConsoleTarget::Stderr => ConsoleWriter::Stderr(io::stderr()),
    };

    // 构造基础 Layer，启用颜色、线程、文件、行号等
    let base_layer = fmt::layer()
        .with_ansi(true) // 强制启用 ANSI 颜色
        .with_writer(make_writer)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_timer(SystemTime)
        .with_span_events(FmtSpan::CLOSE); // 记录 span 关闭事件

    // 根据编码器类型静态分支，保证 Layer 类型具体
    match console_config.encoder {
        EncoderConfig::Pattern(pattern_config) => {
            // Pattern 格式化输出
            let layer = base_layer.event_format(crate::pattern::PatternFormatter::new(
                pattern_config.pattern,
            ));
            let final_subscriber = subscriber.with(layer);
            tracing::subscriber::set_global_default(final_subscriber)
                .map_err(LoggingError::SetGlobalDefault)?;
        }
        EncoderConfig::Json(_) => {
            // JSON 格式化输出
            let layer = base_layer.json();
            let final_subscriber = subscriber.with(layer);
            tracing::subscriber::set_global_default(final_subscriber)
                .map_err(LoggingError::SetGlobalDefault)?;
        }
    }

    Ok(())
}

pub fn init_default() -> Result<(), LoggingError> {
    let mut config = LoggingConfig::default();

    // 添加控制台 appender，使用自定义格式
    let console_appender = AppenderConfig::Console(ConsoleAppenderConfig {
        target: ConsoleTarget::Stdout,
        encoder: EncoderConfig::Pattern(PatternEncoderConfig {
            pattern: "%d{%Y-%m-%d %H:%M:%S} [%t] %l %T - %m%n".to_string(),
        }),
    });

    config
        .appenders
        .insert("console".to_string(), console_appender);

    // 初始化日志系统
    init(&config).expect("Failed to initialize logging");
    Ok(())
}
/// 控制台输出包装，支持 stdout/stderr。
enum ConsoleWriter {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
}

impl Write for ConsoleWriter {
    /// 写入字节到目标流
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            ConsoleWriter::Stdout(out) => out.write(buf),
            ConsoleWriter::Stderr(err) => err.write(buf),
        }
    }

    /// 刷新目标流
    fn flush(&mut self) -> io::Result<()> {
        match self {
            ConsoleWriter::Stdout(out) => out.flush(),
            ConsoleWriter::Stderr(err) => err.flush(),
        }
    }
}
