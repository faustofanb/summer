use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, ImplItem, ItemImpl, ItemStruct, PatType, Type};
use syn::spanned::Spanned;

pub fn anno_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input_struct.ident;
    let struct_name_str = struct_ident.to_string();

    // --- Generate Constructor Wrapper ---
    let constructor_wrapper = quote! {
        |container: ::std::sync::Arc<dyn ::summer_core::BeanProvider>| -> Result<::std::sync::Arc<dyn ::std::any::Any + Send + Sync>, ::summer_core::ConstructorError> {
            struct _RequiresDefault where #struct_ident: Default;

            let instance = #struct_ident::default();
            Ok(::std::sync::Arc::new(instance))
        }
    };

    // Generate the code to submit metadata to inventory
    let generated_inventory_submission = quote! {
        ::summer_core::inventory::submit! {
            ::summer_core::BeanDefinitionMetadata {
                bean_name: #struct_name_str,
                bean_type_id: || ::std::any::TypeId::of::<#struct_ident>(),
                constructor: #constructor_wrapper,
            }
        }
    };

    // Add `#[derive(Default)]` if it doesn't exist (simplistic approach for now)
    let mut final_struct = input_struct.clone();
    let mut has_derive_default = false;
    for attr in &input_struct.attrs {
        if attr.path().is_ident("derive") {
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Default") {
                    has_derive_default = true;
                    return Ok(());
                }
                Ok(())
            });
            if result.is_err() { /* Handle error */ }
            if has_derive_default {
                break;
            }
        }
    }
    if !has_derive_default {
        let default_derive_attr: Attribute = parse_quote!(#[derive(Default)]);
        final_struct.attrs.push(default_derive_attr);
    }

    // Combine the potentially modified struct definition with the generated code
    let output = quote! {
        #final_struct
        #generated_inventory_submission
    };

    output.into()
}


// --- Helper function (Example of how parsing might look - NOT USED YET) ---
fn _find_new_and_generate_wrapper(
    struct_item: &ItemStruct,
    impl_item: &ItemImpl,
) -> proc_macro2::TokenStream {
    let struct_ident = &struct_item.ident;
    let mut new_fn = None;

    // Check if the impl block is for our struct
    if let Type::Path(type_path) = &*impl_item.self_ty {
        if type_path.path.is_ident(struct_ident) {
            // Find the 'new' function
            for item in &impl_item.items {
                if let ImplItem::Fn(method) = item {
                    if method.sig.ident == "new" {
                        new_fn = Some(method);
                        break;
                    }
                }
            }
        }
    }

    if let Some(func) = new_fn {
        let mut dep_fetches = Vec::new();
        let mut dep_names = Vec::new();
        for input in func.sig.inputs.iter() {
            if let FnArg::Typed(PatType { pat, ty, .. }) = input {
                let dep_name = &**pat;
                let dep_type = &**ty;
                dep_fetches.push(quote_spanned! {ty.span()=>
                    let #dep_name = container.get_bean_by_typeid(::std::any::TypeId::of::<#dep_type>())?
                        .downcast::<#dep_type>()
                        .map_err(|_| Box::new(::summer_core::ConstructorError::from(format!("Type mismatch for dependency of type {}", stringify!(#dep_type)))) as Box<dyn std::error::Error + Send + Sync>)?;
                });
                dep_names.push(dep_name.to_token_stream());
            } else if let FnArg::Receiver(_) = input {
                // Skip `self` if present
            } else {
                return quote_spanned! {input.span()=> compile_error!("Unsupported argument type in constructor"); };
            }
        }

        quote! {
            |container: ::std::sync::Arc<dyn ::summer_core::BeanProvider>| -> Result<::std::sync::Arc<dyn ::std::any::Any + Send + Sync>, ::summer_core::ConstructorError> {
                #(#dep_fetches)*
                let instance = #struct_ident::new(#(#dep_names),*);
                Ok(::std::sync::Arc::new(instance))
            }
        }
    } else {
        quote! {
            |container: ::std::sync::Arc<dyn ::summer_core::BeanProvider>| -> Result<::std::sync::Arc<dyn ::std::any::Any + Send + Sync>, ::summer_core::ConstructorError> {
                struct _RequiresDefault where #struct_ident: Default;
                let instance = #struct_ident::default();
                Ok(::std::sync::Arc::new(instance))
            }
        }
    }
}