extern crate proc_macro;

mod app;
mod component;
mod mapping;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn summer_app(args: TokenStream, input: TokenStream) -> TokenStream {
    app::summer_app_impl(args, input)
}

#[proc_macro_attribute]
pub fn rest_controller(args: TokenStream, input: TokenStream) -> TokenStream {
    component::rest_controller_impl(args, input)
}

#[proc_macro_attribute]
pub fn get_mapping(args: TokenStream, input: TokenStream) -> TokenStream {
    mapping::get_mapping_impl(args, input)
}
