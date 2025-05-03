use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, Expr, ItemFn, Lit, Meta};

pub fn get_mapping_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    let handler_fn_name = item_fn.sig.ident.to_string();

    // Parse path from args: #[get_mapping("/path")]
    let meta = parse_macro_input!(args as Meta);
    let path_lit = match meta {
        Meta::Path(_) => return Error::new_spanned(
            meta,
            "get_mapping requires a path string literal, e.g. #[get_mapping = \"/path\"]"
        ).to_compile_error().into(),
        Meta::List(_) => return Error::new_spanned(
            meta,
            "get_mapping requires a path string literal, not a list, e.g. #[get_mapping = \"/path\"]"
        ).to_compile_error().into(),
        Meta::NameValue(meta_name_value) => {
            match &meta_name_value.value {
                Expr::Lit(expr_lit) => {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        lit_str.clone()
                    } else {
                        return Error::new_spanned(
                            &expr_lit.lit,
                            "get_mapping path must be a string literal"
                        ).to_compile_error().into()
                    }
                }
                _ => return Error::new_spanned(
                    &meta_name_value.value,
                    "get_mapping path must be a string literal"
                ).to_compile_error().into()
            }
        }
    };
    let path_str = path_lit.value();

    let controller_type_id_str = "PLACEHOLDER_CONTROLLER_ID";

    let registration = quote! {
        inventory::submit! {
            ::boot::mvc::route::RouteInfo {
                path: #path_str,
                method: ::boot::mvc::route::HttpMethodWrapper::Get,
                controller_type_id_str: #controller_type_id_str,
                handler_fn_name: #handler_fn_name,
            }
        }
    };

    quote! {
        #item_fn
        #registration
    }
    .into()
}

// TODO: Implement other mapping macros (Post, Put, Delete) similarly
