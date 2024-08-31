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
    pub fn from_mod(module: &'m syn::ItemMod, name: String) -> syn::Result<Self> {
        let methods = Self::get_mod_methods(module)?;

        let mod_ident = &module.ident;
        let prefix = quote! { #mod_ident :: };

        let path = Path::from(Ident::new(&name, Span::call_site()));

        Ok(Self {
            vis: Some(&module.vis),
            prefix,
            name: TypePath { qself: None, path },
            methods,
        })
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

    pub fn passport_methods(&self) -> (Vec<String>, Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut names = Vec::new();
        let mut calls = Vec::new();
        let mut impls = Vec::new();

        for method in &self.methods {
            let call = method.call_ident();
            let body = method.passport_body(&self.prefix);
            let name = self.name_path();
            let call_impl = quote! {
                unsafe extern "C" fn #call(
                    thing: *mut som_asset_t,
                    argc: UINT,
                    argv: *const VALUE,
                    p_result: *mut VALUE
                ) -> SBOOL {
                    let me = ::rsciter::som::IAsset::<#name>::from_raw(&thing);
                    let args = ::rsciter::args_from_raw_parts(argv, argc);
                    #body
                }
            };

            names.push(method.name());
            let method_name = method.name();
            let args_count: usize = method.args_count() - 1;
            calls.push(quote! {
                method.name = sapi.atom(#method_name).unwrap();
                method.func = Some(#call);
                method.params = #args_count;
            });
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

    fn get_impl_methods(impl_block: &'m syn::ItemImpl) -> syn::Result<Vec<MethodInfo<'m>>> {
        let mut methods = Vec::new();
        for it in &impl_block.items {
            match it {
                syn::ImplItem::Fn(m) if matches!(m.vis, Visibility::Public(_)) => {
                    let info = MethodInfo::new(&m.sig)?;
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
