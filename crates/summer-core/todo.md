# Summer Core - TODO & Progress Log

## Completed Goals (截至 2025-05-04)

1.  **移动共享类型:**
    - 将 `BeanDefinition`, `BeanScope`, `BeanDependency`, `ComponentDefinitionProvider`, `IocError` 从 `summer-ioc` 移动到 `summer-core` 以解决循环依赖。

## TODOs (待办事项)

1.  **完善核心类型:**
    - [ ] **`BeanDefinition` 增强:** (关联 F3, F6, F5)
      - [ ] 添加字段用于存储 AOP 相关的元数据 (例如，是否需要代理)。
      - [ ] 添加字段用于存储条件装配相关的元数据。
      - [ ] 添加 `is_primary`, `is_lazy` 等标志字段。
      - [ ] 添加字段存储 `@PostConstruct` / `@PreDestroy` 方法名或引用。
    - [ ] **错误处理 (`IocError`, `SummerError`):** (关联 T0.8)
      - [ ] 定义核心错误类型。
      - [ ] 细化错误类型，提供更具体的错误信息。
      - [ ] 实现 `std::error::Error` 和 `std::fmt::Display`。
    - [ ] **AOP 基础类型:** (关联 F6)
      - [ ] 定义 `JoinPoint` trait 或 struct。
      - [ ] 定义 Pointcut 相关的基础类型 (如果需要)。
    - [ ] **事件机制基础类型:** (关联 F3 - 间接)
      - [ ] 定义 `ApplicationEvent` trait 或基类。
      - [ ] 定义 `ApplicationListener` trait。
      - [ ] 定义 `ApplicationEventMulticaster` trait。
    - [ ] **配置相关 Trait:** (关联 F4)
      - [ ] 定义 `ConfigResolver` trait。
    - [ ] **插件/中间件 Trait:** (关联 F9)
      - [ ] 定义 `Plugin` trait 基础接口 (T3.2)。
      - [ ] 定义 Web 中间件基础 trait 或函数签名 (T3.1)。
    - [ ] **HTTP 基础类型:** (关联 F1, F2)
      - [ ] 定义框架内部统一的 Request/Response 抽象 (T1.2)。
      - [ ] 定义 `IntoResponse` trait 用于统一响应处理 (T2.3)。
2.  **测试:**
    - [ ] 为核心类型和 Trait 编写单元测试。

## Development Plan Tasks (关联开发计划)

- [T0.8] 定义核心错误类型
- (Implicit) 定义各模块共享的基础 Traits 和 Structs
