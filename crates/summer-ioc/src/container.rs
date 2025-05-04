// Import necessary items from summer_core, including the new aliases
use crate::definition::BeanDefinition;
use crate::error::IocError;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use summer_core::{
    BeanConstructorResult, // Result<BeanInstance, ConstructorError>
    BeanDefinitionMetadata,
    BeanInstance,    // Arc<dyn Any + Send + Sync>
    BeanProvider,    // The trait
    BeanProviderRef, // Arc<dyn BeanProvider + Send + Sync>
    ConstructorError,
};

/// The core Inversion of Control (IoC) container.
pub struct IocContainer {
    definitions: RwLock<HashMap<String, BeanDefinition>>,
    // Use the BeanInstance alias from summer_core
    singleton_instances: RwLock<HashMap<String, BeanInstance>>,
    beans_by_type: RwLock<HashMap<TypeId, Vec<String>>>,
    initialized: RwLock<bool>,
    currently_in_creation: RwLock<HashSet<String>>,
    // Store self as Arc<Self> to pass to constructors
    // Use BeanProviderRef for the type
    self_arc: RwLock<Option<BeanProviderRef>>,
}

// Implement the BeanProvider trait for IocContainer
impl BeanProvider for IocContainer {
    // Use BeanInstance alias in the return type
    fn get_bean_by_typeid(&self, type_id: TypeId) -> Result<BeanInstance, ConstructorError> {
        // Ensure container is initialized before attempting to get beans.
        if !*self.initialized.read() {
            // Return ConstructorError, which is Box<dyn Error>
            return Err(ConstructorError::ContainerNotInitialized);
        }

        let bean_names = {
            let beans_by_type_guard = self.beans_by_type.read();
            // Find the bean names associated with the TypeId
            beans_by_type_guard.get(&type_id).cloned()
        };

        match bean_names {
            Some(names) => {
                if names.len() == 1 {
                    // Exactly one bean found for this type. Attempt to retrieve it.
                    // Use the internal method that returns Result<BeanInstance, IocError>
                    self.get_bean_by_name_any(&names[0])
                        .map_err(|_| ConstructorError::BaseError) // Map IocError to ConstructorError
                } else if names.is_empty() {
                    // This case should ideally not happen if beans_by_type is consistent,
                    // but handle it defensively.
                    Err(ConstructorError::BeanNotFoundByType(type_id))
                } else {
                    // Multiple beans found for the same type. This is an error unless
                    // mechanisms like @Primary or @Qualifier are implemented.
                    Err(ConstructorError::MultipleBeansFound(type_id))
                }
            }
            None => {
                // No bean definition found for this type ID.
                Err(ConstructorError::BeanNotFoundByType(type_id))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    // get_bean<T> uses the default implementation from summer_core::provider::BeanProvider
}

impl IocContainer {
    /// Creates a new, empty IocContainer wrapped in an Arc.
    // The return type Arc<Self> implicitly works with BeanProviderRef due to the blanket impl
    pub fn new() -> Arc<Self> {
        let container = Arc::new(IocContainer {
            definitions: RwLock::new(HashMap::new()),
            singleton_instances: RwLock::new(HashMap::new()),
            beans_by_type: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
            currently_in_creation: RwLock::new(HashSet::new()),
            self_arc: RwLock::new(None), // Initialize as None
        });
        // Store the Arc<Self> within the container itself, it acts as BeanProviderRef
        *container.self_arc.write() = Some(container.clone());
        container
    }

    /// Initializes the container by collecting bean metadata.
    // Takes Arc<Self> which acts as BeanProviderRef
    pub fn initialize(self: &Arc<Self>) -> Result<(), IocError> {
        let mut initialized_guard = self.initialized.write();
        if *initialized_guard {
            // Already initialized, do nothing.
            return Ok(());
        }

        log::info!("Initializing Summer IOC container..."); // Added logging

        for metadata in summer_core::inventory::iter::<BeanDefinitionMetadata>() {
            let bean_name = metadata.bean_name.to_string();
            let bean_type_id = (metadata.bean_type_id)();
            log::debug!(
                "Registering bean definition: name='{}', type_id={:?}",
                bean_name,
                bean_type_id
            ); // Added logging
               // metadata.constructor is already BeanConstructor type
            let definition =
                BeanDefinition::new(bean_name.clone(), bean_type_id, metadata.constructor);
            // Use internal registration which handles locking
            self.register_bean_definition_internal(definition)?;
        }

        *initialized_guard = true;
        log::info!("Summer IOC container initialized successfully."); // Added logging
        Ok(())
    }

    /// Internal registration logic, now takes &self
    fn register_bean_definition_internal(
        &self,
        definition: BeanDefinition,
    ) -> Result<(), IocError> {
        let bean_name = definition.bean_name.clone();
        let bean_type_id = definition.bean_type_id;

        // Acquire write locks
        let mut definitions_guard = self.definitions.write();
        let mut beans_by_type_guard = self.beans_by_type.write();

        // Check for duplicate bean names
        if definitions_guard.contains_key(&bean_name) {
            log::error!(
                "Bean registration failed: Bean with name '{}' already exists.",
                bean_name
            ); // Added logging
            return Err(IocError::BeanAlreadyExists(bean_name));
        }

        // Insert definition
        definitions_guard.insert(bean_name.clone(), definition);

        // Update type mapping
        beans_by_type_guard
            .entry(bean_type_id)
            .or_default()
            .push(bean_name.clone());

        log::trace!("Successfully registered bean definition: '{}'", bean_name); // Added logging
        Ok(())
    }

    /// Registers a bean definition. Takes &self.
    /// Allows registration even after initialization, e.g., for dynamic beans.
    pub fn register_bean_definition(&self, definition: BeanDefinition) -> Result<(), IocError> {
        // No check for initialized needed here, allow dynamic registration.
        log::info!(
            "Dynamically registering bean definition: '{}'",
            definition.bean_name
        ); // Added logging
        self.register_bean_definition_internal(definition)
    }

    /// Retrieves a bean instance by name, returning BeanInstance.
    fn get_bean_by_name_any(&self, name: &str) -> Result<BeanInstance, IocError> {
        // Check initialization status first.
        if !*self.initialized.read() {
            log::warn!(
                "Attempted to get bean '{}' before container initialization.",
                name
            ); // Added logging
            return Err(IocError::ContainerNotInitialized);
        }

        // 1. Check singleton cache (Read Lock)
        {
            let instances_guard = self.singleton_instances.read();
            if let Some(instance_arc) = instances_guard.get(name) {
                log::trace!("Cache hit for bean '{}'", name); // Added logging
                return Ok(instance_arc.clone());
            }
            // Read lock is released here
        }

        log::trace!("Cache miss for bean '{}', attempting instantiation.", name); // Added logging

        // 2. Instantiate (needs BeanProviderRef)
        // Clone the BeanProviderRef stored internally.
        let self_provider_ref = self.self_arc.read().clone().ok_or_else(|| {
            // This indicates a critical internal error if None.
            log::error!(
                "Internal container error: self_arc is None during bean instantiation for '{}'.",
                name
            );
            IocError::InternalError("Container self_arc not initialized".to_string())
        })?;

        // Call the main instantiation logic, passing the BeanProviderRef
        self.instantiate_bean(self_provider_ref, name)
    }

    /// Retrieves a bean instance by its name, downcasting to the requested type T.
    pub fn get_bean_by_name<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, IocError> {
        let requested_type_id = TypeId::of::<T>();
        log::debug!(
            "Requesting bean by name: '{}', expected type: {:?}",
            name,
            requested_type_id
        ); // Added logging

        // Get the instance as BeanInstance first.
        let instance_any: BeanInstance = self.get_bean_by_name_any(name)?;

        // Attempt to downcast to the requested concrete type T.
        instance_any.downcast::<T>().map_err(|arc| {
            // Downcast failed, means the stored type doesn't match T.
            let stored_type_id = arc.type_id();
            log::error!(
                "Type mismatch for bean '{}': Requested type {:?}, but stored type is {:?}.",
                name,
                requested_type_id,
                stored_type_id
            ); // Added logging
            IocError::TypeMismatchError {
                bean_name: name.to_string(),
                requested: requested_type_id,
                stored: stored_type_id,
            }
        })
    }

    /// Internal helper to handle instantiation, locking, and caching. Takes BeanProviderRef.
    // Returns Result<BeanInstance, IocError>
    fn instantiate_bean(
        &self,                         // Keep &self for accessing container fields
        provider_ref: BeanProviderRef, // Pass BeanProviderRef for the constructor call
        name: &str,
    ) -> Result<BeanInstance, IocError> {
        // --- Cycle Detection Start (Write Lock on currently_in_creation) ---
        {
            let mut creating_guard = self.currently_in_creation.write();
            if !creating_guard.insert(name.to_string()) {
                // If insert returns false, the name was already present. Cycle detected.
                let cycle_path = creating_guard.iter().cloned().collect();
                log::error!(
                    "Dependency cycle detected while creating bean '{}'. Path: {:?}",
                    name,
                    cycle_path
                ); // Added logging
                return Err(IocError::DependencyCycle(name.to_string(), cycle_path));
            }
            log::trace!(
                "Starting creation of bean '{}'. Current creation path: {:?}",
                name,
                creating_guard.iter()
            ); // Added logging
               // Write lock is released here
        }

        // --- Get Definition (Read Lock on definitions) ---
        // Clone the definition to avoid holding the read lock during potentially long constructor call.
        let definition = {
            let definitions_guard = self.definitions.read();
            definitions_guard.get(name).cloned()
            // Read lock is released here
        };

        // --- Instantiate Bean (using the cloned definition) ---
        let bean_instance_result: Result<BeanInstance, IocError> = match definition {
            Some(def) => {
                log::debug!("Found definition for bean '{}'. Calling constructor.", name); // Added logging
                let constructor = def.constructor; // constructor is BeanConstructor type
                                                   // Pass the BeanProviderRef to the constructor.
                                                   // The constructor returns BeanConstructorResult (Result<BeanInstance, ConstructorError>)
                let instance_arc_any: BeanInstance = constructor(provider_ref) // Pass the BeanProviderRef
                    .map_err(|e| {
                        // Constructor returned an error (Box<dyn Error>)
                        log::error!("Failed to instantiate bean '{}': {}", name, e); // Added logging
                        IocError::InstantiationError {
                            bean_name: name.to_string(),
                            reason: e.to_string(),
                        }
                    })?; // Propagate error if constructor fails

                log::debug!(
                    "Successfully constructed bean instance for '{}'. Caching...",
                    name
                ); // Added logging

                // --- Store in Singleton Cache (Write Lock on singleton_instances) ---
                // Use Double-Checked Locking pattern: Check cache again after acquiring write lock.
                {
                    let mut instances_guard = self.singleton_instances.write();
                    if let Some(existing_instance) = instances_guard.get(name) {
                        // Another thread might have created and cached the instance while we were waiting for the lock.
                        log::trace!("Bean '{}' was already cached by another thread. Using cached instance.", name); // Added logging
                        Ok(existing_instance.clone()) // Use the existing instance
                    } else {
                        // Cache is still empty for this name, insert the newly created instance.
                        instances_guard.insert(name.to_string(), instance_arc_any.clone());
                        log::trace!("Bean '{}' successfully cached.", name); // Added logging
                        Ok(instance_arc_any) // Return the newly created instance
                    }
                    // Write lock is released here
                }
            }
            None => {
                log::error!(
                    "Bean definition not found for name '{}' during instantiation attempt.",
                    name
                ); // Added logging
                Err(IocError::BeanNotFoundByName(name.to_string()))
            }
        };

        // --- Cycle Detection End (Write Lock on currently_in_creation) ---
        // Remove the bean name from the set regardless of success or failure.
        {
            let mut creating_guard = self.currently_in_creation.write();
            creating_guard.remove(name);
            log::trace!(
                "Finished creation attempt for bean '{}'. Remaining in creation: {:?}",
                name,
                creating_guard.iter()
            ); // Added logging
               // Write lock is released here
        }

        bean_instance_result // Return the result (Ok(BeanInstance) or Err)
    }

    /// Retrieves a bean instance by its type T.
    pub fn get_bean<T: Any + Send + Sync>(&self) -> Result<Arc<T>, IocError> {
        let type_id = TypeId::of::<T>();
        // Find potential bean names for this type
        let bean_names = {
            let beans_by_type_read = self.beans_by_type.read();
            beans_by_type_read.get(&type_id).cloned() // Clone the Vec<String>
        };

        match bean_names {
            Some(names) => {
                if names.len() == 1 {
                    // If only one bean of this type, get it by name
                    self.get_bean_by_name::<T>(&names[0])
                } else if names.is_empty() {
                    // Should not happen if TypeId is in the map, but handle defensively
                    Err(IocError::BeanNotFoundByType(type_id))
                } else {
                    // Multiple beans found for the type
                    Err(IocError::MultipleBeansFound(type_id))
                }
            }
            None => {
                // No bean found for the type
                Err(IocError::BeanNotFoundByType(type_id))
            }
        }
    }
}
