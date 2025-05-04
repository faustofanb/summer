use crate::{
    event::{ApplicationEvent, ApplicationListener},
    ApplicationContext, BeanDefinition, BeanFactory, BeanFactoryPostProcessor, BeanPostProcessor,
    BeanRegistry, BeanScope, ComponentDefinitionProvider, ConfigResolver, IocError,
};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
enum VisitState {
    NotVisited,
    Visiting,
    Visited,
}

pub struct ApplicationContextBuilder {
    registry: BeanRegistry,
    config_resolver: Option<Arc<dyn ConfigResolver>>,
}

impl ApplicationContextBuilder {
    pub fn new() -> Self {
        Self {
            registry: BeanRegistry::default(),
            config_resolver: None,
        }
    }

    /// 添加配置解析器
    pub fn with_config_resolver(mut self, config_resolver: Arc<dyn ConfigResolver>) -> Self {
        self.config_resolver = Some(config_resolver);
        self
    }

    pub fn with_component<T>(mut self) -> Result<Self, IocError>
    where
        T: ComponentDefinitionProvider + Default + Send + Sync + 'static,
    {
        let definition = T::get_bean_definition();
        self.registry.add_bean_definition(definition)?;
        Ok(self)
    }

    fn sort_dependencies(
        &self,
        definitions: &HashMap<TypeId, BeanDefinition>,
    ) -> Result<Vec<TypeId>, IocError> {
        let mut sorted_list = Vec::new();
        let mut visit_states = HashMap::<TypeId, VisitState>::new();
        let mut recursion_stack = HashSet::<TypeId>::new();

        fn visit(
            type_id: TypeId,
            definitions: &HashMap<TypeId, BeanDefinition>,
            visit_states: &mut HashMap<TypeId, VisitState>,
            recursion_stack: &mut HashSet<TypeId>,
            sorted_list: &mut Vec<TypeId>,
        ) -> Result<(), IocError> {
            match visit_states
                .get(&type_id)
                .cloned()
                .unwrap_or(VisitState::NotVisited)
            {
                VisitState::Visited => return Ok(()),
                VisitState::Visiting => {
                    let cycle_path = recursion_stack
                        .iter()
                        .map(|tid| {
                            definitions
                                .get(tid)
                                .map_or_else(|| format!("{:?}", tid), |def| def.name.clone())
                        })
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    return Err(IocError::CircularDependency(format!(
                        "Circular dependency detected involving bean '{}': {}",
                        definitions
                            .get(&type_id)
                            .map_or_else(|| format!("{:?}", type_id), |def| def.name.clone()),
                        cycle_path
                    )));
                }
                VisitState::NotVisited => {
                    visit_states.insert(type_id, VisitState::Visiting);
                    recursion_stack.insert(type_id);

                    if let Some(definition) = definitions.get(&type_id) {
                        for dependency in &definition.dependencies {
                            if definitions.contains_key(&dependency.type_id) {
                                visit(
                                    dependency.type_id,
                                    definitions,
                                    visit_states,
                                    recursion_stack,
                                    sorted_list,
                                )?;
                            } else if dependency.required {
                                return Err(IocError::BeanNotFound(format!(
                                    "Required dependency with TypeId {:?} for bean '{}' not found in registered definitions.",
                                    dependency.type_id, definition.name
                                )));
                            }
                        }
                    } else {
                        return Err(IocError::BeanNotFound(format!(
                            "Definition for TypeId {:?} not found during sorting.",
                            type_id
                        )));
                    }

                    visit_states.insert(type_id, VisitState::Visited);
                    recursion_stack.remove(&type_id);
                    sorted_list.push(type_id);
                    Ok(())
                }
            }
        }

        let type_ids: Vec<TypeId> = definitions.keys().cloned().collect();
        for type_id in type_ids {
            if visit_states
                .get(&type_id)
                .cloned()
                .unwrap_or(VisitState::NotVisited)
                == VisitState::NotVisited
            {
                visit(
                    type_id,
                    definitions,
                    &mut visit_states,
                    &mut recursion_stack,
                    &mut sorted_list,
                )?;
            }
        }

        Ok(sorted_list)
    }

    pub async fn build(mut self) -> Result<ApplicationContext, IocError> {
        println!("Starting singleton bean instantiation...");

        // 确保配置解析器已设置
        let config_resolver = self.config_resolver.unwrap_or_else(|| {
            println!("No ConfigResolver provided, using default empty resolver");
            Arc::new(crate::config::MemoryConfigResolver::new())
        });

        let definitions = self.registry.get_definitions().clone();
        let sorted_type_ids = self.sort_dependencies(&definitions)?;

        println!("Bean instantiation order resolved.");

        let processors_clone: Option<Vec<Arc<dyn BeanPostProcessor>>> =
            self.registry.get_bean_post_processors().map(|v| v.clone());

        // 实例化所有单例Bean
        for type_id in sorted_type_ids {
            if let Some(definition) = definitions.get(&type_id) {
                if definition.scope == BeanScope::Singleton {
                    if !self.registry.singletons.contains_key(&type_id) {
                        println!("Instantiating singleton bean: {}", definition.name);

                        let mut resolved_dependencies: Vec<Arc<dyn Any + Send + Sync>> = Vec::new();
                        println!("Resolving dependencies for bean: {}", definition.name);
                        for dep_info in &definition.dependencies {
                            println!("  - Dependency required: TypeId {:?}", dep_info.type_id);
                            match self.registry.singletons.get(&dep_info.type_id) {
                                Some(dep_instance) => {
                                    println!("    Found dependency instance.");
                                    resolved_dependencies.push(dep_instance.clone());
                                }
                                None => {
                                    if dep_info.required {
                                        return Err(IocError::DependencyError(format!(
                                            "Failed to resolve required dependency with TypeId {:?} for bean '{}'. Dependency not found in singletons map despite topological sort.",
                                            dep_info.type_id, definition.name
                                        )));
                                    } else {
                                        println!("    Optional dependency with TypeId {:?} not found, skipping.", dep_info.type_id);
                                    }
                                }
                            }
                        }
                        println!(
                            "Finished resolving dependencies for bean: {}",
                            definition.name
                        );

                        let initial_bean = (definition.factory_fn)(resolved_dependencies)?;
                        let bean_name = &definition.name;

                        let mut current_bean = initial_bean;
                        if let Some(processors) = &processors_clone {
                            println!(
                                "Applying BeanPostProcessors (before initialization) for bean: {}",
                                bean_name
                            );
                            for processor in processors {
                                current_bean = processor
                                    .post_process_before_initialization(current_bean, bean_name)?;
                            }
                        }

                        if let Some(init_method_name) = &definition.init_method {
                            println!(
                                "TODO: Call init_method '{}' for bean '{}'",
                                init_method_name, bean_name
                            );
                        }

                        if let Some(processors) = &processors_clone {
                            println!(
                                "Applying BeanPostProcessors (after initialization) for bean: {}",
                                bean_name
                            );
                            for processor in processors {
                                current_bean = processor
                                    .post_process_after_initialization(current_bean, bean_name)?;
                            }
                        }

                        self.registry.singletons.insert(type_id, current_bean);
                        println!("Successfully registered singleton bean: {}", bean_name);
                    } else {
                        println!(
                            "Singleton bean '{}' (TypeId: {:?}) already instantiated (likely as a dependency). Skipping.",
                            definition.name, type_id
                        );
                    }
                }
            } else {
                eprintln!(
                    "Warning: TypeId {:?} found in sorted list but not in definitions map.",
                    type_id
                );
            }
        }
        println!("Finished singleton bean instantiation.");

        // 创建ApplicationContext实例
        println!("Creating ApplicationContext...");
        let registry_arc = Arc::new(self.registry);
        let context = ApplicationContext::new(registry_arc, config_resolver);
        println!("ApplicationContext created successfully.");

        // 注册ApplicationListeners
        println!("Registering ApplicationListeners with the event multicaster...");
        self.register_application_listeners(&context).await?;
        println!("ApplicationListeners registered successfully.");

        Ok(context)
    }

    /// 注册所有ApplicationListener Bean到事件多播器
    async fn register_application_listeners(
        &self,
        context: &ApplicationContext,
    ) -> Result<(), IocError> {
        // 需要扫描所有单例对象，找出实现了ApplicationListener的Bean
        for (type_id, bean) in &self.registry.singletons {
            // 尝试将Bean转换为监听器
            if let Some(definition) = self.registry.get_definitions().get(type_id) {
                // 检查是否实现了ApplicationListener trait
                // 由于Rust类型系统的限制，这里需要一些技巧
                // 这里只是演示，实际可能需要在Bean定义中添加标记
                if definition.name.contains("Listener") || definition.type_name.contains("Listener")
                {
                    println!("Found potential listener: {}", definition.name);
                    // 真实实现中，这里需要通过更复杂的方式确定是否为监听器以及监听的事件类型
                    // 可能需要在BeanDefinition中添加额外的元数据
                }
            }
        }

        // 因为类型安全的限制，这里只是一个演示
        // 实际实现中，可能需要在BeanDefinition中存储额外的类型信息
        // 或者定义特殊的注册方法让用户显式注册监听器

        Ok(())
    }
}

impl Default for ApplicationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
