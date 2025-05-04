extern crate proc_macro;
mod ioc;

use proc_macro::TokenStream;
use crate::ioc::anno_component;

/// Macro to mark a struct as a component managed by the IOC container.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    anno_component(_attr, item)
}


