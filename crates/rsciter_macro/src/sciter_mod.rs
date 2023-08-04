use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Visibility};

use super::items::MethodInfo;
use crate::TokenStream2;

pub struct SciterMod<'m> {
    module: &'m syn::ItemMod,
    name: Ident,
    methods: Vec<MethodInfo<'m>>,
}

impl<'m> SciterMod<'m> {
    pub fn from_mod(module: &'m syn::ItemMod, name: String) -> syn::Result<Self> {
        let methods = Self::get_mod_methods(module)?;
        Ok(Self {
            module,
            name: Ident::new(&name, Span::call_site()),
            methods,
        })
    }

    pub fn ident(&self) -> &Ident {
        &self.name
    }

    pub fn visibility(&self) -> &syn::Visibility {
        &self.module.vis
    }

    pub fn methods(&self) -> (Vec<String>, Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut names = Vec::new();
        let mut calls = Vec::new();
        let mut impls = Vec::new();

        let mod_ident = &self.module.ident;
        let prefix = quote! { #mod_ident :: };

        for method in &self.methods {
            let call = method.call_ident();
            let body = method.body(&prefix);
            let call_impl = quote! {
                fn #call(args: &[::rsciter::Value]) -> ::rsciter::Result<Option<::rsciter::Value>> {
                    #body
                }
            };

            names.push(method.name());
            calls.push(quote! { Self::#call(args) });
            impls.push(call_impl);
        }

        (names, calls, impls)
    }

    fn get_mod_methods(module: &'m syn::ItemMod) -> syn::Result<Vec<MethodInfo<'m>>> {
        let mut res = Vec::<MethodInfo>::new();
        if let Some((_, items)) = module.content.as_ref() {
            for item in items {
                match item {
                    syn::Item::Fn(fn_item) if matches!(fn_item.vis, Visibility::Public(_)) => {
                        let info = MethodInfo::new(&fn_item.sig)?;
                        res.push(info);
                    }
                    _ => (),
                }
            }
        }

        Ok(res)
    }
}
