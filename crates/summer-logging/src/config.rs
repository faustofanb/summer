use serde::Deserialize;
use std::collections::HashMap;

/// 日志系统主配置结构
/// 
/// 包含日志记录器和输出目标的基本配置
/// 
/// # Fields
/// 
/// * `loggers` - 日志记录器配置映射，键为模块名称，值为日志级别
/// * `appenders` - 输出目标配置映射，键为appender名称，值为目标配置
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    #[serde(default)]
    pub loggers: HashMap<String, String>,
    #[serde(default)]
    pub appenders: HashMap<String, AppenderConfig>,
}

/// 输出目标配置枚举
/// 
/// 支持控制台和文件两种输出方式
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AppenderConfig {
    Console(ConsoleAppenderConfig),
    File(FileAppenderConfig),
}

/// 控制台输出器配置
/// 
/// # Fields
/// 
/// * `target` - 输出目标（stdout/stderr）
/// * `encoder` - 日志消息编码配置
#[derive(Deserialize, Debug, Default, Clone)] // Derive Clone
#[serde(deny_unknown_fields)]
pub struct ConsoleAppenderConfig {
    #[serde(default = "default_stdout_target")]
    pub target: ConsoleTarget,
    pub encoder: EncoderConfig,
}
/// 供serde调用的默认函数
fn default_stdout_target() -> ConsoleTarget {
    ConsoleTarget::Stdout
}
/// 控制台输出目标枚举
/// 
/// # Variants
/// 
/// * `Stdout` - 标准输出（默认）
/// * `Stderr` - 标准错误输出
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default)] // Derive Clone and Copy
#[serde(rename_all = "lowercase")]
pub enum ConsoleTarget {
    #[default]
    Stdout,
    Stderr,
}
/// 文件输出器配置
/// 
/// # Fields
/// 
/// * `path` - 输出文件路径
/// * `encoder` - 日志消息编码配置
/// * `rolling_policy` - 滚动策略配置（可选）
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    pub path: String,
    pub encoder: EncoderConfig,
    pub rolling_policy: Option<RollingPolicyConfig>,
}

/// 滚动策略配置枚举
/// 
/// 支持时间滚动和大小+时间组合滚动两种策略
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RollingPolicyConfig {
    Time(TimeBasedRollingPolicy),
    SizeAndTime(SizeAndTimeBasedRollingPolicy),
}

/// 时间滚动策略配置
/// 
/// # Fields
/// 
/// * `file_name_pattern` - 滚动文件名模式（需包含%d日期占位符）
/// * `max_history` - 最大保留历史文件数
#[derive(Deserialize, Debug)]
pub struct TimeBasedRollingPolicy {
    pub file_name_pattern: String,
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

/// 大小+时间组合滚动策略配置
/// 
/// # Fields
/// 
/// * `file_name_pattern` - 滚动文件名模式（需包含%d日期和%i索引占位符）
/// * `max_file_size` - 单个文件最大尺寸（格式：数字+MB/GB）
/// * `max_history` - 最大保留历史文件数
#[derive(Deserialize, Debug)]
pub struct SizeAndTimeBasedRollingPolicy {
    pub file_name_pattern: String,
    pub max_file_size: String,
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

fn default_max_history() -> usize {
    7
}

/// 日志编码器配置枚举
/// 
/// # Variants
/// 
/// * `Pattern` - 模式编码器（默认）
/// * `Json` - JSON格式编码器
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EncoderConfig {
    Pattern(PatternEncoderConfig),
    Json(JsonEncoderConfig),
}

// 为 EncoderConfig 实现 Default
impl Default for EncoderConfig {
    fn default() -> Self {
        EncoderConfig::Pattern(PatternEncoderConfig {
            pattern: "%d{%Y-%m-%d %H:%M:%S} [%t] %l %T - %m%n".to_string(),
        })
    }
}

/// 模式编码器配置
/// 
/// # Fields
/// 
/// * `pattern` - 日志输出格式模式字符串
#[derive(Deserialize, Debug, Clone, Default)]
pub struct PatternEncoderConfig {
    pub pattern: String,
}

/// JSON编码器配置
/// 
/// # Fields
/// 
/// * `json_options` - JSON序列化选项（键值对形式）
#[derive(Deserialize, Debug, Clone, Default)]
pub struct JsonEncoderConfig {
    #[serde(flatten)]
    pub json_options: HashMap<String, serde_json::Value>,
}

// 添加一些辅助方法实现
impl LoggingConfig {
    /// 验证配置的有效性
    /// 
    /// 执行以下验证：
    /// 1. 检查所有日志记录器的级别是否有效
    /// 2. 验证各appender配置的完整性
    /// 
    /// # Returns
    /// 
    /// 如果配置有效返回Ok(()), 否则返回包含错误信息的Result::Err
    pub fn validate(&self) -> crate::Result<()> {
        // 验证每个 logger 的日志级别是否有效
        for (target, level) in &self.loggers {
            if !is_valid_level(level) {
                return Err(crate::LoggingError::InvalidLevel(format!(
                    "Invalid log level '{}' for target '{}'",
                    level, target
                )));
            }
        }

        // 验证每个 appender 的配置
        for (name, appender) in &self.appenders {
            match appender {
                AppenderConfig::File(file_config) => {
                    if let Some(policy) = &file_config.rolling_policy {
                        validate_rolling_policy(policy).map_err(|e| {
                            crate::LoggingError::ConfigParse(format!(
                                "Invalid rolling policy for appender '{}': {}",
                                name, e
                            ))
                        })?;
                    }
                }
                AppenderConfig::Console(_) => {
                    // 控制台配置相对简单，暂时不需要特殊验证
                }
            }
        }

        Ok(())
    }
}

/// 检查日志级别字符串是否有效
/// 
/// # Arguments
/// 
/// * `level` - 待验证的日志级别字符串
/// 
/// # Returns
/// 
/// 如果是有效的TRACE/DEBUG/INFO/WARN/ERROR级别返回true，否则false
fn is_valid_level(level: &str) -> bool {
    matches!(
        level.to_uppercase().as_str(),
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"
    )
}

/// 验证滚动策略配置的有效性
/// 
/// # Arguments
/// 
/// * `policy` - 要验证的滚动策略配置
/// 
/// # Returns
/// 
/// 如果验证通过返回Ok(()), 否则返回包含错误信息的Result::Err
fn validate_rolling_policy(policy: &RollingPolicyConfig) -> crate::Result<()> {
    match policy {
        RollingPolicyConfig::Time(time_policy) => {
            if !time_policy.file_name_pattern.contains("%d") {
                return Err(crate::LoggingError::InvalidRollingPolicy(
                    "Time-based rolling policy must contain %d in file_name_pattern".to_string(),
                ));
            }
        }
        RollingPolicyConfig::SizeAndTime(size_time_policy) => {
            if !size_time_policy.file_name_pattern.contains("%d")
                || !size_time_policy.file_name_pattern.contains("%i")
            {
                return Err(crate::LoggingError::InvalidRollingPolicy(
                    "Size and time based rolling policy must contain both %d and %i in file_name_pattern"
                        .to_string(),
                ));
            }

            // 验证文件大小格式
            if !is_valid_size_format(&size_time_policy.max_file_size) {
                return Err(crate::LoggingError::InvalidRollingPolicy(format!(
                    "Invalid max_file_size format: {}. Expected format: <number>MB or <number>GB",
                    size_time_policy.max_file_size
                )));
            }
        }
    }
    Ok(())
}

/// 验证文件大小格式是否符合要求
/// 
/// # Arguments
/// 
/// * `size` - 文件大小字符串（如"10MB"）
/// 
/// # Returns
/// 
/// 如果格式正确且数值有效返回true，否则false
fn is_valid_size_format(size: &str) -> bool {
    let size = size.trim().to_uppercase();
    if let Some(num_str) = size.strip_suffix("MB") {
        num_str.parse::<u64>().is_ok()
    } else if let Some(num_str) = size.strip_suffix("GB") {
        num_str.parse::<u64>().is_ok()
    } else {
        false
    }
}
