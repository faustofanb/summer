extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use summer_core::{
    BeanDefinition, BeanDependency, BeanScope, ComponentDefinitionProvider, IocError,
};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, Path, Type};

// Helper function to check for a specific attribute path (e.g., "autowired")
fn has_attribute(attrs: &[Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attr_name))
}

#[proc_macro_derive(Component, attributes(component, autowired))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let mut bean_name = struct_name.to_string(); // Default bean name
    let mut bean_scope = quote! { summer_core::BeanScope::Singleton }; // Default scope

    // --- Parse struct-level attributes ---
    for attr in &input.attrs {
        if attr.path().is_ident("component") {
            // Parse #[component(name = "...", scope = "...")]
            if let Err(e) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    if let Ok(Lit::Str(lit)) = meta.value()?.parse::<Lit>() {
                        bean_name = lit.value(); // Set custom bean name
                        Ok(())
                    } else {
                        Err(meta.error("Expected string literal for component name"))
                    }
                } else if meta.path.is_ident("scope") {
                    if let Ok(Lit::Str(lit)) = meta.value()?.parse::<Lit>() {
                        let scope_str = lit.value();
                        match scope_str.to_lowercase().as_str() {
                            "singleton" => {
                                bean_scope = quote! { summer_core::BeanScope::Singleton }
                            }
                            _ => {
                                return Err(
                                    meta.error(format!("Unsupported bean scope: {}", scope_str))
                                )
                            }
                        }
                        Ok(())
                    } else {
                        Err(meta.error("Expected string literal for component scope"))
                    }
                } else {
                    Err(meta.error("Unsupported component attribute"))
                }
            }) {
                return TokenStream::from(e.to_compile_error());
            }
        }
    }

    // --- Parse field-level attributes (for dependencies) ---
    let mut dependencies = Vec::new();
    let mut field_assignments = Vec::new();
    let mut autowired_field_indices = Vec::new(); // Track indices of @Autowired fields

    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for (index, field) in fields.named.iter().enumerate() {
                // Check if the field has the #[autowired] attribute
                if has_attribute(&field.attrs, "autowired") {
                    let field_name = field.ident.as_ref().expect("Named field must have a name");
                    let field_type = &field.ty;

                    let type_id_expr = quote! { TypeId::of::<#field_type>() };

                    dependencies.push(quote! {
                        summer_core::BeanDependency {
                            type_id: #type_id_expr,
                            field_name: Some(stringify!(#field_name).to_string()),
                            required: true,
                        }
                    });

                    autowired_field_indices.push((index, field_name, field_type));
                }
            }

            // Generate field assignments ONLY for autowired fields
            for (dep_index, (_original_index, field_name, field_type)) in
                autowired_field_indices.iter().enumerate()
            {
                let dep_index_lit = syn::Index::from(dep_index);
                field_assignments.push(quote! {
                    #field_name: {
                        let dep_any = dependencies_vec[#dep_index_lit].clone();
                        dep_any.downcast::<#field_type>().expect("Dependency type mismatch in factory (Autowired field)")
                    }
                });
            }

            // Handle non-autowired fields - they need Default::default() or similar
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                if !autowired_field_indices
                    .iter()
                    .any(|(_, name, _)| name == field_name)
                {
                    field_assignments.push(quote! {
                        #field_name: Default::default()
                    });
                }
            }
        }
    }

    let dependencies_vec_expr = quote! { vec![#(#dependencies),*] };
    let num_dependencies = dependencies.len();

    // Generate factory function
    let factory_fn = quote! {
        fn factory(dependencies_vec: Vec<Arc<dyn Any + Send + Sync>>) -> Result<Arc<dyn Any + Send + Sync>, summer_core::IocError> {
            if dependencies_vec.len() != #num_dependencies {
                return Err(summer_core::IocError::DependencyError(format!(
                    "Internal error: Factory for {} expected {} dependencies (based on @Autowired), but received {}",
                    #bean_name,
                    #num_dependencies,
                    dependencies_vec.len()
                )));
            }

            let instance = #struct_name {
                #(#field_assignments),*
            };
            Ok(Arc::new(instance))
        }
    };

    // Generate ComponentDefinitionProvider implementation
    let expanded = quote! {
        use std::any::{Any, TypeId};
        use std::sync::Arc;
        use std::str::FromStr;
        use summer_core::{BeanDefinition, BeanScope, BeanDependency, ComponentDefinitionProvider, IocError};

        impl summer_core::ComponentDefinitionProvider for #struct_name {
            fn get_bean_definition() -> summer_core::BeanDefinition {
                #factory_fn

                let mut definition = summer_core::BeanDefinition::new(
                    #bean_name.to_string(),
                    TypeId::of::<#struct_name>(),
                    std::any::type_name::<#struct_name>(),
                    factory,
                    #dependencies_vec_expr,
                );

                definition.scope = #bean_scope;

                definition
            }
        }
    };

    TokenStream::from(expanded)
}
