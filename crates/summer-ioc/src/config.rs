use crate::error::ConfigError; // Import from error module
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for resolving configuration values.
pub trait ConfigResolver: Send + Sync {
    /// Resolves a configuration value by key.
    /// Returns the value as a String if found, otherwise None.
    /// Errors during resolution (e.g., parsing errors in underlying sources) should be returned as Err.
    fn resolve(&self, key: &str) -> Result<Option<String>, ConfigError>;

    /// Resolves a configuration value by key or returns a default value.
    fn resolve_or_default(&self, key: &str, default_value: String) -> Result<String, ConfigError> {
        self.resolve(key).map(|opt| opt.unwrap_or(default_value))
    }
}

/// A `ConfigResolver` implementation that reads properties from a HashMap.
#[derive(Default)]
pub struct MemoryConfigResolver {
    properties: Mutex<HashMap<String, String>>,
}

impl MemoryConfigResolver {
    pub fn new() -> Self {
        Self {
            properties: Mutex::new(HashMap::new()),
        }
    }

    pub fn set(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        self.properties
            .lock()
            .unwrap()
            .insert(key.to_string(), value);
        Ok(())
    }
}

impl ConfigResolver for MemoryConfigResolver {
    fn resolve(&self, key: &str) -> Result<Option<String>, ConfigError> {
        Ok(self.properties.lock().unwrap().get(key).cloned())
    }
}

/// A `ConfigResolver` implementation that delegates to a list of other resolvers.
pub struct CompositeConfigResolver {
    resolvers: Vec<Arc<dyn ConfigResolver>>,
}

impl Default for CompositeConfigResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeConfigResolver {
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    pub fn add_resolver(&mut self, resolver: Arc<dyn ConfigResolver>) -> &mut Self {
        self.resolvers.push(resolver);
        self
    }
}

impl ConfigResolver for CompositeConfigResolver {
    fn resolve(&self, key: &str) -> Result<Option<String>, ConfigError> {
        for resolver in &self.resolvers {
            match resolver.resolve(key) {
                Ok(Some(value)) => return Ok(Some(value)),
                Ok(None) => continue,
                Err(e) => {
                    if matches!(e, ConfigError::NotFound(_)) {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(None)
    }
}
