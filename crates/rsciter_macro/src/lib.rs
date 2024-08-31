use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::spanned::Spanned;

pub(crate) mod items;
pub(crate) mod sciter_mod;
pub(crate) mod passport;

use passport::passport_impl;

#[proc_macro_attribute]
pub fn transparent(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn passport(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data = passport_impl(attr.into(), input.into());
    match data {
        Ok(res) => res.into(),
        Err(e) => {
            let span = e.span();
            let message = format!("{e}");
            proc_macro_error::abort!(span, message);
        }        
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn xmod(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data = xmod_impl(attr.into(), input.into());
    match data {
        Ok(res) => res.into(),
        Err(e) => {
            let span = e.span();
            let message = format!("{e}");
            proc_macro_error::abort!(span, message);
        }        
    }
}

fn xmod_impl(attr: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
    const MESSAGE: &str =
        "the #[rsciter::xmod] attribute can only be applied to an inline module or impl block!";

    match syn::parse2::<syn::ItemMod>(input.clone()) {
        Ok(m) if m.content.is_none() => return Err(syn::Error::new(m.span(), MESSAGE)),
        Ok(module) => return process_module(attr, module),
        _ => (),
    }

    match syn::parse2::<syn::ItemImpl>(input) {
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE.to_string())),
        Ok(impl_block) => process_impl_block(attr, impl_block),
    }
}

fn process_module(
    attr: TokenStream2,
    mut module: syn::ItemMod,
) -> Result<TokenStream2, syn::Error> {
    let mut struct_name = attr.to_string();
    if struct_name.is_empty() {
        struct_name = module.ident.to_string();
        // struct and module names are in the same namespace,
        // have to rename the module to use its name
        module.ident = Ident::new(&format!("{}_mod", &module.ident), module.ident.span());
    }

    let info = sciter_mod::SciterMod::from_mod(&module, struct_name)?;
    let vis = info.visibility();
    let provider_struct_name = info.name_path();
    let code = generate_xfunction_provider(&info);
    Ok(quote!(
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #module // TODO: remove attrs like #[transparent]

        #vis struct #provider_struct_name;

        #code
    ))
}

fn process_impl_block(
    _attr: TokenStream2,
    block: syn::ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    let info = sciter_mod::SciterMod::from_impl_block(&block)?;
    let code = generate_xfunction_provider(&info);

    Ok(quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #block

        #code
    })
}

fn generate_xfunction_provider(info: &sciter_mod::SciterMod) -> TokenStream2 {
    let provider_struct_name = info.name_path();
    let (names, calls, implementations) = info.methods();

    quote! {
        impl ::rsciter::XFunctionProvider for #provider_struct_name {
            fn call(&mut self, name: &str, args: &[::rsciter::Value]) -> ::rsciter::Result<Option<::rsciter::Value>> {
                match name {
                    #( #names => #calls, )*
                    _ => Err(::rsciter::Error::ScriptingNoMethod(name.to_string())),
                }
            }
        }

        #[allow(non_snake_case)]
        impl #provider_struct_name {
            #( #implementations )*
        }
    }
}

#[test]
fn test() {
    use std::str::FromStr;

    let code = TokenStream2::from_str(
        r#"
    impl S {
        pub fn second(&self, x: u64, x_ref: &u64) {
            let _ = x;
            let _ = x_ref;
        }
    }    
    "#,
    )
    .unwrap();

    let s = xmod_impl(TokenStream2::new(), code).unwrap();
    let result = s.to_string();

    eprintln!("{result}");
}
