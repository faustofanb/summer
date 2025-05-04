# Summer Config - TODO & Progress Log

## Completed Goals (截至 2025-05-04)

1.  **基础配置处理:**
    - 定义了 `ConfigResolver` trait 和 `MemoryConfigResolver`, `CompositeConfigResolver` 实现 (占位符)。

## TODOs (待办事项)

1.  **完善配置加载:** (关联 F4)
    - [ ] **实现 `FileConfigResolver`:** (T0.5 扩展)
      - [ ] 从 YAML/JSON 文件加载配置。
    - [ ] **实现 `EnvConfigResolver`:** (T0.5 扩展)
      - [ ] 从环境变量加载配置。
    - [ ] **实现分层加载和 Profile 支持:** (T0.5 扩展)
      - [ ] 结合 `CompositeConfigResolver` 实现 `application.yaml` 和 `application-{profile}.yaml` 的加载与覆盖逻辑。
      - [ ] 实现环境变量覆盖文件配置。
    - [ ] **提供配置源优先级策略。**
2.  **提供配置访问 API:**
    - [ ] 实现 `ConfigResolver` trait 的 `get<T>(&self, key: &str) -> Option<T>` 方法。
    - [ ] 实现 `get_required<T>(&self, key: &str) -> Result<T, ConfigError>` 方法。
    - [ ] 实现获取指定前缀下所有配置的方法 (用于 `@ConfigurationProperties`)。
3.  **支持配置注入 (运行时):** (由 IOC 调用)
    - [ ] **`@Value` 支持:** (T1.11) 提供 API 给 IOC，根据 key 获取值。
    - [ ] **`@ConfigurationProperties` 支持:** (T2.11) 提供 API 给 IOC，根据 prefix 获取配置块并支持反序列化。
4.  **高级功能:**
    - [ ] **热加载:** (可选) 实现监控配置文件变化并重新加载配置的机制。
    - [ ] **加密配置:** (可选) 支持对配置文件中的敏感信息进行加密和解密。
5.  **测试:**
    - [ ] 为不同配置源 (`File`, `Env`) 编写测试。
    - [ ] 为分层加载和 Profile 支持编写测试。
    - [ ] 为配置访问 API 编写测试。

## Development Plan Tasks (关联开发计划)

- [T0.5] 集成 `config-rs` 并实现基础配置加载逻辑
- [T1.11] 支持 `@Value` 注入 (提供运行时 API)
- [T2.11] 支持 `@ConfigurationProperties` (提供运行时 API)
