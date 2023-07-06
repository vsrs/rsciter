use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{proc_macro_error, ResultExt};
use quote::quote;
use syn::spanned::Spanned;

pub(crate) mod items;
pub(crate) mod sciter_mod;

#[proc_macro_attribute]
pub fn transparent(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn xmod(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data = xmod_impl(attr.into(), input.into()).unwrap_or_abort();
    data.into()
}

fn xmod_impl(attr: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
    match syn::parse2::<syn::ItemMod>(input) {
        Ok(m) if m.content.is_none() => Err(syn::Error::new(
            m.span(),
            "the #[rsciter::xmod] attribute can only be applied to an inline module!",
        )),
        Err(e) => Err(syn::Error::new(
            e.span(),
            format!("{e}: the #[rsciter::xmod] attribute can only be applied to an inline module!"),
        )),
        Ok(mut module) => {
            let mut struct_name = attr.to_string();
            if struct_name.is_empty() {
                struct_name = module.ident.to_string();
                // struct and module names are in the same namespace,
                // have to rename the module to use its name
                module.ident = Ident::new(&format!("{}_mod", &module.ident), module.ident.span());
            }

            let info = sciter_mod::SciterMod::new(&module, struct_name)?;
            let provider_struct_name = info.ident();
            let vis = info.visibility();
            let (names, calls, implementations) = info.methods();

            Ok(quote!(
                #[allow(non_snake_case)]
                #[allow(dead_code)]
                #module // TODO: remove attrs like #[tranparent]

                #vis struct #provider_struct_name;

                impl ::rsciter::XFunctionProvider for #provider_struct_name {
                    fn call(&mut self, name: &str, args: &[::rsciter::Value]) -> ::rsciter::Result<Option<::rsciter::Value>> {
                        match name {
                            #( #names => #calls, )*
                            _ => Err(::rsciter::Error::ScriptingNoMethod(name.to_string())),
                        }
                    }
                }

                impl #provider_struct_name {
                    #( #implementations )*
                }
            ))
        }
    }
}
