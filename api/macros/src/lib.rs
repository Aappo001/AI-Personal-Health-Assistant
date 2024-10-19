use jres::jres_impl;
use proc_macro::{TokenStream};
use response::response_gen_impl;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, Expr};

mod response;
mod jres;

/// Generate an ad-hoc response struct that implements `serde::Serialize` with the first
/// argument as the message and the rest of the arguments as their names and values.
#[proc_macro]
pub fn response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Comma>::parse_terminated);
    response_gen_impl(input).into()
}

#[proc_macro]
pub fn jres(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Comma>::parse_terminated);
    jres_impl(input).into()
}
