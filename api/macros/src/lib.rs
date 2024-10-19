use jres::jres_impl;
use proc_macro::{TokenStream};
use response::response_gen_impl;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, Expr};

mod response;
mod jres;

#[proc_macro]
pub fn response_gen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Comma>::parse_terminated);
    response_gen_impl(input).into()
}

#[proc_macro]
pub fn jres(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with Punctuated<Expr, Comma>::parse_terminated);
    jres_impl(input).into()
}
