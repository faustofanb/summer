use crate::IocError;
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 应用事件 trait
pub trait ApplicationEvent: Any + Send + Sync {
    /// 获取事件名称
    fn event_name(&self) -> &'static str;

    /// 获取事件时间戳
    fn timestamp(&self) -> u64;

    /// 获取事件源（可选）
    fn source(&self) -> Option<&dyn Any> {
        None
    }

    /// 转换为任意类型
    fn as_any(&self) -> &dyn Any;
}

/// 应用事件监听器
#[async_trait]
pub trait ApplicationListener<E: ApplicationEvent>: Send + Sync {
    /// 处理事件
    async fn on_application_event(&self, event: &E) -> Result<(), IocError>;

    /// 获取支持的事件类型ID
    fn supports_event_type(&self) -> TypeId {
        TypeId::of::<E>()
    }
}

/// 类型擦除的事件监听器接口
#[async_trait]
trait AnyApplicationListener: Send + Sync {
    /// 处理任意事件
    async fn on_any_event(&self, event: &dyn ApplicationEvent) -> Result<(), IocError>;

    /// 获取支持的事件类型ID
    fn get_event_type(&self) -> TypeId;
}

/// 类型擦除的事件监听器包装器
struct TypeErasedListener<E: ApplicationEvent + 'static> {
    listener: Box<dyn ApplicationListener<E> + Send + Sync>,
    event_type: TypeId,
}

#[async_trait]
impl<E: ApplicationEvent + 'static> AnyApplicationListener for TypeErasedListener<E> {
    async fn on_any_event(&self, event: &dyn ApplicationEvent) -> Result<(), IocError> {
        // 尝试将通用事件转型为特定事件类型
        if let Some(typed_event) = event.as_any().downcast_ref::<E>() {
            self.listener.on_application_event(typed_event).await
        } else {
            Err(IocError::EventHandlingError(format!(
                "事件类型不匹配: 期望 {}, 获得 {}",
                std::any::type_name::<E>(),
                event.event_name()
            )))
        }
    }

    fn get_event_type(&self) -> TypeId {
        self.event_type
    }
}

/// 应用事件多播器
pub struct ApplicationEventMulticaster {
    listeners: RwLock<HashMap<TypeId, Vec<Box<dyn AnyApplicationListener>>>>,
}

impl Default for ApplicationEventMulticaster {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationEventMulticaster {
    /// 创建新的事件多播器
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }

    /// 添加事件监听器
    pub async fn add_application_listener<E: ApplicationEvent + 'static>(
        &self,
        listener: Box<dyn ApplicationListener<E> + Send + Sync>,
    ) {
        let type_id = TypeId::of::<E>();

        let erased_listener = Box::new(TypeErasedListener {
            listener,
            event_type: type_id,
        });

        let mut listeners = self.listeners.write().await;
        listeners.entry(type_id).or_insert_with(Vec::new);

        if let Some(type_listeners) = listeners.get_mut(&type_id) {
            type_listeners.push(erased_listener);
        }
    }

    /// 移除事件监听器
    pub async fn remove_application_listener<E: ApplicationEvent + 'static>(
        &self,
        index: usize,
    ) -> Result<(), IocError> {
        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.write().await;

        if let Some(type_listeners) = listeners.get_mut(&type_id) {
            if index < type_listeners.len() {
                type_listeners.remove(index);
                Ok(())
            } else {
                Err(IocError::EventHandlingError(format!(
                    "监听器索引 {} 超出范围",
                    index
                )))
            }
        } else {
            Err(IocError::EventHandlingError(format!(
                "找不到类型为 {:?} 的监听器",
                type_id
            )))
        }
    }

    /// 发布事件
    pub async fn publish_event<E: ApplicationEvent + 'static>(
        &self,
        event: &E,
    ) -> Result<(), IocError> {
        let type_id = TypeId::of::<E>();

        let listeners = self.listeners.read().await;
        if let Some(type_listeners) = listeners.get(&type_id) {
            for listener in type_listeners {
                listener.on_any_event(event).await?;
            }
        }

        Ok(())
    }
}

/// 基本的应用事件实现
pub struct SimpleApplicationEvent {
    name: &'static str,
    timestamp: u64,
    source: Option<Box<dyn Any + Send + Sync>>,
}

impl SimpleApplicationEvent {
    /// 创建新事件
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source: None,
        }
    }

    /// 设置事件源
    pub fn with_source<S: Any + Send + Sync>(mut self, source: S) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

impl ApplicationEvent for SimpleApplicationEvent {
    fn event_name(&self) -> &'static str {
        self.name
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn source(&self) -> Option<&dyn Any> {
        self.source.as_deref().map(|s| s as &dyn Any)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 应用就绪事件
pub struct ContextRefreshedEvent {
    base: SimpleApplicationEvent,
}

impl ContextRefreshedEvent {
    /// 创建新的应用就绪事件
    pub fn new() -> Self {
        Self {
            base: SimpleApplicationEvent::new("ContextRefreshedEvent"),
        }
    }
}

impl ApplicationEvent for ContextRefreshedEvent {
    fn event_name(&self) -> &'static str {
        self.base.event_name()
    }

    fn timestamp(&self) -> u64 {
        self.base.timestamp()
    }

    fn source(&self) -> Option<&dyn Any> {
        self.base.source()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 应用启动事件
pub struct ApplicationStartedEvent {
    base: SimpleApplicationEvent,
}

impl ApplicationStartedEvent {
    /// 创建新的应用启动事件
    pub fn new() -> Self {
        Self {
            base: SimpleApplicationEvent::new("ApplicationStartedEvent"),
        }
    }
}

impl ApplicationEvent for ApplicationStartedEvent {
    fn event_name(&self) -> &'static str {
        self.base.event_name()
    }

    fn timestamp(&self) -> u64 {
        self.base.timestamp()
    }

    fn source(&self) -> Option<&dyn Any> {
        self.base.source()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 应用准备关闭事件
pub struct ApplicationClosingEvent {
    base: SimpleApplicationEvent,
}

impl ApplicationClosingEvent {
    /// 创建新的应用准备关闭事件
    pub fn new() -> Self {
        Self {
            base: SimpleApplicationEvent::new("ApplicationClosingEvent"),
        }
    }
}

impl ApplicationEvent for ApplicationClosingEvent {
    fn event_name(&self) -> &'static str {
        self.base.event_name()
    }

    fn timestamp(&self) -> u64 {
        self.base.timestamp()
    }

    fn source(&self) -> Option<&dyn Any> {
        self.base.source()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
