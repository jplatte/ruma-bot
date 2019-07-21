extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

use args::Args;

mod args;

#[proc_macro_attribute]
pub fn command_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let handler_fn = parse_macro_input!(input as ItemFn);
    let macro_args = parse_macro_input!(args as Args);

    assert!(
        handler_fn.unsafety.is_none(),
        "ruma_bot handler functions are not allowed to be `unsafe`"
    );

    let ident = handler_fn.ident;
    // TODO
    let commands = vec![ident.to_string()];

    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        struct #ident;

        impl ruma_bot::CommandHandler for #ident {
            fn commands(&self) -> &'static [&'static str] {
                &[#(#commands),*]
            }
        }
    })
}
