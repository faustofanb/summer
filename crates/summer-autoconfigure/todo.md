# Summer Autoconfigure - TODO & Progress Log

## TODOs (待办事项)

1.  **实现自动配置核心逻辑:** (关联 F5)
    - [ ] **自动配置类发现与加载:** (T2.9)
      - [ ] 设计一种机制来发现和加载 `summer-starter-*` 包或其他位置定义的自动配置类 (`@Configuration` 类)。(可能通过 build script 或特定文件约定)。
    - [ ] **条件评估:** (T2.9, T3.7)
      - [ ] 实现 `@Conditional...` 注解的运行时评估逻辑。需要访问 IOC 容器状态 (已注册的 Bean)、配置信息 (`summer-config`)、类路径信息。
    - [ ] **`@Configuration` 和 `@Bean` 处理:** (T2.8 - 运行时部分)
      - [ ] 实现处理 `@Configuration` 类和其中 `@Bean` 方法的逻辑，根据条件评估结果，将 `@Bean` 方法的返回值注册为 IOC Bean (需与 IOC 模块协作)。
2.  **创建 Starter 包 (自动配置逻辑):**
    - [ ] **`summer-starter-web`:** (T3.3) 实现 Web 服务器、MVC 相关的自动配置。
    - [ ] **`summer-starter-sqlx`:** (T3.4) 实现 `sqlx` 数据源、连接池的自动配置。
    - [ ] **`summer-starter-redis`:** (T3.5) 实现 Redis 客户端的自动配置。
    - [ ] (其他 Starter...)
3.  **测试:**
    - [ ] 为条件评估逻辑编写测试。
    - [ ] 为 `@Configuration`/`@Bean` 处理逻辑编写测试。
    - [ ] 为各个 Starter 包编写集成测试，验证自动配置是否生效。

## Development Plan Tasks (关联开发计划)

- [T2.8] 实现自动配置基础注解 (运行时处理)
- [T2.9] 搭建自动配置加载和条件评估框架
- [T3.3] 开发 `summer-starter-web` (自动配置部分)
- [T3.4] 开发 `summer-starter-sqlx` (自动配置部分)
- [T3.5] 开发 `summer-starter-redis` (自动配置部分)
- [T3.7] 完善自动配置的条件注解实现
