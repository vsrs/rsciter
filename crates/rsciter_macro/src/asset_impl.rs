use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::{sciter_mod::SciterMod, to_cstr_lit};

pub fn asset(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    const MESSAGE: &str =
        "the #[rsciter::asset] attribute can only be applied to a struct, inline module or impl block!";

    match syn::parse2::<syn::ItemMod>(input.clone()) {
        Ok(m) if m.content.is_none() => return Err(syn::Error::new(m.span(), MESSAGE)),
        Ok(module) => return asset_process_module(attr, module),
        _ => (),
    }

    match syn::parse2::<syn::ItemImpl>(input.clone()) {
        Ok(impl_block) => return asset_process_impl_block(attr, impl_block),
        _ => (),
    }

    match syn::parse2::<syn::ItemStruct>(input) {
        Ok(s) => asset_process_struct(attr, s),
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE.to_string())),
    }
}

fn asset_process_struct(
    attr: TokenStream,
    struct_item: syn::ItemStruct,
) -> Result<TokenStream, syn::Error> {
    todo!()
}

fn asset_process_impl_block(
    attr: TokenStream,
    impl_block: syn::ItemImpl,
) -> Result<TokenStream, syn::Error> {
    todo!()
}

fn asset_process_module(
    attr: TokenStream,
    mut module: syn::ItemMod,
) -> Result<TokenStream, syn::Error> {
    let (info, module) = SciterMod::prepare(&mut module, attr)?;
    let vis = info.visibility();
    let provider_struct_name = info.name_path();

    let code = generate_mod_asset(&info);
    Ok(quote!(
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #module // TODO: remove attrs like #[transparent]

        #vis struct #provider_struct_name;

        #code
    ))
}

fn generate_mod_asset(smod: &SciterMod) -> TokenStream {
    let provider_struct_name = smod.name_path();
    let (names, calls, implementations, arg_counts) = smod.methods(Some("asset_mut"));

    let mut method_defs = Vec::new();
    for ((name, call), arg_count) in names.iter().zip(calls).zip(arg_counts) {
        let thunk_name = quote::format_ident!("{name}_thunk");
        let cstr_name = to_cstr_lit(&name);
        method_defs.push(quote! {
            {
                unsafe extern "C" fn #thunk_name(
                    thing: *mut bindings::som_asset_t,
                    argc: bindings::UINT,
                    argv: *const bindings::SCITER_VALUE,
                    p_result: *mut bindings::SCITER_VALUE,
                ) -> bindings::SBOOL {
                    let args = ::rsciter::args_from_raw_parts(argv, argc);
                    let mut asset_mut = som::AssetRefMut::<#provider_struct_name>::new(thing);
                    match #call {
                        Ok(Some(res)) => {
                            *p_result = res.take();
                            1
                        },
                        Ok(_) => {
                            // successful call, no return value
                            1
                        },
                        Err(_err) => {
                            // TODO: Value::error_string
                            0
                        }
                    }
                }

                ::rsciter::som::Atom::new(#cstr_name).map(|name| ::rsciter::som::MethodDef {
                    reserved: std::ptr::null_mut(),
                    name: name.into(),
                    params: #arg_count,
                    func: Some(#thunk_name),
                })
            },
        });
    }

    let count = method_defs.len();

    quote! {
        // for mod generate passport here
        impl ::rsciter::som::HasPassport for #provider_struct_name {
            fn passport(&self) -> ::rsciter::Result<&'static ::rsciter::som::Passport> {
                let passport = ::rsciter::som::impl_passport!(self, #provider_struct_name);
                passport
            }
        }

        impl ::rsciter::som::Methods for #provider_struct_name {
            fn methods() -> &'static [Result<som::MethodDef>] {
                static METHODS: std::sync::OnceLock<[Result<som::MethodDef>; #count]> = std::sync::OnceLock::new();
                METHODS.get_or_init(|| {
                    [
                        #( #method_defs )*
                    ]
                })
            }
        }

        #[allow(non_snake_case)]
        impl #provider_struct_name {
            #( #implementations )*
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::expand;

    use super::*;

    #[test]
    fn test_asset_mod() {
        let result = expand(
            "",
            r#"
mod Namespace {
    pub fn open(path: &str, flags:usize) {
        todo!()
    }
}
"#,
            asset,
        );

        println!("{result}");
    }
}
