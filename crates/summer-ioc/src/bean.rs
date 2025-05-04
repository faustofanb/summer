// Remove the definitions of BeanScope, BeanDependency, BeanDefinition, ComponentDefinitionProvider
// They are now in summer-core and re-exported in lib.rs

// Keep/adapt BeanDefinitionBuilder if needed, using imported types
// Example:
use crate::{BeanDefinition, BeanDependency, BeanScope, IocError}; // Use re-exported types from lib.rs

/// Builder for creating BeanDefinition instances.
#[derive(Default)]
pub struct BeanDefinitionBuilder {
    // ... fields ...
}
// ... impl ...
