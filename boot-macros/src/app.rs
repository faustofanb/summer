use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

// 只保留实现逻辑，不带 #[proc_macro_attribute]
pub fn summer_app_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. Parse the input function
    let main_fn = parse_macro_input!(input as ItemFn);
    let main_sig = &main_fn.sig;
    let main_block = &main_fn.block; // Keep the original function block

    // 2. Validate the function signature (expect async fn main() -> Result<...>)
    let valid_sig = main_sig.asyncness.is_some() && main_sig.ident == "main";
    let returns_result = match &main_sig.output {
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = &**ty {
                type_path
                    .path
                    .segments
                    .last()
                    .map_or(false, |seg| seg.ident == "Result")
            } else {
                false
            }
        }
        _ => false,
    };
    if !valid_sig || !returns_result {
        let error_msg =
            "Function marked with #[summer_app] must be `async fn main() -> Result<...>`";
        return syn::Error::new_spanned(&main_sig.fn_token, error_msg)
            .to_compile_error()
            .into();
    }

    // 3. Generate code: Add #[tokio::main] and keep the original function
    let expanded = quote! {
        // Add #[tokio::main] to handle the async runtime
        #[tokio::main]
        // Keep the original function signature and body
        #main_sig #main_block
    };

    TokenStream::from(expanded)
}
