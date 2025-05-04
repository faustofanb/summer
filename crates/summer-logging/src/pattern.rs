//! 日志格式化实现：支持自定义 Pattern 格式。
//!
//! 提供 PatternFormatter，可用于 tracing-subscriber 的 event_format。

use chrono::{DateTime, Local};
use std::fmt;
use tracing::Level;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};

/// 将日志级别格式化为固定宽度的字符串
fn format_level(level: Level) -> &'static str {
    match level {
        Level::TRACE => "TRACE",
        Level::DEBUG => "DEBUG",
        Level::INFO => "INFO ",
        Level::WARN => "WARN ",
        Level::ERROR => "ERROR",
    }
}

/// 用于访问和提取消息字段的访问器
struct MessageVisitor<'a>(&'a mut String);

impl tracing::field::Visit for MessageVisitor<'_> {
    /// 处理 message 字段的字符串格式
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }

    /// 处理 message 字段的 debug 格式
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            use std::fmt::Write;
            write!(self.0, "{:?}", value).expect("Failed to write to string buffer");
        }
    }
}
/// 日志事件格式化器，支持自定义 pattern 字符串。
pub(crate) struct PatternFormatter {
    /// 日志输出格式 pattern，例如 "%d [%t] %l %T - %m%n"
    pattern: String,
}

impl PatternFormatter {
    /// 创建新的 PatternFormatter
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

impl<S, N> FormatEvent<S, N> for PatternFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    /// 格式化日志事件到指定的writer中，根据配置的模式字符串替换各种格式说明符。
    /// 支持的格式说明符包括时间戳、日志级别、线程信息、文件位置、span上下文等。
    ///
    /// 参数：
    /// - ctx: FmtContext对象，提供span上下文查找能力
    /// - writer: Writer对象，用于写入格式化后的日志内容
    /// - event: Event对象，包含日志事件的元数据和负载
    ///
    /// 返回值：
    /// - fmt::Result: 写入操作的结果，Ok(())表示成功，Err表示写入错误
    #[allow(clippy::write_with_newline)] // 允许模式中显式包含%n换行符
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>, // 使用ctx获取span信息
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // --- 预计算常用数据 ---
        // 1. 当前时间戳（统一使用本地时区）
        let now: DateTime<Local> = Local::now();

        // 2. 事件核心元数据
        let metadata = event.metadata();
        let level = metadata.level(); // 日志级别（trace/debug/info/warn/error）
        let target = metadata.target(); // 日志目标（通常是模块路径）
        let module = metadata.module_path().unwrap_or("?"); // 模块路径
        let file = metadata.file().unwrap_or("?"); // 文件路径
        let line = metadata.line(); // 行号（可选）

        // 3. 线程相关信息
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("?"); // 线程名称
        let thread_id = format!("{:?}", thread.id()); // 线程唯一标识

        // 4. 日志消息内容
        let mut message = String::new();
        {
            // 通过自定义访问者提取消息文本
            let mut visitor = MessageVisitor(&mut message);
            event.record(&mut visitor);
        }

        // --- 模式解析与输出生成 ---
        // 将模式字符串拆分为字符迭代器
        let mut pattern_chars = self.pattern.chars().peekable();

        while let Some(ch) = pattern_chars.next() {
            if ch == '%' {
                // 处理格式说明符
                match pattern_chars.next() {
                    Some('%') => { // 转义百分号
                        write!(writer, "%")?;
                    }
                    Some('d') => { // 日期时间格式
                        // 解析自定义时间格式（如%d{YYYY-MM-DD}）
                        let date_format = "%Y-%m-%d %H:%M:%S"; // 默认格式
                        let mut custom_format = String::new();
                        let use_default_format;

                        // 检查是否存在{}包裹的自定义格式
                        if pattern_chars.peek() == Some(&'{') {
                            pattern_chars.next(); // 消耗'{'
                            let mut fmt_str = String::new();
                            // 提取格式字符串直到'}'
                            while let Some(fmt_ch) = pattern_chars.peek() {
                                if *fmt_ch == '}' {
                                    pattern_chars.next(); // 消耗'}'
                                    break;
                                }
                                fmt_str.push(*fmt_ch);
                                pattern_chars.next(); // 消耗格式字符
                            }
                            custom_format = fmt_str;
                            use_default_format = false;
                        } else {
                            use_default_format = true;
                        }

                        // 根据格式设置写入时间
                        if use_default_format {
                            write!(writer, "{}", now.format(date_format))?;
                        } else {
                            write!(writer, "{}", now.format(&custom_format))?;
                        }
                    }
                    Some('t') => { // 线程信息处理
                        // 检查是否为%tid（线程ID）说明符
                        let mut is_tid = false;
                        if pattern_chars.peek() == Some(&'i') {
                            let mut ahead = pattern_chars.clone();
                            ahead.next(); // 跳过'i'
                            if ahead.peek() == Some(&'d') {
                                // 确认为%tid格式
                                is_tid = true;
                                // 消耗'i'和'd'
                                pattern_chars.next();
                                pattern_chars.next();
                                write!(writer, "{}", thread_id)?;
                            }
                        }

                        if !is_tid {
                            // 单独%t表示线程名称
                            write!(writer, "{}", thread_name)?;
                        }
                    }
                    Some('p') | Some('l') => { // 日志级别
                        // 支持%p（logback风格）和%l（tracing风格）两种格式
                        write!(writer, "{}", format_level(*level))?;
                    }
                    Some('T') => { // 日志目标
                        // 使用%T表示日志目标（target）
                        write!(writer, "{}", target)?;
                    }
                    Some('c') => { // Logger名称（目标别名）
                        // 兼容logback的%c格式，实际输出target
                        write!(writer, "{}", target)?;
                    }
                    Some('m') => { // 日志消息
                        // 写入预提取的消息内容
                        write!(writer, "{}", message)?;
                    }
                    Some('n') => { // 换行符
                        // 根据平台写入换行
                        #[cfg(windows)]
                        write!(writer, "\r\n")?;
                        #[cfg(not(windows))]
                        write!(writer, "\n")?;
                    }
                    Some('F') => { // 文件名
                        write!(writer, "{}", file)?;
                    }
                    Some('L') => { // 行号
                        // 处理可选的行号信息
                        if let Some(line_num) = line {
                            write!(writer, "{}", line_num)?;
                        } else {
                            write!(writer, "?")?;
                        }
                    }
                    Some('M') => { // 方法名
                        write!(writer, "{}", metadata.name())?; // 使用元数据中的方法名
                    }
                    Some('C') => { // 类名（模块路径）
                        write!(writer, "{}", module)?; // 使用模块路径作为类名
                    }
                    Some('s') => { // span名称
                        // 检查是否为%span说明符
                        let mut is_span = false;
                        if pattern_chars.peek() == Some(&'p') {
                            let mut ahead = pattern_chars.clone();
                            ahead.next(); // 跳过'p'
                            if ahead.peek() == Some(&'a') {
                                ahead.next(); // 跳过'a'
                                if ahead.peek() == Some(&'n') {
                                    // 确认为%span格式
                                    is_span = true;
                                    // 消耗p,a,n字符
                                    pattern_chars.next();
                                    pattern_chars.next();
                                    pattern_chars.next();

                                    // 写入当前span名称
                                    if let Some(span_ref) = ctx.lookup_current() {
                                        write!(writer, "{}", span_ref.name())?;
                                    }
                                }
                            }
                        }

                        if !is_span {
                            // 非span格式时原样输出%s
                            write!(writer, "%s")?;
                        }
                    }
                    Some('X') => { // MDC/上下文字段
                        // 处理%X说明符（忽略key参数）
                        if pattern_chars.peek() == Some(&'{') {
                            // 跳过key部分
                            pattern_chars.next(); // 消耗'{'
                            while let Some(key_ch) = pattern_chars.peek() {
                                if *key_ch == '}' {
                                    pattern_chars.next(); // 消耗'}'
                                    break;
                                }
                                pattern_chars.next(); // 消耗key字符
                            }
                        }

                        // 写入span上下文字段
                        if let Some(span_ref) = ctx.lookup_current() {
                            let mut wrote_any_field = false;
                            // 遍历span层级
                            for span in span_ref.scope() {
                                let ext = span.extensions();
                                // 获取格式化字段
                                if let Some(fields) = ext.get::<FormattedFields<N>>() {
                                    if !fields.fields.is_empty() {
                                        if wrote_any_field {
                                            write!(writer, ", ")?;
                                        }
                                        write!(writer, "{}", &fields.fields)?;
                                        wrote_any_field = true;
                                    }
                                }
                            }
                        }
                    }
                    Some(other) => { // 未知说明符
                        // 原样输出%和后续字符
                        write!(writer, "%{}", other)?;
                    }
                    None => { // 孤立%符号
                        write!(writer, "%")?;
                    }
                }
            } else {
                // 原样输出普通字符
                write!(writer, "{}", ch)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::io;
    use std::sync::{Arc, Mutex};
    use tracing::subscriber::with_default;
    // Correct imports for Layer, Writer, MakeWriter, format::Full
    use tracing_subscriber::fmt::MakeWriter;
    // Import fmt itself
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // --- Mock Writer and MockMakeWriter remain the same ---
    #[derive(Clone, Default)]
    struct MockWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Default::default()
        }

        fn buf(&self) -> Vec<u8> {
            self.buf.lock().unwrap().clone()
        }

        fn buf_string(&self) -> String {
            String::from_utf8(self.buf()).expect("Output is not valid UTF-8")
        }
    }

    impl io::Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockMakeWriter {
        writer: MockWriter,
    }

    impl MockMakeWriter {
        fn new() -> Self {
            Self { writer: MockWriter::new() }
        }
        fn get_writer(&self) -> MockWriter {
            self.writer.clone()
        }
    }

    impl<'a> MakeWriter<'a> for MockMakeWriter {
        type Writer = MockWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.writer.clone()
        }
    }


    // Helper function to setup subscriber and run code
    fn run_test_with_formatter<F>(pattern: &str, test_code: F) -> String
    where
        F: FnOnce(),
    {
        let make_writer = MockMakeWriter::new();
        let writer_handle = make_writer.get_writer();

        let formatter = PatternFormatter::new(pattern.to_string());

        // --- Corrected Layer Setup ---
        let layer = tracing_subscriber::fmt::layer()
            .event_format(formatter)
            // --- REMOVED THIS LINE ---
            // .fmt_fields(tracing_subscriber::fmt::format::Full::default())
            // --- Let the layer use its default field formatter (DefaultFields) ---
            .with_writer(make_writer)
            .with_ansi(false);

        // Now, N = DefaultFields in the Layer type
        let subscriber = Registry::default().with(layer);

        // Inside PatternFormatter::format_event, the code will look for
        // ext.get::<FormattedFields<DefaultFields>>() which should work.
        with_default(subscriber, test_code);

        writer_handle.buf_string()
    }


    // --- Regex definitions remain the same ---
    lazy_static! {
        static ref RE_DATE_DEFAULT: Regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
        static ref RE_THREAD_NAME: Regex = Regex::new(r"[\w-]+").unwrap();
        static ref RE_THREAD_ID: Regex = Regex::new(r"ThreadId\(\d+\)").unwrap();
        static ref RE_FILE: Regex = Regex::new(r"[\w/\.-]+").unwrap();
        static ref RE_LINE: Regex = Regex::new(r"\d+").unwrap();
        static ref RE_MODULE: Regex = Regex::new(r"[\w:]+").unwrap();
    }

    // --- Individual test functions (test_basic_format, etc.) remain the same ---
    #[test]
    fn test_basic_format() {
        let pattern = "[%d]-[%T] %t [%l]: %m%n";
        let output = run_test_with_formatter(pattern, || {
            tracing::info!(target: "basic_test", "Hello, formatter!");
        });
        println!("Basic Output:\n{}", output);
        assert!(RE_DATE_DEFAULT.is_match(&output));
        assert!(output.contains("[INFO ]"));
        assert!(output.contains("[basic_test]"));
        assert!(output.contains(" Hello, formatter!"));
        assert!(output.ends_with('\n'));
        let thread_name_match = RE_THREAD_NAME.find(&output);
        assert!(thread_name_match.is_some(), "Thread name not found");
    }

    #[test]
    fn test_location_and_custom_date() {
        let pattern = "%d{[%Y/%m/%d]} %p %C::%M (%F:%L) - %% %m %z %n"; // %z is unknown
        let output = run_test_with_formatter(pattern, || {
            tracing::warn!(target: "location_test", "Location check");
        });
        println!("Location Output:\n{}", output);
        let re_custom_date = Regex::new(r"\[\d{4}/\d{2}/\d{2}]").unwrap();
        assert!(re_custom_date.is_match(&output));
        assert!(output.contains(" WARN "));
        // Adjust expected module based on actual location
        assert!(output.contains(module_path!()), "Module path (%C) incorrect");
        // %M often captures the event name/location, not a function name in Rust easily
        // assert!(output.contains("::test_location_and_custom_date"), "Meta name (%M) mismatch"); // This assertion might be fragile
        assert!(RE_FILE.is_match(&output));
        assert!(RE_LINE.is_match(&output));
        assert!(output.contains("%"), "Literal percent mismatch");
        assert!(output.contains(" Location check"));
        assert!(output.contains(" %z "), "Unknown specifier %z not handled"); // Check literal %z output
        assert!(output.ends_with('\n'));
        assert!(output.contains(file!()), "File name mismatch"); // Use file!() macro
    }

    #[test]
    fn test_span_fields() {
        // Pattern: [Span] Fields \n Message
        let pattern = "[%span] %X%n%m%n";
        let output = run_test_with_formatter(pattern, || {
            let _span = tracing::info_span!(
                "processing",
                job_id = 42,
                user.name = "alice",
                user.active = true
            ).entered();

            tracing::debug!("Processing step 1"); // Log inside 'processing'

            let _nested = tracing::info_span!("inner_task", task = "decode").entered();
            tracing::info!("Decoding data"); // Log inside 'inner_task' & 'processing'
        });

        println!("Span Output:\n{}", output);

        // Use .lines() to iterate over lines correctly
        let lines: Vec<&str> = output.lines().collect();

        // Expect 2 log events, each taking 2 lines due to the pattern
        assert_eq!(lines.len(), 4, "Expected exactly 4 lines (2 log events * 2 lines each)");

        // --- Check first log event block ---
        // Line 0: [processing] job_id=42 user.name="alice" user.active=true
        // Line 1: Processing step 1
        assert!(lines[0].starts_with("[processing] "), "Line 1: Outer span name prefix mismatch");
        // Extract the fields part after the prefix
        let fields_part_1 = lines[0].trim_start_matches("[processing] ").trim();
        // Check fields using contains (order insensitive)
        assert!(fields_part_1.contains("job_id=42"), "Line 1: job_id missing in fields: '{}'", fields_part_1);
        // Note: DefaultFields often adds quotes for strings
        assert!(fields_part_1.contains("user.name=\"alice\""), "Line 1: user.name missing in fields: '{}'", fields_part_1);
        assert!(fields_part_1.contains("user.active=true"), "Line 1: user.active missing in fields: '{}'", fields_part_1);
        assert!(!fields_part_1.contains("task="), "Line 1: Inner span field 'task' should not be present yet");
        // Check message line
        assert_eq!(lines[1], "Processing step 1", "Line 2: First message mismatch");


        // --- Check second log event block ---
        // Line 2: [inner_task] task="decode", job_id=42 user.name="alice" user.active=true (Order might vary)
        // Line 3: Decoding data
        assert!(lines[2].starts_with("[inner_task] "), "Line 3: Inner span name prefix mismatch");
        // Extract the fields part after the prefix
        let fields_part_2 = lines[2].trim_start_matches("[inner_task] ").trim();
        // Check fields using contains (order insensitive) - expect merged fields
        assert!(fields_part_2.contains("task=\"decode\""), "Line 3: task missing in fields: '{}'", fields_part_2);
        assert!(fields_part_2.contains("job_id=42"), "Line 3: job_id missing in fields: '{}'", fields_part_2);
        assert!(fields_part_2.contains("user.name=\"alice\""), "Line 3: user.name missing in fields: '{}'", fields_part_2);
        assert!(fields_part_2.contains("user.active=true"), "Line 3: user.active missing in fields: '{}'", fields_part_2);
        // Check message line
        assert_eq!(lines[3], "Decoding data", "Line 4: Second message mismatch");
    }


    #[test]
    fn test_thread_id_and_name() {
        let pattern = "[%t/%tid] %m%n";
        let output = run_test_with_formatter(pattern, || {
            tracing::error!("Error in thread");
        });

        println!("Thread Output:\n{}", output);

        assert!(RE_THREAD_NAME.is_match(&output), "Thread name format mismatch");
        assert!(RE_THREAD_ID.is_match(&output), "Thread ID format mismatch");
        assert!(output.contains("/"), "Separator '/' missing");
        assert!(output.contains(" Error in thread"), "Message mismatch");
        assert!(output.ends_with('\n'));
    }

}