# Summer Macros - TODO & Progress Log

## Completed Goals (截至 2025-05-04)

1.  **基础 `#[derive(Component)]` 宏:**
    - 创建了 `summer-macros` crate。
    - 实现了基本的 `#[derive(Component)]` 宏，生成 `impl ComponentDefinitionProvider`。
    - 宏生成的 `BeanDefinition` 包含基本的 `factory_fn`（最初假设 `Default`，后改为接收依赖）。
    - 宏尝试解析字段作为依赖项（简化版）。
2.  **增强 `#[derive(Component)]` 宏 (部分):**
    - 实现了对结构体属性 `#[component(name = "...", scope = "...")]` 的解析。
    - 实现了对字段属性 `#[autowired]` 的解析，现在只有标记了此注解的字段才被视为依赖项。
    - 宏生成的 `BeanDefinition` 现在使用解析出的 `name` 和 `scope`。
    - 工厂函数现在只接收 `@Autowired` 字段对应的依赖，并为非注入字段使用 `Default::default()` (需要目标字段实现 `Default`)。

## TODOs (待办事项)

1.  **IOC 相关宏:** (关联 F3, F7)
    - [ ] **完善 `@Component` / `@Service` / `@Repository` / `@Controller` / `@RestController`:** (T1.3)
      - [ ] 确保正确生成 `BeanDefinition` 元数据 (name, scope, dependencies, lifecycle methods)。
      - [ ] 支持更多属性 (e.g., `primary`, `lazy`)。
    - [ ] **实现 `@Autowired`:** (T1.3)
      - [ ] 正确识别字段类型 (包括 `Arc<T>`, `Option<Arc<T>>`) 并记录依赖。
      - [ ] 支持构造函数注入的分析。
    - [ ] **实现 `@Value`:** (T1.3, T1.11)
      - [ ] 解析 `${key:defaultValue}` 表达式。
      - [ ] 记录需要注入的配置键。
    - [ ] **实现 `@PostConstruct` / `@PreDestroy`:** (T2.10)
      - [ ] 解析注解并记录方法名到 `BeanDefinition`。
    - [ ] **实现 `@Configuration` / `@Bean`:** (T2.8)
      - [ ] 解析 `@Configuration` 类。
      - [ ] 解析 `@Bean` 方法，生成对应的 `BeanDefinition` (包括方法参数依赖)。
2.  **MVC 相关宏:** (关联 F2, F7)
    - [ ] **实现路由注解 (`@RequestMapping`, `@GetMapping`, `@PostMapping`, etc.):** (T1.7)
      - [ ] 解析路径、方法、参数等。
      - [ ] 生成静态路由信息供运行时使用 (T1.8)。
    - [ ] **实现参数绑定注解 (`@PathVariable`, `@RequestParam`, `@RequestHeader`, `@RequestBody`):** (T2.1, T2.2)
      - [ ] 解析注解属性。
      - [ ] 生成代码或元数据以支持运行时的参数提取和反序列化。
    - [ ] **实现 `@ExceptionHandler`:** (T2.4)
      - [ ] 解析注解，记录异常类型和处理方法。
3.  **AOP 相关宏:** (关联 F6, F7)
    - [ ] **实现 `@Aspect`:** (T2.5)
      - [ ] 标记切面类。
    - [ ] **实现 `@Pointcut`:** (T2.5)
      - [ ] 解析切点表达式 (初步可支持基于注解或路径)。
    - [ ] **实现通知注解 (`@Before`, `@AfterReturning`, `@AfterThrowing`, `@After`, `@Around`):** (T2.5)
      - [ ] 解析注解，关联到 Pointcut。
    - [ ] **实现编译时织入:** (T2.6)
      - [ ] 修改被 IOC 管理的 Bean 的方法代码，插入通知调用逻辑。
4.  **配置相关宏:** (关联 F4, F7)
    - [ ] **实现 `@ConfigurationProperties`:** (T2.11)
      - [ ] 解析 `prefix` 属性。
      - [ ] 记录需要绑定的配置前缀和目标类型。
5.  **自动配置相关宏:** (关联 F5, F7)
    - [ ] **实现条件注解 (`@ConditionalOnProperty`, `@ConditionalOnBean`, etc.):** (T2.8)
      - [ ] 解析注解条件。
      - [ ] 生成元数据供自动配置模块运行时评估。
6.  **测试:**
    - [ ] 为各种宏编写单元测试和集成测试 (使用 `trybuild` 等)。

## Development Plan Tasks (关联开发计划)

- [T1.3] 实现核心注解 (`@Component`, `@Service`, `@Controller`, `@Autowired`, `@Value` 基础)
- [T1.7] 实现 MVC 路由注解 (`@GetMapping`, `@PostMapping` 等)
- [T1.8] 实现静态路由表生成 (部分)
- [T2.1] 实现请求绑定注解 (`@PathVariable`, `@RequestParam`, `@RequestHeader`)
- [T2.2] 实现 `@RequestBody` 宏
- [T2.5] 实现 AOP 核心注解 (`@Aspect`, `@Pointcut`, `@Before`, `@AfterReturning` 等)
- [T2.6] 实现编译时 AOP 织入
- [T2.8] 实现自动配置基础注解 (`@Configuration`, `@Bean`, `@Conditional...` 基础)
- [T2.10] 实现生命周期注解 (`@PostConstruct`, `@PreDestroy`)
- [T2.11] 实现 `@ConfigurationProperties` 注解
