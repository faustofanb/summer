use crate::{BeanRegistry, IocError};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// Bean 后处理器
#[async_trait]
pub trait BeanPostProcessor: Send + Sync {
    /// 在初始化之前处理 Bean
    async fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError>;

    /// 在初始化之后处理 Bean
    async fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError>;
}

/// Bean 工厂后处理器
#[async_trait]
pub trait BeanFactoryPostProcessor: Send + Sync {
    /// 处理 Bean 工厂
    async fn post_process_bean_factory(&self, registry: &mut BeanRegistry) -> Result<(), IocError>;
}

/// 一个简单的日志记录 Bean 后处理器
pub struct LoggingBeanPostProcessor;

#[async_trait]
impl BeanPostProcessor for LoggingBeanPostProcessor {
    async fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError> {
        println!("Bean [{}] 开始初始化", bean_name);
        Ok(bean)
    }

    async fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, IocError> {
        println!("Bean [{}] 初始化完成", bean_name);
        Ok(bean)
    }
}

/// 自定义 Bean 工厂后处理器
pub struct CustomBeanFactoryPostProcessor {
    name: String,
}

impl CustomBeanFactoryPostProcessor {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl BeanFactoryPostProcessor for CustomBeanFactoryPostProcessor {
    async fn post_process_bean_factory(
        &self,
        _registry: &mut BeanRegistry,
    ) -> Result<(), IocError> {
        println!("Bean Factory 处理器 [{}] 开始处理 Bean 定义", self.name);

        // 这里可以修改 Bean 定义，例如替换属性占位符、解析条件注解等
        // 为简化起见，这个示例仅打印日志

        println!("Bean Factory 处理器 [{}] 完成处理", self.name);
        Ok(())
    }
}
