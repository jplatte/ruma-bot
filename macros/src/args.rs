use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    Expr, ExprArray, ExprLit, Ident, Lit, LitStr, Token,
};

pub enum Arg {
    Command(LitStr),
    Commands(HashSet<LitStr>),
    //DmOnly
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.call(Ident::parse)?;
        let arg_name = ident.to_string();
        match &arg_name[..] {
            "command" => {
                if input.peek(Token![=]) {
                    let _: Token![=] = input.parse()?;
                    Ok(Self::Command(input.parse()?))
                } else {
                    Err(input.error("expected `=`"))
                }
            }
            "commands" => {
                if input.peek(Token![=]) {
                    let _: Token![=] = input.parse()?;
                    let array: ExprArray = input.parse()?;

                    if !array.attrs.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &array.attrs[0],
                            "attributes are not allowed here",
                        ));
                    }

                    let commands = array
                        .elems
                        .into_pairs()
                        .map(Pair::into_value)
                        .map(|expr| {
                            if let Expr::Lit(ExprLit { attrs, lit }) = expr {
                                if !attrs.is_empty() {
                                    return Err(syn::Error::new_spanned(
                                        &attrs[0],
                                        "attributes are not allowed here",
                                    ));
                                }

                                if let Lit::Str(s) = lit {
                                    Ok(s)
                                } else {
                                    Err(syn::Error::new_spanned(lit, "expected string literal"))
                                }
                            } else {
                                Err(syn::Error::new_spanned(expr, "expected string literal"))
                            }
                        })
                        .collect::<Result<_, _>>()?;

                    Ok(Self::Commands(commands))
                } else {
                    Err(input.error("expected `=`"))
                }
            }
            _ => Err(syn::Error::new_spanned(ident, "unknown ruma_bot option")),
        }
    }
}

pub struct Args(pub Vec<Arg>);

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Arg, Token![,]>::parse_terminated(input)?;
        Ok(Self(args.into_iter().collect()))
    }
}
