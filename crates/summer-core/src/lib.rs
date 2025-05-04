mod error;

use std::any::{Any, TypeId};
use std::sync::Arc;

// --- Type Aliases ---
/// A shared, thread-safe, dynamically-typed bean instance.
pub type BeanInstance = Arc<dyn Any + Send + Sync>;

/// Result type returned by bean constructors.
pub type BeanConstructorResult = Result<BeanInstance, ConstructorError>;

/// A shared, thread-safe reference to a bean provider (typically the IoC container itself).
pub type BeanProviderRef = Arc<dyn BeanProvider + Send + Sync>;

/// Function signature for a bean constructor.
/// Takes a reference to the bean provider and returns a result containing the bean instance or an error.
pub type BeanConstructor = fn(provider: BeanProviderRef) -> BeanConstructorResult;

/// Function signature for getting a TypeId.
pub type TypeIdGetter = fn() -> TypeId;

// Forward declaration for the trait alias
pub trait BeanProvider: Any + Send + Sync {
    /// Retrieves a bean instance by its TypeId.
    /// Returns an error if the bean is not found, multiple beans are found,
    /// or if the container is not initialized.
    fn get_bean_by_typeid(&self, type_id: TypeId) -> Result<BeanInstance, ConstructorError>;

    // Removed get_bean<T> due to dyn safety issues.
    // Callers should use get_bean_by_typeid and downcast manually.

    /// Returns the provider as a `dyn Any` reference.
    fn as_any(&self) -> &dyn Any;
}
// --- Metadata Struct --- (Moved from metadata.rs for simplicity, could be separate)
pub struct BeanDefinitionMetadata {
    pub bean_name: &'static str,
    pub bean_type_id: TypeIdGetter,
    pub constructor: BeanConstructor,
}

inventory::collect!(BeanDefinitionMetadata);

// --- Public Exports ---
pub use inventory;
pub use error::ConstructorError;