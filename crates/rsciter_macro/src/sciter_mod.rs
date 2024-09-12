use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{spanned::Spanned, Path, TypePath, Visibility};

use super::items::MethodInfo;
use crate::TokenStream2;

pub struct SciterMod<'m> {
    vis: Option<&'m syn::Visibility>,
    prefix: TokenStream2,
    name: TypePath,
    methods: Vec<MethodInfo<'m>>,
}

impl<'m> SciterMod<'m> {
    const ERR_SRC: &'static str = "xmod";

    pub fn prepare(
        module: &'m mut syn::ItemMod,
        attr: TokenStream2,
    ) -> syn::Result<(Self, &'m syn::ItemMod)> {
        let mut struct_name = attr.to_string();
        if struct_name.is_empty() {
            struct_name = module.ident.to_string();
            // struct and module names are in the same namespace,
            // have to rename the module to use its name
            module.ident = Ident::new(&format!("{}_mod", &module.ident), module.ident.span());
        }

        Self::from_mod(module, struct_name)
    }

    fn from_mod(module: &'m syn::ItemMod, name: String) -> syn::Result<(Self, &'m syn::ItemMod)> {
        let methods = Self::get_mod_methods(module)?;

        let mod_ident = &module.ident;
        let prefix = quote! { #mod_ident :: };

        let path = Path::from(Ident::new(&name, Span::call_site()));

        Ok((
            Self {
                vis: Some(&module.vis),
                prefix,
                name: TypePath { qself: None, path },
                methods,
            },
            module,
        ))
    }

    pub fn from_impl_block(impl_block: &'m syn::ItemImpl) -> syn::Result<Self> {
        let methods = Self::get_impl_methods(impl_block)?;
        Ok(Self {
            vis: None,
            prefix: quote! { self. },
            name: Self::get_impl_struct_name(impl_block)?,
            methods,
        })
    }

    pub fn name_path(&self) -> &TypePath {
        &self.name
    }

    pub fn visibility(&self) -> &syn::Visibility {
        self.vis.unwrap_or(&syn::Visibility::Inherited)
    }

    pub fn methods(&self) -> (Vec<String>, Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut names = Vec::new();
        let mut calls = Vec::new();
        let mut impls = Vec::new();

        for method in &self.methods {
            let call = method.call_ident();
            let body = method.body(&self.prefix);
            let call_impl = quote! {
                fn #call(&mut self, args: &[::rsciter::Value]) -> ::rsciter::Result<Option<::rsciter::Value>> {
                    #body
                }
            };

            names.push(method.name());
            calls.push(quote! { self.#call(args) });
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
                        let info = MethodInfo::new(&fn_item.sig, Self::ERR_SRC)?;
                        res.push(info);
                    }
                    _ => (),
                }
            }
        }

        Ok(res)
    }

    fn get_impl_methods(impl_block: &'m syn::ItemImpl) -> syn::Result<Vec<MethodInfo<'m>>> {
        let mut methods = Vec::new();
        for it in &impl_block.items {
            match it {
                syn::ImplItem::Fn(m) if matches!(m.vis, Visibility::Public(_)) => {
                    let info = MethodInfo::new(&m.sig, Self::ERR_SRC)?;
                    methods.push(info);
                }
                _ => (),
            }
        }

        Ok(methods)
    }

    fn get_impl_struct_name(impl_block: &syn::ItemImpl) -> syn::Result<TypePath> {
        if impl_block.generics.lt_token.is_some() {
            return Err(syn::Error::new(
                impl_block.generics.span(),
                "#[rsciter::xmod] Generic impl blocks are not supported!",
            ));
        }

        let ty = impl_block.self_ty.as_ref();

        match ty {
            syn::Type::Path(path) => Ok(path.clone()),
            _ => Err(syn::Error::new(
                ty.span(),
                "#[rsciter::xmod] Unsupported impl block type!",
            )),
        }
    }
}
