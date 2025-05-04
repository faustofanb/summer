# Summer IOC - TODO & Progress Log

## Completed Goals (截至 2025-05-04)

1.  **基础 IOC 容器结构:**
    - 定义了核心结构 (移至 `summer-core`)。
    - 实现了基于 `TypeId` 的 Bean 注册和检索 (`BeanRegistry`)。
    - 实现了基于 `HashMap` 的单例 Bean 存储 (`singletons`)。
    - 定义了 `BeanFactory` trait。
2.  **基础 Bean 生命周期管理:**
    - 在 `ApplicationContextBuilder` 中实现了 Bean 实例化的大致流程。
    - 引入了 `BeanPostProcessor` 和 `BeanFactoryPostProcessor` trait 及其处理流程（占位符）。
    - 实现了基于依赖关系的拓扑排序 (`sort_dependencies`) 以确定实例化顺序。
3.  **基础配置处理:**
    - `ApplicationContext` 持有 `ConfigResolver` (占位符)。
4.  **基础事件机制:**
    - 定义了 `ApplicationEvent`, `ApplicationListener`, `ApplicationEventMulticaster` (占位符)。
    - `ApplicationContext` 提供 `publish_event` 方法 (占位符)。
5.  **基础依赖注入 (字段注入):**
    - 修改了 `BeanDefinition` 的 `factory_fn` 签名以接收依赖 (移至 `summer-core`)。
    - 修改了 `ApplicationContextBuilder::build` 方法，在实例化 Bean 之前解析其依赖项（从 `singletons` 获取），并将依赖项传递给 `factory_fn`。
6.  **错误处理和修复:**
    - 定义了 `IocError` (移至 `summer-core`)。
    - 修复了多轮编译错误。
7.  **重构:**
    - 解决了 `summer-ioc` 和 `summer-macros` 循环依赖 (使用 `summer-core`)。

## TODOs (待办事项)

1.  **容器构建与初始化:** (关联 F3)
    - [ ] **组件扫描与注册:** (T1.4)
      - [ ] 实现查找 `ComponentDefinitionProvider` 实现并调用 `get_bean_definitions`。
      - [ ] 将获取到的 `BeanDefinition` 注册到容器中。
    - [ ] **依赖注入实现:** (T1.5)
      - [ ] 实现构造函数注入逻辑 (分析 `BeanDefinition` 中的依赖，查找并传递)。
      - [ ] 实现字段注入逻辑 (在 Bean 实例化后，查找并设置 `#[autowired]` 字段)。
    - [ ] **Bean 实例化:**
      - [ ] 根据 `BeanDefinition` 调用工厂函数或构造函数创建 Bean 实例。
      - [ ] 处理循环依赖（检测并报错，或支持三级缓存）。
    - [ ] **Bean 生命周期管理:** (T2.10)
      - [ ] 实现 `BeanPostProcessor` 调用点 (初始化前后)。
      - [ ] 实现 `@PostConstruct` 方法的调用。
      - [ ] 实现 `@PreDestroy` 方法的调用 (容器关闭时)。
    - [ ] **作用域管理:**
      - [ ] 完善单例 (Singleton) 作用域实现。
      - [ ] (可选) 实现原型 (Prototype) 作用域。
2.  **配置集成:** (关联 F4)
    - [ ] **`@Value` 注入:** (T1.11)
      - [ ] 在 Bean 初始化过程中，根据 `BeanDefinition` 中的信息，从 `ConfigResolver` 获取值并注入。
    - [ ] **`@ConfigurationProperties` 绑定:** (T2.11)
      - [ ] 在 Bean 初始化后，根据 `BeanDefinition` 中的信息，从 `ConfigResolver` 获取配置块并反序列化绑定到 Bean 实例。
3.  **AOP 集成:** (关联 F6)
    - [ ] **代理创建:** (T2.6)
      - [ ] 在 `BeanPostProcessor` 中检查 Bean 是否需要 AOP 代理。
      - [ ] 如果需要，创建代理对象替换原始 Bean 实例。
4.  **自动配置集成:** (关联 F5)
    - [ ] **加载自动配置类:** (T2.9)
      - [ ] 实现扫描和加载 `@Configuration` 类（可能需要特定机制）。
    - [ ] **处理 `@Bean` 方法:** (T2.8)
      - [ ] 在处理 `@Configuration` 类时，执行 `@Bean` 方法并将结果注册为 Bean。
    - [ ] **条件评估:** (T2.9, T3.7)
      - [ ] 在注册 BeanDefinition 或实例化 Bean 前，评估 `@Conditional...` 注解。
5.  **事件机制:**
    - [ ] 实现 `ApplicationEventMulticaster` 的具体逻辑 (注册监听器、广播事件)。
    - [ ] 实现 `ApplicationContext` 的 `publish_event` 逻辑。
    - [ ] 在容器启动、关闭等关键节点发布内置事件。
6.  **API 与易用性:**
    - [ ] 提供获取 Bean 的 API (`get_bean<T>`, `get_bean_by_name`) (T1.6)。
    - [ ] 完善错误处理和提示信息。
7.  **测试:**
    - [ ] 为 IOC 核心功能 (注册、注入、生命周期、作用域) 编写单元测试和集成测试。
    - [ ] 测试与 Config, AOP, AutoConfig 的集成。

## Development Plan Tasks (关联开发计划)

- [T0.7] 搭建 IOC 容器的基础框架
- [T1.4] 实现编译时组件扫描与注册
- [T1.5] 实现构造函数注入和字段注入
- [T1.6] 实现 IOC 容器的 Bean 获取 API
- [T1.11] 集成 `@Value` 注入 (运行时部分)
- [T2.10] 实现 IOC 生命周期回调
- [T2.11] 集成 `@ConfigurationProperties` (运行时部分)
- (Implicit) AOP 集成点, AutoConfig 集成点, 事件机制实现
