use crate::bean::BeanDefinition;
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

/// 作用域策略特征
#[async_trait]
pub trait Scope: Send + Sync {
    async fn get_or_create<T: Any + Send + Sync + Clone>(
        &self,
        factory: Box<dyn Fn() -> T + Send + Sync>,
    ) -> Arc<T>;

    async fn remove<T: Any + Send + Sync>(&self);

    async fn clear(&self);
}

/// 单例作用域
pub struct SingletonScope {
    instances: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    definitions: RwLock<HashMap<TypeId, BeanDefinition>>,
}

impl SingletonScope {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            definitions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Scope for SingletonScope {
    async fn get_or_create<T: Any + Send + Sync + Clone>(
        &self,
        factory: Box<dyn Fn() -> T + Send + Sync>,
    ) -> Arc<T> {
        let type_id = TypeId::of::<T>();

        // 检查是否已存在
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(&type_id) {
                if let Ok(typed_arc) = instance.clone().downcast::<T>() {
                    return typed_arc;
                }
            }
        }

        // 创建新实例
        let mut instances = self.instances.write().await;
        let instance = Arc::new(factory());
        let definition = BeanDefinition {
            type_id,
            name: std::any::type_name::<T>().to_string(), // 使用类型名称作为默认名称
            scope: crate::bean::BeanScope::Singleton,
            autowire: true, // 默认开启自动装配
            primary: false,
            init_method: None,
            destroy_method: None,
            dependencies: Vec::new(),
            type_parameters: Vec::new(), // 添加类型参数字段
        };

        instances.insert(type_id, instance.clone());
        let mut definitions = self.definitions.write().await;
        definitions.insert(type_id, definition);

        instance
    }

    async fn remove<T: Any + Send + Sync>(&self) {
        let type_id = TypeId::of::<T>();
        let mut instances = self.instances.write().await;
        let mut definitions = self.definitions.write().await;
        instances.remove(&type_id);
        definitions.remove(&type_id);
    }

    async fn clear(&self) {
        let mut instances = self.instances.write().await;
        let mut definitions = self.definitions.write().await;
        instances.clear();
        definitions.clear();
    }
}

/// 请求作用域
pub struct RequestScope {
    request_instances: OnceCell<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Default for RequestScope {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestScope {
    pub fn new() -> Self {
        Self {
            request_instances: OnceCell::new(),
        }
    }

    pub async fn begin_request(&self) {
        self.request_instances
            .get_or_init(|| async { RwLock::new(HashMap::new()) })
            .await;
    }

    pub async fn end_request(&self) {
        if let Some(instances) = self.request_instances.get() {
            let mut map = instances.write().await;
            map.clear();
        }
    }
}

#[async_trait]
impl Scope for RequestScope {
    async fn get_or_create<T: Any + Send + Sync + Clone>(
        &self,
        factory: Box<dyn Fn() -> T + Send + Sync>,
    ) -> Arc<T> {
        let type_id = TypeId::of::<T>();

        let instances = self
            .request_instances
            .get_or_init(|| async { RwLock::new(HashMap::new()) })
            .await;

        // 检查是否已存在
        {
            let map = instances.read().await;
            if let Some(instance) = map.get(&type_id) {
                if let Ok(typed_arc) = instance.clone().downcast::<T>() {
                    return typed_arc;
                }
            }
        }

        // 创建新实例
        let mut map = instances.write().await;
        let instance = Arc::new(factory());
        map.insert(type_id, instance.clone());
        instance
    }

    async fn remove<T: Any + Send + Sync>(&self) {
        if let Some(instances) = self.request_instances.get() {
            let mut map = instances.write().await;
            map.remove(&TypeId::of::<T>());
        }
    }

    async fn clear(&self) {
        if let Some(instances) = self.request_instances.get() {
            let mut map = instances.write().await;
            map.clear();
        }
    }
}
