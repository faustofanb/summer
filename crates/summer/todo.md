# Summer (Main Crate) - TODO & Progress Log

## TODOs (待办事项)

1.  **Re-exports:**
    - [ ] 导出核心模块的关键类型和宏，方便用户使用。
    - [ ] 组织导出结构，使其易于发现和使用。
    - [ ] **(New)** 确保 re-export 的 API 稳定性，避免不必要的破坏性更改。
2.  **便捷启动器:**
    - [ ] 提供一个简单的 `Summer::new().run()` 或类似 API 来启动应用。
    - [ ] 封装 `ApplicationContextBuilder` 的常用设置。
    - [ ] **(New)** 提供配置启动器行为的选项（例如，日志级别、配置文件位置）。
3.  **Feature Gates:**
    - [ ] 定义和管理 Feature Gates，允许用户选择性地启用功能 (e.g., `web`, `sqlx`, `redis`, `aop`)。
    - [ ] **(New)** 测试不同 Feature 组合下的编译和运行情况，确保兼容性。
    - [ ] **(New)** 在文档中清晰说明每个 Feature 的作用和依赖。
4.  **测试:**
    - [ ] 编写简单的集成测试，验证通过主 Crate 启动的应用能正常工作。
    - [ ] **(New)** 针对不同的 Feature 组合编写集成测试。
5.  **(New) 文档:**
    - [ ] 编写 `summer` crate 的 README 或入门指南，说明如何快速开始。
    - [ ] 提供 API 文档，解释 re-export 的模块和便捷启动器的用法。
6.  **(New) 错误处理:**
    - [ ] 定义统一的错误类型或错误处理策略，整合来自不同子模块的错误。

## Development Plan Tasks (关联开发计划)

- (Implicit) 作为框架的最终用户入口。
- [ ] 设计和实现基准测试 (`benches`) 以评估关键路径性能。
- [ ] 创建全面的示例项目 (`examples`) 展示框架的不同用法和特性集成。
- [ ] 开发启动器模板 (`starters`) 简化常见应用类型的项目初始化。
- [ ] 完善集成测试和单元测试 (`tests`) 覆盖核心功能和边缘情况.
- [ ] **(New)** 制定发布流程和版本管理策略，协调各子 Crate 的版本。
