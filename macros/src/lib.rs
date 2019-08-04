extern crate proc_macro;

use proc_macro::TokenStream;
use std::mem;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn, LitStr};

use args::{Arg, Args};

mod args;

#[proc_macro_attribute]
pub fn command_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut handler_fn = parse_macro_input!(input as ItemFn);
    let macro_args = parse_macro_input!(args as Args);

    assert!(
        handler_fn.decl.variadic.is_none(),
        "handler functions are not allowed to be variadic",
    );

    assert!(
        handler_fn.unsafety.is_none(),
        "handler functions are not allowed to be `unsafe`",
    );

    let fn_ident = Ident::new(&format!("_{}_impl", handler_fn.ident), Span::call_site());
    let ident = mem::replace(&mut handler_fn.ident, fn_ident.clone());

    // TODO: Error handling (required for State parameters)
    let get_calls = (0..handler_fn.decl.inputs.len()).map(|_| quote!(param_matcher.get().unwrap()));

    let mut commands = Vec::new();
    for arg in macro_args.0 {
        match arg {
            Arg::Command(cmd_arg) => commands.push(cmd_arg),
            Arg::Commands(cmd_args) => commands.extend(cmd_args),
        }
    }

    // If no commands are given as arguments to this proc macro, use the function name as the
    // command name
    if commands.is_empty() {
        commands.push(LitStr::new(&ident.to_string(), Span::call_site()));
    }

    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        struct #ident;

        #handler_fn

        impl ruma_bot::CommandHandler for #ident {
            fn commands() -> &'static [&'static str] {
                &[#(#commands),*]
            }

            fn handle(
                &self,
                bot: &ruma_bot::Bot,
                msg_content: &str,
            ) -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + Send>> {
                use ruma_bot::GetParam;
                let param_matcher = ruma_bot::HandlerParamMatcher { bot, msg_content };
                let fut = #fn_ident(#(#get_calls),*);

                Box::pin(async move {
                    let res = fut.await;
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                })
            }
        }
    })
}
