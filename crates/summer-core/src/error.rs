use std::any::TypeId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConstructorError {
    #[error("Constructor error.")]
    BaseError,

    #[error("Constructor error with message: {0}.")]
    BaseMsgError(String),
    
    #[error("Container has not been initialized yet.")]
    ContainerNotInitialized,

    #[error("Bean definition not found for type ID: {0:?}")]
    BeanNotFoundByType(TypeId),

    #[error("Bean definition not found for name: {0}")]
    BeanNotFoundByName(String),
    
    #[error(
        "Multiple beans found for type ID: {0:?}. Use qualifiers or @Primary to disambiguate."
    )]
    MultipleBeansFound(TypeId),
}