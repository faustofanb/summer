use crate::bean::BeanDefinition;
use crate::config::ConfigResolver;
use crate::error::{ConfigError, IocError};
use crate::event::{ApplicationEvent, ApplicationEventMulticaster, ApplicationListener};
use crate::factory::{BeanFactory, BeanRegistry};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// 应用上下文
pub struct ApplicationContext {
    pub(crate) registry: Arc<BeanRegistry>,
    config_resolver: Arc<dyn ConfigResolver>,
    pub(crate) event_multicaster: ApplicationEventMulticaster,
}

impl ApplicationContext {
    /// 创建新的应用上下文
    pub(crate) fn new(
        registry: Arc<BeanRegistry>,
        config_resolver: Arc<dyn ConfigResolver>,
    ) -> Self {
        Self {
            registry,
            config_resolver,
            event_multicaster: ApplicationEventMulticaster::new(),
        }
    }

    /// 获取 Bean 实例
    pub fn get_bean<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.registry.get_bean::<T>()
    }

    /// 发布事件到所有相关监听器
    pub async fn publish_event<E: ApplicationEvent + 'static>(
        &self,
        event: &E,
    ) -> Result<(), IocError> {
        // 记录事件发布
        println!("Publishing event: {}", event.event_name());
        self.event_multicaster.publish_event(event).await
    }

    /// 添加事件监听器
    pub async fn add_application_listener<E: ApplicationEvent + 'static>(
        &self,
        listener: Box<dyn ApplicationListener<E> + Send + Sync>,
    ) {
        self.event_multicaster
            .add_application_listener(listener)
            .await;
    }

    /// 解析配置值
    pub fn resolve_config<T: serde::de::DeserializeOwned + 'static>(
        &self,
        path: &str,
    ) -> Result<T, ConfigError> {
        let value_str = self
            .config_resolver
            .resolve(path)?
            .ok_or_else(|| ConfigError::NotFound(path.to_string()))?;
        serde_json::from_str(&value_str)
            .map_err(|e| ConfigError::ParseError(format!("JSON parse error: {}", e)))
    }

    /// 解析属性值
    pub fn resolve_property<T: DeserializeOwned + 'static>(
        &self,
        key: &str,
    ) -> Result<T, ConfigError> {
        self.resolve_config(key)
    }

    /// 获取内部注册表
    pub fn get_registry(&self) -> &BeanRegistry {
        &self.registry
    }

    /// 获取内部事件多播器
    pub fn get_event_multicaster(&self) -> &ApplicationEventMulticaster {
        &self.event_multicaster
    }
}

#[async_trait]
impl BeanFactory for ApplicationContext {
    /// 根据类型获取 Bean 实例
    fn get_bean<T: Any + Send + Sync>(&self) -> Result<Arc<T>, IocError> {
        let type_id = TypeId::of::<T>();
        self.registry
            .singletons
            .get(&type_id)
            .and_then(|bean| bean.clone().downcast::<T>().ok())
            .ok_or_else(|| {
                IocError::BeanNotFound(format!(
                    "No singleton bean of type {:?} found",
                    std::any::type_name::<T>()
                ))
            })
    }

    /// 根据类型 ID 获取 Bean 实例
    fn get_bean_by_type_id(&self, type_id: TypeId) -> Result<Arc<dyn Any + Send + Sync>, IocError> {
        self.registry
            .singletons
            .get(&type_id)
            .cloned()
            .ok_or_else(|| {
                IocError::BeanNotFound(format!("No singleton bean with TypeId {:?} found", type_id))
            })
    }

    /// 根据名称和类型获取 Bean 实例 (Optimized)
    fn get_bean_by_name<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, IocError> {
        if let Some(type_id) = self.registry.get_type_id_by_name(name) {
            self.registry
                .singletons
                .get(&type_id)
                .and_then(|bean| bean.clone().downcast::<T>().ok())
                .ok_or_else(|| {
                    if self.registry.get_definitions().contains_key(&type_id) {
                         IocError::BeanNotFound(format!(
                            "Bean definition found for name '{}' (TypeId: {:?}), but no singleton instance of the correct type {:?} exists or is ready.",
                            name, type_id, std::any::type_name::<T>()
                        ))
                    } else {
                         IocError::BeanNotFound(format!(
                            "Internal inconsistency: Name '{}' maps to TypeId {:?}, but no definition found.", name, type_id
                        ))
                    }
                })
        } else {
            Err(IocError::BeanNotFound(format!(
                "No bean definition found for name '{}'",
                name
            )))
        }
    }

    /// 检查是否包含指定类型的 Bean 定义或实例
    fn contains_bean<T: Any + Send + Sync>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.registry.get_definitions().contains_key(&type_id)
            || self.registry.singletons.contains_key(&type_id)
    }

    /// 检查是否包含指定名称的 Bean 定义或实例 (Optimized)
    fn contains_bean_by_name(&self, name: &str) -> bool {
        self.registry.get_type_id_by_name(name).is_some()
    }

    /// 获取配置解析器
    fn get_config_resolver(&self) -> Arc<dyn ConfigResolver> {
        self.config_resolver.clone()
    }
}
