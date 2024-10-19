use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{parse, parse_macro_input, punctuated::Punctuated, token::Comma, Expr, Ident, LitStr, Token, TypeParam};

pub fn response_gen_impl(input: Punctuated<Expr, Comma>) -> TokenStream {
    let message_literal = input.first().unwrap();
    let field_idents = input.iter().skip(1).collect::<Vec<_>>();

    let struct_name = Ident::new("Response", Span::call_site());
    let generics = field_idents.iter().enumerate().map(|(i, _)| {
        TypeParam::from(Ident::new(&format!("T{}", i), Span::call_site()))
    }).collect::<Vec<_>>();

    let fields = field_idents.iter().zip(&generics).map(|(name, generic)| {
        quote! {
            pub #name: #generic
        }
    });

    let struct_definition = quote! {
        {
            #[derive(::serde::Serialize)]
            struct #struct_name<'a, #( #generics: ::serde::Serialize ),*>{
                pub message: &'a str,
                #( #fields ),*
            }
            Response {
                message: #message_literal,
                #( #field_idents ),*
            }
        }
    };
    struct_definition
}
