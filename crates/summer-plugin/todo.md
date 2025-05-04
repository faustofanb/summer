# Summer Plugin - TODO & Progress Log

## TODOs (待办事项)

1.  **插件 API 定义:** (关联 F9)
    - [ ] **`Plugin` Trait:** (T3.2)
      - [ ] 定义 `Plugin` trait (在 `summer-core`)，包含初始化、关闭等生命周期方法。
      - [ ] 确定插件如何访问 `ApplicationContext` (获取 Bean) 和 `ConfigResolver` (获取配置)。
2.  **插件加载与管理:** (关联 F9)
    - [ ] **发现机制:** (T3.2)
      - [ ] 设计插件的发现机制 (e.g., 通过特定标记、配置文件或代码注册)。
    - [ ] **生命周期管理:** (T3.2)
      - [ ] 在 `ApplicationContext` 构建和关闭过程中，调用插件的生命周期方法。
3.  **测试:**
    - [ ] 编写插件加载和生命周期管理的测试。
    - [ ] 创建示例插件进行集成测试。

## Development Plan Tasks (关联开发计划)

- [T3.2] 设计并实现插件 API (`Plugin` trait) 和生命周期管理
