extern crate proc_macro;

use proc_macro::TokenStream;
use std::mem;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

use args::Args;

mod args;

#[proc_macro_attribute]
pub fn command_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut handler_fn = parse_macro_input!(input as ItemFn);
    let macro_args = parse_macro_input!(args as Args);

    assert!(
        handler_fn.decl.variadic.is_none(),
        "ruma_bot handler functions are not allowed to be variadic",
    );

    assert!(
        handler_fn.unsafety.is_none(),
        "ruma_bot handler functions are not allowed to be `unsafe`",
    );

    let infer_type = syn::Type::Infer(syn::TypeInfer {
        underscore_token: Default::default(),
    });
    let fn_type = syn::TypeBareFn {
        lifetimes: None,
        unsafety: None,
        abi: handler_fn.abi.clone(),
        fn_token: Default::default(),
        paren_token: Default::default(),
        inputs: (0..handler_fn.decl.inputs.len())
            .map(|_| syn::BareFnArg {
                name: None,
                ty: infer_type.clone(),
            })
            .collect(),
        variadic: None,
        output: syn::ReturnType::Type(Default::default(), Box::new(infer_type)),
    };

    let ident = mem::replace(
        &mut handler_fn.ident,
        Ident::new("command_handler_impl", Span::call_site()),
    );
    // TODO
    let commands = vec![ident.to_string()];

    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        struct #ident;

        impl ruma_bot::CommandHandler for #ident {
            fn commands() -> &'static [&'static str] {
                &[#(#commands),*]
            }

            fn call(
                &mut self,
                bot: &ruma_bot::Bot,
            ) -> Box<dyn futures::Future<Output = Result<(), failure::Error>>> {
                #handler_fn

                ruma_bot::CommandHandlerFn::call(command_handler_impl as #fn_type, bot)
            }
        }
    })
}
