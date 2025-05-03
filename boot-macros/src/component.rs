use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

// TODO: Implement #[component], #[service]

pub fn rest_controller_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. Parse the input struct (Controller definition)
    let item_struct = parse_macro_input!(input as ItemStruct);
    let _controller_ident = &item_struct.ident; // Use if needed later

    // TODO: Register controller type using inventory if needed for DI
    // For now, this macro mainly acts as a marker.

    // Return the original struct definition
    quote! {
        #item_struct
    }
    .into()
}
