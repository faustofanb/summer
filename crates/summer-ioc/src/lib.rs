//! Summer IOC (Inversion of Control) Core Crate
//! Provides the foundation for dependency injection and component management.

pub mod container;
pub mod definition;
pub mod error;

// Re-export key types for easier access
pub use container::IocContainer;
pub use definition::BeanDefinition;
pub use error::IocError;

// --- Traits and other public items to be added later ---
// pub trait BeanFactory { ... }
// pub trait ApplicationContext: BeanFactory { ... }
// pub trait BeanPostProcessor { ... }
// pub trait Aware { ... } // e.g., BeanNameAware, ApplicationContextAware
