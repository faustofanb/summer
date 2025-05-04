use crate::IocError;
use async_trait::async_trait;
use std::sync::Arc;

/// 条件接口，用于条件化 Bean 注册
#[async_trait]
pub trait Condition: Send + Sync {
    /// 评估条件是否满足
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError>;
}

/// 条件评估上下文
pub struct ConditionContext {
    // 环境属性
    pub properties: std::collections::HashMap<String, String>,
    // 类路径检查器，用于检测类是否存在
    pub class_checker: Arc<dyn ClassChecker>,
    // 资源加载器，用于加载配置文件等资源
    pub resource_loader: Arc<dyn ResourceLoader>,
}

/// 类检查器，用于检查类是否存在
#[async_trait]
pub trait ClassChecker: Send + Sync {
    /// 检查类型是否存在
    async fn is_class_present(&self, class_name: &str) -> bool;
}

/// 资源加载器，用于加载配置文件等资源
#[async_trait]
pub trait ResourceLoader: Send + Sync {
    /// 加载资源
    async fn load_resource(&self, path: &str) -> Result<Vec<u8>, IocError>;
}

/// 条件评估器，用于评估多个条件
pub struct ConditionEvaluator {
    context: ConditionContext,
}

impl ConditionEvaluator {
    /// 创建条件评估器
    pub fn new(context: ConditionContext) -> Self {
        Self { context }
    }

    /// 评估所有条件是否满足
    pub async fn evaluate(&self, conditions: &[Arc<dyn Condition>]) -> Result<bool, IocError> {
        for condition in conditions {
            if !condition.matches(&self.context).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// 基于属性的条件
pub struct PropertyCondition {
    name: String,
    having_value: Option<String>,
}

impl PropertyCondition {
    pub fn new(name: impl Into<String>, having_value: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            having_value: having_value.map(|v| v.into()),
        }
    }
}

#[async_trait]
impl Condition for PropertyCondition {
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError> {
        if let Some(value) = context.properties.get(&self.name) {
            if let Some(having_value) = &self.having_value {
                Ok(value == having_value)
            } else {
                // 如果只检查属性是否存在
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

/// 基于类存在的条件
pub struct ClassCondition {
    class_name: String,
}

impl ClassCondition {
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
        }
    }
}

#[async_trait]
impl Condition for ClassCondition {
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError> {
        Ok(context
            .class_checker
            .is_class_present(&self.class_name)
            .await)
    }
}

/// 组合条件 (AND)
pub struct AndCondition {
    conditions: Vec<Arc<dyn Condition>>,
}

impl AndCondition {
    pub fn new(conditions: Vec<Arc<dyn Condition>>) -> Self {
        Self { conditions }
    }
}

#[async_trait]
impl Condition for AndCondition {
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError> {
        for condition in &self.conditions {
            if !condition.matches(context).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// 组合条件 (OR)
pub struct OrCondition {
    conditions: Vec<Arc<dyn Condition>>,
}

impl OrCondition {
    pub fn new(conditions: Vec<Arc<dyn Condition>>) -> Self {
        Self { conditions }
    }
}

#[async_trait]
impl Condition for OrCondition {
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError> {
        for condition in &self.conditions {
            if condition.matches(context).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// 组合条件 (NOT)
pub struct NotCondition {
    condition: Arc<dyn Condition>,
}

impl NotCondition {
    pub fn new(condition: Arc<dyn Condition>) -> Self {
        Self { condition }
    }
}

#[async_trait]
impl Condition for NotCondition {
    async fn matches(&self, context: &ConditionContext) -> Result<bool, IocError> {
        Ok(!self.condition.matches(context).await?)
    }
}
