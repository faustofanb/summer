use crate::bean::BeanDefinition;
use crate::config::ConfigResolver;
use crate::error::IocError;
use crate::processor::{BeanFactoryPostProcessor, BeanPostProcessor};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Bean 工厂接口
#[async_trait::async_trait] // Add async_trait if methods become async
pub trait BeanFactory: Send + Sync {
    /// 根据类型获取 Bean 实例
    fn get_bean<T: Any + Send + Sync>(&self) -> Result<Arc<T>, IocError>;

    /// 根据类型 ID 获取 Bean 实例 (返回 dyn Any)
    fn get_bean_by_type_id(&self, type_id: TypeId) -> Result<Arc<dyn Any + Send + Sync>, IocError>;

    /// 根据名称和类型获取 Bean 实例
    // TODO: Implement name-based retrieval logic in the concrete implementation (ApplicationContext)
    fn get_bean_by_name<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, IocError>;

    /// 检查是否包含指定类型的 Bean 定义或实例
    fn contains_bean<T: Any + Send + Sync>(&self) -> bool;

    /// 检查是否包含指定名称的 Bean 定义或实例
    // TODO: Implement name-based check logic in the concrete implementation (ApplicationContext)
    fn contains_bean_by_name(&self, name: &str) -> bool;

    /// 获取配置解析器
    fn get_config_resolver(&self) -> Arc<dyn ConfigResolver>;

    // TODO: Potentially add methods for getting beans implementing a specific trait
    // fn get_beans_of_trait<T: ?Sized + Any + Send + Sync>(&self) -> Result<Vec<Arc<T>>, IocError>;
}

/// BeanRegistry - 存储所有 Bean 实例和定义的容器
pub struct BeanRegistry {
    pub(crate) singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    definitions: HashMap<TypeId, BeanDefinition>,
    name_to_type_id: HashMap<String, TypeId>, // Added for name lookup optimization
    // Use Arc instead of Box for cloneable processors
    pub(crate) post_processors: Option<Vec<Arc<dyn BeanPostProcessor>>>,
    // Keep FactoryPostProcessors as Box for now, assuming they don't need cloning in the same way
    pub(crate) factory_post_processors: Option<Vec<Box<dyn BeanFactoryPostProcessor>>>,
}

impl Default for BeanRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanRegistry {
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            definitions: HashMap::new(),
            name_to_type_id: HashMap::new(), // Initialize map
            post_processors: None,
            factory_post_processors: None,
        }
    }

    pub fn register<T: Any + Send + Sync>(&mut self, instance: Arc<T>, definition: BeanDefinition) {
        let type_id = TypeId::of::<T>();
        self.singletons
            .insert(type_id, instance as Arc<dyn Any + Send + Sync>);
        self.definitions.insert(type_id, definition);
    }

    /// 添加 Bean 后处理器
    pub fn add_bean_post_processor(&mut self, processor: Arc<dyn BeanPostProcessor>) {
        if self.post_processors.is_none() {
            self.post_processors = Some(Vec::new());
        }

        if let Some(processors) = &mut self.post_processors {
            processors.push(processor);
        }
    }

    /// 添加 Bean 工厂后处理器
    pub fn add_bean_factory_post_processor(
        &mut self,
        processor: Box<dyn BeanFactoryPostProcessor>,
    ) {
        if self.factory_post_processors.is_none() {
            self.factory_post_processors = Some(Vec::new());
        }

        if let Some(processors) = &mut self.factory_post_processors {
            processors.push(processor);
        }
    }

    /// 注册 Bean 定义
    pub fn add_bean_definition(&mut self, definition: BeanDefinition) -> Result<(), IocError> {
        let type_id = definition.type_id;
        let name = definition.name.clone(); // Clone name for map insertion

        if self.definitions.contains_key(&type_id) {
            return Err(IocError::BeanAlreadyExists(format!(
                "Bean definition for type {:?} already exists",
                definition.name
            )));
        }
        if self.name_to_type_id.contains_key(&name) {
            // Handle name collision - decide on behavior (error, overwrite, ignore?)
            // For now, let's return an error.
            return Err(IocError::BeanAlreadyExists(format!(
                "Bean definition with name '{}' already exists (maps to TypeId {:?})",
                name, self.name_to_type_id[&name]
            )));
        }

        self.definitions.insert(type_id, definition);
        self.name_to_type_id.insert(name, type_id); // Populate name map
        Ok(())
    }

    /// 获取 Bean 实例
    pub fn get_bean<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        self.singletons
            .get(&type_id)
            .and_then(|bean| bean.clone().downcast::<T>().ok())
    }

    /// 获取 Bean 定义
    pub fn get_bean_definition<T: Any>(&self) -> Option<&BeanDefinition> {
        let type_id = TypeId::of::<T>();
        self.definitions.get(&type_id)
    }

    /// 获取所有 Bean 定义 (只读访问)
    pub fn get_definitions(&self) -> &HashMap<TypeId, BeanDefinition> {
        &self.definitions
    }

    /// 获取所有 Bean 定义 (可变访问, 供 FactoryPostProcessor 使用)
    pub fn get_definitions_mut(&mut self) -> &mut HashMap<TypeId, BeanDefinition> {
        &mut self.definitions
    }

    /// 根据名称获取 TypeId
    pub(crate) fn get_type_id_by_name(&self, name: &str) -> Option<TypeId> {
        self.name_to_type_id.get(name).cloned()
    }

    /// 获取 Bean 工厂后处理器 (只读)
    pub(crate) fn get_factory_post_processors(
        &self,
    ) -> Option<&Vec<Box<dyn BeanFactoryPostProcessor>>> {
        self.factory_post_processors.as_ref()
    }

    /// 获取 Bean 后处理器 (只读, 返回 Arc)
    pub(crate) fn get_bean_post_processors(&self) -> Option<&Vec<Arc<dyn BeanPostProcessor>>> {
        self.post_processors.as_ref()
    }
}
