# Summer Logging - TODO & Progress Log

## TODOs (待办事项)

1.  **集成 Tracing:** (关联 F8)
    - [ ] **基础设置:** (T0.4)
      - [ ] 添加 `tracing` 和相关生态库 (`tracing-subscriber`, `tracing-appender`) 依赖。
      - [ ] 提供基础的日志初始化函数。
    - [ ] **配置:**
      - [ ] 实现从 `summer-config` 读取日志配置 (级别、格式、输出目标)。
      - [ ] 支持按模块设置日志级别。
    - [ ] **格式化:**
      - [ ] 提供默认的日志格式 (文本、JSON)。
      - [ ] 允许用户自定义格式。
    - [ ] **输出目标:**
      - [ ] 支持输出到控制台 (stdout/stderr)。
      - [ ] 支持输出到文件 (包括文件轮转 `tracing-appender`)。
    - [ ] **异步写入:** 确保文件写入是异步的。
2.  **上下文关联:** (关联 F8)
    - [ ] **Trace ID 集成:** (T2.12)
      - [ ] 与 Web 模块或中间件集成，自动在日志中包含请求的 Trace ID (如果可用)。
    - [ ] **(可选) MDC 支持:** 提供类似 MDC 的机制添加自定义上下文。
3.  **API:**
    - [ ] 确保 `tracing` 提供的宏 (`trace!`, `debug!`, `info!`, `warn!`, `error!`) 可用且配置生效。
4.  **测试:**
    - [ ] 测试日志初始化和配置加载。
    - [ ] 测试不同级别、格式、输出目标的日志记录。
    - [ ] 测试异步写入和文件轮转。

## Development Plan Tasks (关联开发计划)

- [T0.4] 集成 `tracing` 并实现基础日志配置
- [T2.12] 增强日志上下文关联 (Trace ID)
