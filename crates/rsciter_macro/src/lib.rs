use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Literal as Literal2, TokenStream as TokenStream2, TokenTree as TokenTree2};
use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::spanned::Spanned;

pub(crate) mod asset_impl;
pub(crate) mod items;
pub(crate) mod sciter_mod;
pub(crate) mod xmod_impl;

#[proc_macro_error]
#[proc_macro]
pub fn cstr(ts: TokenStream) -> TokenStream {
    let ts = TokenStream2::from(ts);
    let span = ts.span();
    let mut iter = ts.into_iter();
    let Some(TokenTree2::Ident(ident)) = iter.next() else {
        proc_macro_error::abort!(span, "Expected ident");
    };

    to_cstr_lit(&ident).into()
}

pub(crate) fn to_cstr_lit(data: &impl std::fmt::Display) -> TokenStream2 {
    let data = format!("c\"{data}\"");
    Literal2::from_str(&data).unwrap().into_token_stream()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn asset(attr: TokenStream, input: TokenStream) -> TokenStream {
    with_impl(attr, input, asset_impl::asset)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn asset_ns(attr: TokenStream, input: TokenStream) -> TokenStream {
    with_impl(attr, input, asset_impl::asset_ns)
}

#[proc_macro_error]
#[proc_macro_derive(Passport)]
pub fn passport(input: TokenStream) -> TokenStream {
    with_impl(TokenStream::new(), input, asset_impl::passport)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn xmod(attr: TokenStream, input: TokenStream) -> TokenStream {
    with_impl(attr, input, xmod_impl::xmod)
}

fn with_impl(
    attr: TokenStream,
    input: TokenStream,
    impl_fn: impl FnOnce(TokenStream2, TokenStream2) -> syn::Result<TokenStream2>,
) -> TokenStream {
    let data = impl_fn(attr.into(), input.into());
    match data {
        Ok(res) => res.into(),
        Err(e) => {
            let span = e.span();
            let message = format!("{e}");
            proc_macro_error::abort!(span, message);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::str::FromStr;

    pub fn expand(
        attrs: &str,
        code: &str,
        f: impl FnOnce(TokenStream2, TokenStream2) -> syn::Result<TokenStream2>,
    ) -> String {
        let attrs = if attrs.is_empty() {
            TokenStream2::new()
        } else {
            TokenStream2::from_str(attrs).unwrap()
        };
        let code = TokenStream2::from_str(code).unwrap();
        let result = f(attrs, code).unwrap().to_string();
        let file = syn::parse_file(&result).unwrap();
        prettyplease::unparse(&file)
    }
}
