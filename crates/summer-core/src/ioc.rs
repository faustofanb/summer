use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// --- Error Definition ---
#[derive(Debug)]
pub enum IocError {
    BeanNotFound(String),
    CircularDependency(String),
    DependencyError(String),
    ConfigError(String),
    Other(String),
}

// --- Bean Definitions ---

/// Bean 的作用域
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BeanScope {
    Singleton,
    // Prototype, // TODO
    // Request,   // TODO
}

impl FromStr for BeanScope {
    type Err = IocError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "singleton" => Ok(BeanScope::Singleton),
            _ => Err(IocError::Other(format!("Invalid bean scope: {}", s))),
        }
    }
}

/// 表示 Bean 的依赖项
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeanDependency {
    pub type_id: TypeId,
    pub field_name: Option<String>,
    pub required: bool,
}

/// Represents the definition of a bean.
#[derive(Clone)]
pub struct BeanDefinition {
    pub name: String,
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub scope: BeanScope,
    pub dependencies: Vec<BeanDependency>,
    pub init_method: Option<String>,
    pub destroy_method: Option<String>,
    pub factory_fn: fn(
        dependencies: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError>,
}

impl BeanDefinition {
    pub fn new(
        name: String,
        type_id: TypeId,
        type_name: &'static str,
        factory_fn: fn(
            dependencies: Vec<Arc<dyn Any + Send + Sync>>,
        ) -> Result<Arc<dyn Any + Send + Sync>, IocError>,
        dependencies: Vec<BeanDependency>,
    ) -> Self {
        Self {
            name,
            type_id,
            type_name,
            scope: BeanScope::Singleton,
            dependencies,
            init_method: None,
            destroy_method: None,
            factory_fn,
        }
    }
}

/// Trait implemented by components to provide their BeanDefinition.
/// Typically implemented by the `#[derive(Component)]` macro.
pub trait ComponentDefinitionProvider {
    fn get_bean_definition() -> BeanDefinition;
}

// --- IoC Container ---

/// The main Inversion of Control container.
#[derive(Default)]
pub struct IocContainer {
    /// Stores bean definitions, keyed by bean name.
    definitions: Mutex<HashMap<String, BeanDefinition>>,
    /// Stores singleton instances, keyed by bean name.
    singletons: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// Stores bean definitions, keyed by TypeId for type-based lookup.
    definitions_by_type: Mutex<HashMap<TypeId, Vec<String>>>,
    /// Tracks beans currently being created to detect circular dependencies.
    currently_creating: Mutex<HashSet<String>>,
}

impl IocContainer {
    /// Creates a new, empty IocContainer.
    pub fn new() -> Self {
        Default::default()
    }

    /// Registers a bean definition with the container.
    pub fn register_bean(&self, definition: BeanDefinition) -> Result<(), IocError> {
        let mut definitions = self.definitions.lock().unwrap();
        let mut definitions_by_type = self.definitions_by_type.lock().unwrap();

        if definitions.contains_key(&definition.name) {
            return Err(IocError::Other(format!(
                "Bean with name '{}' already registered.",
                definition.name
            )));
        }

        let name = definition.name.clone();
        let type_id = definition.type_id;

        definitions.insert(name.clone(), definition);
        definitions_by_type.entry(type_id).or_default().push(name);

        Ok(())
    }

    /// Retrieves a bean instance by name.
    /// Handles singleton creation and caching.
    pub fn get_bean<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, IocError> {
        let type_id = TypeId::of::<T>();

        {
            let singletons = self.singletons.lock().unwrap();
            if let Some(instance) = singletons.get(name) {
                return instance.clone().downcast::<T>().map_err(|_| {
                    IocError::Other(format!("Bean '{}' is not of the requested type", name))
                });
            }
        }

        {
            let mut creating = self.currently_creating.lock().unwrap();
            if !creating.insert(name.to_string()) {
                return Err(IocError::CircularDependency(format!(
                    "Circular dependency detected while creating bean '{}'",
                    name
                )));
            }
        }

        let definition = {
            let definitions = self.definitions.lock().unwrap();
            definitions.get(name).cloned().ok_or_else(|| {
                self.currently_creating.lock().unwrap().remove(name);
                IocError::BeanNotFound(name.to_string())
            })?
        };

        if definition.type_id != type_id {
            self.currently_creating.lock().unwrap().remove(name);
            return Err(IocError::Other(format!(
                "Bean definition '{}' has type {:?} but requested type was {:?}",
                name,
                definition.type_name,
                std::any::type_name::<T>()
            )));
        }

        if definition.scope == BeanScope::Singleton {
            {
                let singletons = self.singletons.lock().unwrap();
                if let Some(instance) = singletons.get(name) {
                    self.currently_creating.lock().unwrap().remove(name);
                    return instance.clone().downcast::<T>().map_err(|_| {
                        IocError::Other(format!("Bean '{}' is not of the requested type", name))
                    });
                }
            }

            let mut resolved_deps = Vec::new();
            for dep in &definition.dependencies {
                let dep_instance = self.get_bean_by_type_id(dep.type_id)?;
                resolved_deps.push(dep_instance);
            }

            let instance_result = (definition.factory_fn)(resolved_deps);

            self.currently_creating.lock().unwrap().remove(name);

            match instance_result {
                Ok(instance) => {
                    {
                        let mut singletons = self.singletons.lock().unwrap();
                        singletons.insert(name.to_string(), instance.clone());
                    }
                    instance.downcast::<T>().map_err(|_| {
                        IocError::Other(format!("Failed to downcast created bean '{}'", name))
                    })
                }
                Err(e) => Err(e),
            }
        } else {
            self.currently_creating.lock().unwrap().remove(name);
            Err(IocError::Other(format!(
                "Bean scope '{:?}' not yet supported for bean '{}'",
                definition.scope, name
            )))
        }
    }

    fn get_bean_by_type_id(&self, type_id: TypeId) -> Result<Arc<dyn Any + Send + Sync>, IocError> {
        let definitions_by_type = self.definitions_by_type.lock().unwrap();
        let potential_names = definitions_by_type.get(&type_id).ok_or_else(|| {
            IocError::BeanNotFound(format!("No bean found for type ID {:?}", type_id))
        })?;

        if potential_names.is_empty() {
            return Err(IocError::BeanNotFound(format!(
                "No bean found for type ID {:?}",
                type_id
            )));
        }

        let bean_name = potential_names[0].clone();
        drop(definitions_by_type);

        self.get_bean_instance_internal(&bean_name)
    }

    fn get_bean_instance_internal(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError> {
        {
            let singletons = self.singletons.lock().unwrap();
            if let Some(instance) = singletons.get(name) {
                return Ok(instance.clone());
            }
        }

        {
            let mut creating = self.currently_creating.lock().unwrap();
            if !creating.insert(name.to_string()) {
                return Err(IocError::CircularDependency(format!(
                    "Circular dependency detected while creating bean '{}'",
                    name
                )));
            }
        }

        let definition = {
            let definitions = self.definitions.lock().unwrap();
            definitions.get(name).cloned().ok_or_else(|| {
                self.currently_creating.lock().unwrap().remove(name);
                IocError::BeanNotFound(name.to_string())
            })?
        };

        if definition.scope == BeanScope::Singleton {
            {
                let singletons = self.singletons.lock().unwrap();
                if let Some(instance) = singletons.get(name) {
                    self.currently_creating.lock().unwrap().remove(name);
                    return Ok(instance.clone());
                }
            }

            let mut resolved_deps = Vec::new();
            for dep in &definition.dependencies {
                let dep_instance = self.get_bean_by_type_id(dep.type_id)?;
                resolved_deps.push(dep_instance);
            }

            let instance_result = (definition.factory_fn)(resolved_deps);

            self.currently_creating.lock().unwrap().remove(name);

            match instance_result {
                Ok(instance) => {
                    {
                        let mut singletons = self.singletons.lock().unwrap();
                        singletons.insert(name.to_string(), instance.clone());
                    }
                    Ok(instance)
                }
                Err(e) => Err(e),
            }
        } else {
            self.currently_creating.lock().unwrap().remove(name);
            Err(IocError::Other(format!(
                "Bean scope '{:?}' not yet supported for bean '{}'",
                definition.scope, name
            )))
        }
    }
}
