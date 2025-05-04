// Define modules within summer-core
pub mod error;
pub mod ioc; // Make the ioc module public

// Re-export key types for easier use
pub use error::SummerError; // Assuming you have a core error type
pub use ioc::{
    BeanDefinition, BeanDependency, BeanScope, ComponentDefinitionProvider, IocContainer, IocError,
};

// Other core functionalities can be added here
