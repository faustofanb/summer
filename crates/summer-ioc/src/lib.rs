// Re-export core types from summer-core for the public API of summer-ioc
pub use summer_core::{
    BeanDefinition, BeanDependency, BeanScope, ComponentDefinitionProvider, IocError,
};

// Define modules and export other public items from summer-ioc itself
pub mod builder;
pub mod context;
pub mod factory;
pub mod processor;

pub use builder::ApplicationContextBuilder;
pub use context::ApplicationContext;
pub use factory::{BeanFactory, BeanRegistry};
pub use processor::{BeanFactoryPostProcessor, BeanPostProcessor};
