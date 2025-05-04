// Create or modify this file to potentially include a general error type
// or ensure IocError is handled correctly if defined in ioc.rs

// Example: Define a general SummerError enum
#[derive(Debug)]
pub enum SummerError {
    Io(std::io::Error),
    Config(String),            // Example config error variant
    Ioc(super::ioc::IocError), // Wrap IocError
    Other(String),
}

// Implement From traits for easier error conversion
impl From<std::io::Error> for SummerError {
    fn from(err: std::io::Error) -> Self {
        SummerError::Io(err)
    }
}

impl From<super::ioc::IocError> for SummerError {
    fn from(err: super::ioc::IocError) -> Self {
        SummerError::Ioc(err)
    }
}

// Implement std::error::Error and std::fmt::Display for SummerError
// ...
