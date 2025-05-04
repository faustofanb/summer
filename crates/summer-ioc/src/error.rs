use std::any::TypeId;
use thiserror::Error;

/// Errors that can occur within the Summer IOC container.
#[derive(Error, Debug)]
pub enum IocError {
    #[error("Container has not been initialized yet.")]
    ContainerNotInitialized,

    #[error("Bean with name '{0}' already exists.")]
    BeanAlreadyExists(String),

    #[error("Bean definition not found for name: {0}")]
    BeanNotFoundByName(String),

    #[error("Bean definition not found for type ID: {0:?}")]
    BeanNotFoundByType(TypeId),

    #[error(
        "Multiple beans found for type ID: {0:?}. Use qualifiers or @Primary to disambiguate."
    )]
    MultipleBeansFound(TypeId),

    #[error("Dependency cycle detected while creating bean '{0}'. Path: {1:?}")]
    DependencyCycle(String, Vec<String>), // Store the detected cycle path

    #[error("Failed to instantiate bean '{bean_name}': {reason}")]
    InstantiationError { bean_name: String, reason: String },

    #[error("Type mismatch for bean '{bean_name}': Requested {requested:?}, but found {stored:?}")]
    TypeMismatchError {
        bean_name: String,
        requested: TypeId,
        stored: TypeId,
    },

    // Add the missing variant
    #[error("Internal container error: {0}")]
    InternalError(String),
}
