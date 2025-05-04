# Summer AOP TODO List

- [ ] 实现 `@Aspect` 注解标记切面结构体。
- [ ] 定义切点表达式语法和解析逻辑（基于注解、路径、签名、组合）。
- [ ] 实现 `@Pointcut` 注解。
- [ ] 实现 `@Before` 通知。
- [ ] 实现 `@AfterReturning` 通知。
- [ ] 实现 `@AfterThrowing` 通知。
- [ ] 实现 `@After` 通知。
- [ ] 实现 `@Around` 通知 (挑战性高，可能延后或简化)。
- [ ] 实现编译时代码织入机制（优先）。
- [ ] 研究运行时代理机制（备选）。
- [ ] 与 IOC 容器集成，自动为 Bean 创建代理。
- [ ] 提供 `JoinPoint` API 以在通知中访问上下文信息。
- [ ] 编写 AOP 模块的单元测试和集成测试.
