Okay, based on the provided statement emphasizing a holistic, extensible, maintainable, and efficient coding approach for the Summer Framework, here's a refined set of rules (a "prompt") for an LLM generating code for this project:

---

## **Summer Framework - LLM Code Generation Rules & Guidelines**

**Overarching Goal:** Generate Rust code for the Summer Framework that is idiomatic, robust, maintainable, extensible, performant, and integrates seamlessly within the established modular architecture, prioritizing the use of well-suited existing libraries.

**Core Principles:**

1.  **Architectural Awareness (Global Perspective):**

    - Before writing code for a specific component, consider its place within the overall Summer Framework architecture (as defined in ODD/DDS documents).
    - Understand its interactions and dependencies with other modules (e.g., IOC interacts with Config and Macros, Web interacts with IOC and Middleware, Logging is used by all).
    - Ensure the code respects the established data flow and control flow between modules.

2.  **Design for Extensibility (Future Needs):**

    - Favor designs that allow for future expansion without requiring major rewrites (Open/Closed Principle).
    - Utilize traits (interfaces) for defining contracts between components, especially where different implementations might exist later (e.g., `ConfigSourceProvider`, `LogEngineStrategy`, `Scope`).
    - Employ composition over inheritance where appropriate.
    - Design APIs with potential future parameters or return types in mind, possibly using builder patterns for complex configurations or `Option`/`Result` for flexibility.

3.  **Judicious Use of Design Patterns:**

    - Apply relevant design patterns (Strategy, Builder, Factory Method, Facade, Observer, Command, Registry, etc.) where they genuinely improve structure, reduce coupling, increase cohesion, or enhance extensibility.
    - **Do not force patterns.** Only use them if they solve a specific design problem elegantly.
    - When using a pattern, add a brief comment explaining _which_ pattern is being used and _why_ it's appropriate in this context.

4.  **Modular Structure & File Organization:**

    - Place code in the appropriate crate (`summer-core`, `summer-ioc`, `summer-web`, `summer-macros`, etc.) based on its primary responsibility.
    - Within a crate, organize code into logical modules (`mod`). Adhere to Rust conventions (`src/lib.rs`, `src/module_name/mod.rs`).
    - Separate concerns: Keep distinct functionalities (e.g., configuration parsing, bean registration, HTTP routing) in separate modules or files.
    - Ensure public APIs (`pub`) are intentionally exposed and well-defined, while internal details remain private.

5.  **Code Clarity and Readability:**

    - Write clear, concise, and self-explanatory code.
    - Use meaningful names for variables, functions, types, and modules.
    - Keep functions and methods short and focused on a single task (Single Responsibility Principle).
    - Adhere strictly to Rust idioms and best practices.
    - Use `rustfmt` for consistent code formatting.
    - **Add comments:** Explain _why_ something is done a certain way (especially for complex logic, non-obvious choices, or workarounds), not just _what_ it does. Document public APIs thoroughly using `rustdoc`.

6.  **Maintainability & Refactorability (Low Coupling, High Cohesion):**

    - Strive for **high cohesion** within modules (related code stays together) and **low coupling** between modules (modules depend on each other as little as possible, preferably through stable interfaces/traits).
    - Avoid global mutable state wherever possible. Pass dependencies explicitly (DI).
    - Write **testable code**. Design components with clear inputs and outputs, making unit testing easier. Consider dependency injection even for internal components if it aids testability.
    - Minimize side effects in functions where possible.

7.  **Configuration over Hardcoding:**

    - Avoid hardcoding values that might need to change (e.g., default ports, file paths, timeouts, feature flags).
    - Integrate with the Configuration Module (F4) via `ConfigService` for accessing such values. Define clear configuration keys.

8.  **Performance Consciousness:**

    - Write efficient code by default, but **avoid premature optimization**.
    - Be mindful of performance implications in critical paths (e.g., request handling, core IOC lookups).
    - Understand the cost of operations: avoid unnecessary allocations (cloning `String`s in loops), prefer efficient data structures, be aware of async/await overhead.
    - Use `async/await` appropriately for I/O-bound tasks; avoid blocking threads in async contexts.
    - Write code that is _amenable_ to profiling and benchmarking later if performance becomes an issue.

9.  **Leverage the Ecosystem Wisely:**

    - **Prioritize using well-maintained, community-accepted libraries** (like `tokio`, `hyper`, `serde`, `config`, `tracing`, `log`, `clap`, `sqlx`, `redis-rs`, `syn`, `quote`, etc.) for standard tasks.
    - Do not reinvent the wheel unless there's a compelling reason (e.g., the existing libraries don't fit the architecture, have significant performance drawbacks for the specific use case, or introduce undesirable dependencies).
    - When adding a dependency, consider its transitive dependencies and potential impact on compile times and final binary size.

10. **Prioritization & Trade-offs:**
    - In general, prioritize **correctness, clarity, maintainability, and architectural consistency** over micro-optimizations, unless performance requirements in a specific area are explicitly stated and measured.
    - Be prepared to make documented trade-offs (e.g., sacrificing absolute flexibility for a simpler unified configuration).
