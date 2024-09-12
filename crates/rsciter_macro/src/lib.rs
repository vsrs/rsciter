use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Literal as Literal2, TokenStream as TokenStream2, TokenTree as TokenTree2};
use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

pub(crate) mod items;
pub(crate) mod sciter_mod;

fn to_cstr_lit(data: &impl std::fmt::Display) -> TokenStream2 {
    let data = format!("c\"{data}\"");
    Literal2::from_str(&data).unwrap().into_token_stream()
}

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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn asset(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data = asset_impl(attr.into(), input.into());
    match data {
        Ok(res) => res.into(),
        Err(e) => {
            let span = e.span();
            let message = format!("{e}");
            proc_macro_error::abort!(span, message);
        }
    }
}

fn asset_impl(attr: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
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
    attr: TokenStream2,
    struct_item: syn::ItemStruct,
) -> Result<TokenStream2, syn::Error> {
    todo!()
}

fn asset_process_impl_block(
    attr: TokenStream2,
    impl_block: syn::ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    todo!()
}

fn asset_process_module(
    attr: TokenStream2,
    mut module: syn::ItemMod,
) -> Result<TokenStream2, syn::Error> {
    let (info, module) = sciter_mod::SciterMod::prepare(&mut module, attr)?;
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

fn generate_mod_asset(smod: &sciter_mod::SciterMod) -> TokenStream2 {
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
        // fo mod generate passport here
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
        Ok(module) => return xmod_process_module(attr, module),
        _ => (),
    }

    match syn::parse2::<syn::ItemImpl>(input) {
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE.to_string())),
        Ok(impl_block) => xmod_process_impl_block(attr, impl_block),
    }
}

fn xmod_process_module(
    attr: TokenStream2,
    mut module: syn::ItemMod,
) -> Result<TokenStream2, syn::Error> {
    let (info, module) = sciter_mod::SciterMod::prepare(&mut module, attr)?;
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

fn xmod_process_impl_block(
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
    let (names, calls, implementations, _) = info.methods(None);

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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::str::FromStr;

    fn expand(
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

        dbg!(&result);

        let file = syn::parse_file(&result).unwrap();
        prettyplease::unparse(&file)
    }

    #[test]
    fn test_xmod_mod() {
        let result = expand(
            "attrs",
            r#"
mod M {
    pub fn second(x: u64, x_ref: &u64) {
        let _ = x;
        let _ = x_ref;
    }
}    
"#,
            xmod_impl,
        );

        expect![
            r#"
#[allow(non_snake_case)]
#[allow(dead_code)]
mod M {
    pub fn second(x: u64, x_ref: &u64) {
        let _ = x;
        let _ = x_ref;
    }
}
struct attrs;
impl ::rsciter::XFunctionProvider for attrs {
    fn call(
        &mut self,
        name: &str,
        args: &[::rsciter::Value],
    ) -> ::rsciter::Result<Option<::rsciter::Value>> {
        match name {
            "second" => self.call_second(args),
            _ => Err(::rsciter::Error::ScriptingNoMethod(name.to_string())),
        }
    }
}
#[allow(non_snake_case)]
impl attrs {
    fn call_second(
        &mut self,
        args: &[::rsciter::Value],
    ) -> ::rsciter::Result<Option<::rsciter::Value>> {
        if args.len() != 2usize {
            return Err(::rsciter::Error::ScriptingInvalidArgCount("second".to_string()));
        }
        let x = <u64 as ::rsciter::conv::FromValue>::from_value(&args[0usize])
            .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                "x",
                Box::new(err),
            ))?;
        let x_ref = <u64 as ::rsciter::conv::FromValue>::from_value(&args[1usize])
            .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                "x_ref",
                Box::new(err),
            ))?;
        M::second(x, &x_ref);
        Ok(None)
    }
}
"#
        ]
        .assert_eq(&result);
    }

    #[test]
    fn test_xmod_impl_block() {
        let result = expand(
            "",
            r#"
impl S {
    pub fn second(&self, x: u64, x_ref: &u64) {
        let _ = x;
        let _ = x_ref;
    }
}"#,
            xmod_impl,
        );

        expect![
            r#"
#[allow(non_snake_case)]
#[allow(dead_code)]
impl S {
    pub fn second(&self, x: u64, x_ref: &u64) {
        let _ = x;
        let _ = x_ref;
    }
}
impl ::rsciter::XFunctionProvider for S {
    fn call(
        &mut self,
        name: &str,
        args: &[::rsciter::Value],
    ) -> ::rsciter::Result<Option<::rsciter::Value>> {
        match name {
            "second" => self.call_second(args),
            _ => Err(::rsciter::Error::ScriptingNoMethod(name.to_string())),
        }
    }
}
#[allow(non_snake_case)]
impl S {
    fn call_second(
        &mut self,
        args: &[::rsciter::Value],
    ) -> ::rsciter::Result<Option<::rsciter::Value>> {
        if args.len() != 2usize {
            return Err(::rsciter::Error::ScriptingInvalidArgCount("second".to_string()));
        }
        let x = <u64 as ::rsciter::conv::FromValue>::from_value(&args[0usize])
            .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                "x",
                Box::new(err),
            ))?;
        let x_ref = <u64 as ::rsciter::conv::FromValue>::from_value(&args[1usize])
            .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                "x_ref",
                Box::new(err),
            ))?;
        self.second(x, &x_ref);
        Ok(None)
    }
}
"#
        ]
        .assert_eq(&result);
    }

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
            asset_impl,
        );

        println!("{result}");
    }
}
