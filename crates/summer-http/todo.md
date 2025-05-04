# Summer HTTP - TODO & Progress Log

## TODOs (待办事项)

1.  **HTTP 服务器:** (关联 F1)
    - [ ] **基础服务器:** (T1.1)
      - [ ] 基于 `hyper` 实现 HTTP/1.1 服务器的启动和停止。
      - [ ] 配置监听地址和端口。
    - [ ] **请求处理:**
      - [ ] 将 `hyper::Request` 转换为框架内部的 `Request` 抽象 (T1.2)。
      - [ ] 将框架内部的 `Response` 抽象转换为 `hyper::Response` (T1.2)。
    - [ ] **异步处理:** 确保整个请求处理流程是异步的。
    - [ ] **优雅停机:** 实现 Graceful Shutdown。
    - [ ] **HTTPS/TLS 支持:** (T3.8) 集成 `rustls` 或 `native-tls` (通过 feature gate)。
2.  **请求/响应抽象:** (关联 F1, F2)
    - [ ] 定义框架内部的 `Request` 结构体/trait，封装常用信息 (方法, URI, Headers, Body Stream)。 (T1.2)
    - [ ] 定义框架内部的 `Response` 结构体/trait，封装状态码, Headers, Body。 (T1.2)
    - [ ] 提供读取请求体 (Body) 的便捷方法 (e.g., `read_body_bytes`, `read_json`).
3.  **与 MVC 集成:**
    - [ ] 将接收到的内部 `Request` 对象传递给 MVC 路由进行分发 (T1.9)。
4.  **测试:**
    - [ ] 编写 HTTP 服务器启动/停止/配置的测试。
    - [ ] 编写 HTTPS 支持的测试。
    - [ ] 编写请求/响应抽象的单元测试。

## Development Plan Tasks (关联开发计划)

- [T1.1] 实现基础 HTTP Server 启动/关闭
- [T1.2] 封装基础 Request/Response 对象
- [T1.9] 将 HTTP Server 请求分发到 MVC 路由
- [T3.8] 实现 HTTPS/TLS 支持
