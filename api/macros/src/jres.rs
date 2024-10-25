use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{parse, parse_macro_input, punctuated::Punctuated, token::Comma, Expr, Ident, LitStr, Token, TypeParam};

pub fn jres_impl(input: Punctuated<Expr, Comma>) -> TokenStream {
    let message_literal = input.first().unwrap();
    let field_idents = input.iter().skip(1).collect::<Vec<_>>();

    let fields = field_idents.iter().map(|(name)| {
        quote! {
            "#name": #name
        }
    });

    let struct_definition = quote! {
        {
            json!({
                "message": #message_literal,
                #( #field_idents ),*
            })
        }
    };
    struct_definition
}
