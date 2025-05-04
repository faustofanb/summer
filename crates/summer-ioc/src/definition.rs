use std::any::TypeId;
// Import the type aliases from summer_core
use summer_core::BeanConstructor;

/// Represents the definition of a bean within the IoC container.
#[derive(Clone)]
pub struct BeanDefinition {
    pub bean_name: String,
    pub bean_type_id: TypeId,
    // Use the type alias from summer_core
    pub constructor: BeanConstructor,
}

impl BeanDefinition {
    pub fn new(
        bean_name: String,
        bean_type_id: TypeId,
        constructor: BeanConstructor, // Use the alias here too
    ) -> Self {
        BeanDefinition {
            bean_name,
            bean_type_id,
            constructor,
        }
    }
}
