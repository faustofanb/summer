# Summer-IOC: 核心设计、实现规划与考量

本文档详细阐述了 `summer-ioc` crate 的设计目标、核心工作流程、接口定义、关键算法、资源需求、测试策略以及待办事项。

## I. 核心待办事项列表

**1. 核心容器实现:**

- [ ] 实现 `BeanDefinition` 结构：存储 Bean 元数据（类型 `TypeId`、唯一名称、作用域、依赖项信息、构造器函数指针、生命周期方法信息等）。
- [ ] 实现 `IocContainer` (或类似 `ApplicationContext` 的结构)：作为核心引擎，负责 Bean 的生命周期管理（注册、实例化、依赖注入、销毁）和检索。
- [ ] 实现基于 `TypeId` 和唯一 Bean 名称的注册与检索逻辑。
- [ ] 使用受 `parking_lot::RwLock` 保护的 `HashMap` (或等效并发安全结构) 存储单例 Bean 实例 (`singleton_instances`)、Bean 定义 (`definitions`) 以及类型到名称的映射 (`beans_by_type`)。

**2. 依赖注入:**

- [ ] 实现构造函数注入逻辑：依赖 `summer-macros` 分析构造函数（如 `::new`) 并生成包含依赖解析逻辑的包装器函数。
- [ ] 实现字段注入 (`@Autowired`) 逻辑：依赖 `summer-macros` 识别目标字段、解析其类型，并在实例化后由容器执行注入。
- [ ] 在容器中实现稳健的依赖解析逻辑，包括**循环依赖检测**（使用 `currently_in_creation` 集合）并返回明确的错误 (`DependencyCycleError`)。
- [ ] 确保支持注入 `Arc<T>` (具体类型) 和 `Arc<dyn Trait>` (接口类型)。

**3. Bean 生命周期管理:**

- [ ] 实现 `@PostConstruct` 注解逻辑：在 Bean 实例化和所有依赖注入完成后，自动调用标记的方法。
- [ ] 实现 `@PreDestroy` 注解逻辑：在容器关闭或 Bean 销毁前，按正确的依赖反向顺序调用标记的方法。

**4. 作用域管理:**

- [ ] 实现**单例 (Singleton)** 作用域（默认）。
- [ ] (可选) 设计扩展点 (如 Trait 或配置) 以支持未来的其他作用域（如 Prototype、Request 等）。

**5. 配置集成 (`summer-config`):**

- [ ] 实现 `@Value("${config.key}")` 注解逻辑：将 `summer-config` 提供的配置值注入到 Bean 字段中（需要宏支持类型转换）。
- [ ] 实现 `@ConfigurationProperties("prefix")` 注解逻辑：将特定前缀下的配置项批量绑定到结构体 Bean 的字段上（需要宏支持）。

**6. 日志集成 (`summer-logging`):**

- [ ] 在容器关键操作（Bean 注册、实例化、注入、销毁、错误发生时）添加详细的日志记录，使用 `summer-logging` 框架。

**7. 错误处理:**

- [ ] 定义清晰、具体的 `IocError` 枚举类型（例如：`BeanNotFound(Name/Type)`、`DependencyCycle`、`InstantiationError(cause)`、`InjectionError(field/param)`、`TypeMismatch`、`ConfigError(key)` 等）。 - [ ] 确保所有可能失败的操作都返回 `Result<_, IocError>`，提供有用的上下文信息。

**8. 测试:**

- [ ] 编写**单元测试**覆盖核心容器的各种方法和逻辑（注册覆盖/冲突、按类型/名称获取、错误处理路径、循环依赖检测机制）。
- [ ] 编写**宏测试**（使用 `trybuild` 等）验证 `#[component]`, `#[autowired]` 等宏在不同输入下的代码生成（成功/失败）及生成的代码片段的正确性。
- [ ] 编写**集成测试**模拟真实应用场景：包含多个相互依赖的 Bean、混合使用构造函数/字段注入、集成配置注入、测试生命周期回调的执行、并发访问下的正确性和性能。

## II. 核心工作流

### A. 编译时: 元数据收集 (`summer-macros` + `inventory`)

1.  **注解标记:** 开发者使用 `#[component]`, `#[derive(Default)]` (当前), 或未来配合 `#[autowired]` 等注解标记 Rust 结构体和其字段/方法。
2.  **宏处理:** `summer-macros` 在编译期间解析这些注解和相关的代码结构（结构体名、字段、方法签名等）。
3.  **元数据提取:** 宏提取关键信息，如 Bean 名称、`TypeId`、依赖关系（构造函数参数类型、`#[autowired]` 字段类型）、生命周期回调方法名等。
4.  **构造器包装器生成:** 对于构造函数注入，宏生成一个标准的 `constructor_wrapper` 函数（签名为 `fn(Arc<dyn BeanProvider>) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn Error + Send + Sync>>`）。此函数封装了调用容器 `get_bean_by_typeid` 来获取依赖，并最终调用原始构造函数 `StructName::new(...)` 的逻辑。（_注：当前可能使用 `Default::default()` 作为临时实现，待宏完善_）。
5.  **元数据注册:** 宏利用 `inventory::submit!` 将包含上述信息的 `BeanDefinitionMetadata` 实例注册到全局集合中，等待运行时容器收集。

### B. 运行时: 容器初始化与 Bean 管理 (`summer-ioc`)

1.  **容器创建:** 应用程序启动时，创建 `IocContainer` 实例，通常使用 `Arc` 包裹以支持共享：`let container = Arc::new(IocContainer::new());`。
2.  **初始化 (`container.initialize()`):**
    - 设置内部 `initialized` 状态，防止重复初始化（写锁保护）。
    - 遍历 `inventory::iter::<BeanDefinitionMetadata>()` 收集所有编译时注册的元数据。
    - 为每个元数据创建运行时的 `BeanDefinition` 实例，存储构造器指针、类型信息等。
    - 将 `BeanDefinition` 注册到内部映射中（`definitions` 按名称，`beans_by_type` 按 `TypeId`）（写锁保护）。
    - 存储容器自身的 `Arc` (`self_arc`)，用于后续传递给构造器包装函数。
3.  **Bean 获取与实例化 (`get_bean<T>()`, `get_bean_by_name(...)`):**
    - 外部代码通过类型或名称请求 Bean。基于类型的请求 (`get_bean<T>()`) 会先查找对应的 Bean 名称。
    - 核心逻辑委托给 `get_bean_by_name_any(name)`:
      - **(读锁):** 检查单例缓存 `singleton_instances`。若命中，克隆 `Arc` 并直接返回。
      - 若未命中，则进入实例化流程 `instantiate_bean(name)`。
    - `instantiate_bean(name)` (核心实例化逻辑):
      - **(写锁):** 尝试将 `name` 加入 `currently_in_creation` 集合，若已存在则检测到循环依赖，返回 `DependencyCycleError`。
      - **(读锁):** 获取 `name` 对应的 `BeanDefinition`，若无则返回 `BeanNotFound`。
      - 获取 `constructor_wrapper` 函数指针。
      - **调用构造器:** 执行 `constructor_wrapper(self.self_arc.clone())`，将容器自身 (`Arc<dyn BeanProvider>`) 传入，使其能在内部递归调用 `get_bean_by_typeid` 解析构造函数依赖。
      - **处理构造结果:**
        - 若 `Ok(instance_arc_any)`:
          - **(写锁):** 获取 `singleton_instances` 的写锁。
          - **双重检查锁定 (DCL):** 再次检查缓存。若在等待锁期间其他线程已创建，则使用缓存的实例，丢弃本次创建的。
          - 若缓存仍为空，将 `instance_arc_any` 存入缓存。
          - 执行后续处理：字段注入 (`resolve_dependencies`)、配置注入 (`apply_configuration`)、`@PostConstruct` 回调 (`apply_post_construct`)。这些步骤需在持有实例的 Arc 上操作，可能需要 Mutex 或 反射（如果适用）。
          - 返回最终处理好的 `Ok(instance_arc_any)` (新创建或缓存的)。
        - 若 `Err(e)`: 将构造或后续处理中的错误包装为 `InstantiationError` 或 `InjectionError` 等返回。
      - **(写锁):** 从 `currently_in_creation` 中移除 `name` (无论成功或失败)。
    - 外层调用者收到 `Ok(Arc<dyn Any + Send + Sync>)` 后，尝试 `downcast::<T>()` 到请求的具体类型，失败则返回 `TypeMismatchError`。
4.  **容器关闭:** 触发所有单例 Bean 的 `@PreDestroy` 回调（需考虑依赖关系，按反向顺序销毁）。

## III. 接口设计

- **公共接口 (面向用户):**
  - `IocContainer::new() -> IocContainer`: 创建容器实例。
  - `IocContainer::initialize(self: Arc<Self>) -> Result<(), IocError>`: 初始化容器，收集 Bean 定义并准备就绪。接收 `Arc<Self>` 以便内部存储。
  - `IocContainer::get_bean<T: 'static>(&self) -> Result<Arc<T>, IocError>`: 按类型获取 Bean 实例。
  - `IocContainer::get_bean_by_name<T: 'static>(&self, name: &str) -> Result<Arc<T>, IocError>`: 按名称获取 Bean 实例。
- **内部接口/契约 (宏与容器之间):**
  - `BeanProvider` Trait: 定义容器提供 Bean 的能力 (主要是 `get_bean_by_typeid`)，`IocContainer` 实现此 Trait，并将 `Arc<dyn BeanProvider>` 传递给宏生成的构造器。
  - `BeanDefinitionMetadata`: 编译时由宏生成并通过 `inventory` 提交的结构体，包含容器运行时构建 `BeanDefinition` 所需的静态信息（名称、类型 ID、构造器函数指针等）。

## IV. 关键设计考量

- **核心数据结构:**
  - `definitions: RwLock<HashMap<String, BeanDefinition>>`: 存储 Bean 的完整定义信息。
  - `singleton_instances: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>`: 缓存已创建的单例 Bean 实例。
  - `beans_by_type: RwLock<HashMap<TypeId, String>>`: 存储 `TypeId` 到 Bean 名称的映射，用于按类型查找。
  - `currently_in_creation: RwLock<HashSet<String>>`: 用于在实例化过程中检测循环依赖。
  - `self_arc: RwLock<Option<Arc<dyn BeanProvider>>>`: 存储容器自身的 Arc 引用。
- **类型擦除与检索:** 使用 `TypeId` 和 `Arc<dyn Any + Send + Sync>` 实现类型无关的存储和管理。依赖 `downcast` 进行类型安全的检索。对 `Arc<dyn Trait>` 的注入和查找是关键特性。
- **并发性与线程安全:** 容器内部状态必须通过 `RwLock` 或类似机制保护，以支持多线程环境下的安全访问（多读单写）。Bean 实例通过 `Arc` 安全共享。所有注册为 Bean 的类型必须实现 `Send + Sync`。
- **错误处理:** 设计细粒度的 `IocError` 类型，并在所有可能失败的路径上返回 `Result`，提供足够的上下文信息帮助调试。
- **宏集成:** IOC 容器强依赖 `summer-macros` 提供元数据。两者间的接口（`BeanProvider`, `BeanDefinitionMetadata`）和数据传递（`inventory`）必须清晰、稳定。
- **性能:** 单例缓存是核心优化点。需要关注锁竞争（尤其是在高并发初始化或获取 Bean 时），以及 `HashMap` 查找效率。
- **资源需求:**
  - **内存:** 主要消耗在于存储 `BeanDefinition` 对象和缓存的 `singleton_instances` (`Arc` 指针及共享的实例数据)。Bean 数量和大小直接影响内存占用。
  - **CPU:** 编译时宏展开消耗编译时间。运行时，Bean 实例化（特别是首次创建和涉及复杂依赖链时）以及高并发下的锁操作会消耗 CPU 资源。
- **设计可测试性:** 核心逻辑（实例化、依赖解析、生命周期管理）应尽可能与宏解耦，便于单元测试。
- **容器作为 Bean Provider:** 将容器自身 (`Arc<dyn BeanProvider>`) 传递给构造器包装函数，是实现依赖注入的关键机制。

## V. 核心算法设计

- **Bean 实例化算法:**
  1.  检查 `singleton_instances` 缓存 (读锁)。命中则返回。
  2.  获取 `currently_in_creation` 写锁，添加当前 Bean 名称。检查是否已存在，若存在则报循环依赖错误。
  3.  获取 `definitions` 读锁，查找 Bean 定义。
  4.  调用 `BeanDefinition` 中的 `constructor_wrapper` 函数，传入 `self_arc` (容器自身)。
  5.  `constructor_wrapper` 内部（理想情况下）递归调用 `get_bean` 来解析依赖。
  6.  构造成功后，获取 `singleton_instances` 写锁。
  7.  执行**双重检查锁定**：再次检查缓存，防止并发写入。
  8.  若缓存仍为空，则将新实例存入缓存。
  9.  执行字段注入和 `@PostConstruct` 回调。
  10. 释放 `currently_in_creation` 写锁，移除 Bean 名称。
  11. 返回实例 `Arc`。
- **依赖解析算法:** 主要发生在宏生成的 `constructor_wrapper` 函数内部。它接收 `Arc<dyn BeanProvider>`，并为每个构造函数参数调用 `provider.get_bean_by_typeid(dep_type_id)?` 来递归地获取依赖实例。字段注入则在实例化后，由容器遍历元数据并调用 `get_bean` 来完成。
- **循环依赖检测算法:** 使用 `RwLock<HashSet<String>>` (即 `currently_in_creation`)。在开始实例化一个 Bean `A` 时，将其名称加入集合（写锁）。如果 `A` 依赖 `B`，则递归调用实例化 `B`。若在实例化 `B` 的过程中又需要实例化 `A`，则在尝试将 `A` 加入集合时会发现其已存在，从而检测到循环。实例化完成（无论成功或失败）后，将 Bean 名称从集合中移除（写锁）。

## VI. 测试考量

- **单元测试:**
  - 测试 `IocContainer` 的基本操作：`register`, `get_bean`, `get_bean_by_name` 的成功路径。
  - 测试各种错误情况：Bean 未找到、类型不匹配、重复注册（根据策略报错或覆盖）。
  - 专门测试循环依赖检测逻辑能否在不同场景下（直接循环、间接循环）正确触发 `DependencyCycleError`。
  - 测试 `BeanDefinition` 的构建和元数据存储。
- **宏 (`summer-macros`) 测试:**
  - 使用 `trybuild` 库验证 `#[component]` 等宏在不同结构体定义（含/不含 `impl Default`, 未来含/不含 `new`）下的代码生成。
  - 测试宏能否正确提取类型信息、生成构造器包装函数（即使是临时的）。
  - 测试宏对错误输入的处理（如注解用在不支持的类型上）。
- **集成测试:**
  - 构建包含多个 Bean (structs) 的小型模拟应用，这些 Bean 之间存在多层依赖关系（构造函数注入、字段注入混合）。
  - 验证容器启动时能否正确初始化所有 Bean 并完成所有依赖注入。
  - 测试配置注入 (`@Value`, `@ConfigurationProperties`) 是否能正确读取配置并注入。
  - 测试生命周期方法 (`@PostConstruct`, `@PreDestroy`) 是否按预期顺序执行。
  - 模拟并发场景：多个线程同时请求不同的 Bean 或同一个 Bean，验证容器的线程安全性和缓存的有效性。
  - 测试容器关闭时 `@PreDestroy` 的调用顺序是否符合预期（反向依赖顺序）。

## VII. 潜在风险与挑战

- **循环依赖处理:** 虽然能检测，但如何优雅地向用户报告循环链，以及是否支持某些有限的循环依赖场景（如通过延迟注入/代理）是挑战。
- **线程安全细节:** 锁的粒度和持有时间需要仔细设计，避免死锁（特别是涉及多个锁的操作）和性能瓶颈（锁竞争）。
- **类型系统与 `dyn Any`:** `downcast` 的使用需要格外小心，确保类型信息的准确传递和转换，否则易引发运行时 panic。对 `dyn Trait` 的支持需要精确处理 `TypeId` 和实例存储。
- **生命周期管理复杂性:** 确保 `@PreDestroy` 的调用顺序严格遵守依赖反向关系，尤其在复杂的依赖图中，实现起来可能比较复杂且容易出错。
- **宏的健壮性:** `summer-macros` 需要处理各种 Rust 语法结构和边缘情况，保证生成的代码正确且高效，这本身就是一项复杂任务。
- **配置集成错误处理:** 配置缺失、类型不匹配等问题需要清晰地报告给用户，而不是导致隐晦的运行时失败。

## VIII. 未来改进与优化方向

- **完善宏功能:** 优先实现对 `impl` 块和 `new` 函数的解析，支持真正的构造函数注入。实现字段注入、生命周期、配置等注解的完整宏支持。
- **实现运行时逻辑:** 完成字段注入、生命周期回调 (`@PostConstruct`, `@PreDestroy`) 的容器运行时逻辑。
- **扩展作用域:** 在 Singleton 基础上，实现 Prototype（每次请求创建新实例）等作用域。
- **高级特性:** 引入 `@Qualifier` 和 `@Primary` 来解决同一类型有多个 Bean 实现时的歧义问题。考虑支持 Bean 的懒加载 (`@Lazy`)。
- **错误报告增强:** 提供更详尽的错误信息，例如在循环依赖错误中打印出完整的依赖链。
- **性能调优:** 对容器进行性能分析（profiling），识别并优化锁竞争、查找效率等瓶颈。
- **容器自身可注入:** 允许其他 Bean 注入 `Arc<IocContainer>` 或 `Arc<dyn BeanProvider>`，方便进行更动态的操作（需小心处理引导和循环引用问题）。

---
