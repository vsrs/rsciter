use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::{sciter_mod::SciterMod, to_cstr_lit};

pub fn asset_ns(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    const MESSAGE: &str =
        "the #[rsciter::asset_ns] attribute can only be applied to an inline module!";

    match syn::parse2::<syn::ItemMod>(input.clone()) {
        Ok(m) if m.content.is_none() => return Err(syn::Error::new(m.span(), MESSAGE)),
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE)),
        Ok(module) => {
            let (mod_code, name) = asset_process_module(attr, module, true)?;

            Ok(quote! {
                #mod_code

                impl #name {
                    pub fn new() -> ::rsciter::Result<::rsciter::som::GlobalAsset<#name>> {
                        ::rsciter::som::GlobalAsset::new(#name)
                    }
                }
            })
        }
    }
}

pub fn asset(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    const MESSAGE: &str =
        "the #[rsciter::asset] attribute can only be applied to a struct, inline module or impl block!";

    match syn::parse2::<syn::ItemMod>(input.clone()) {
        Ok(m) if m.content.is_none() => return Err(syn::Error::new(m.span(), MESSAGE)),
        Ok(module) => return asset_process_module(attr, module, false).map(|r| r.0),
        _ => (),
    }

    match syn::parse2::<syn::ItemImpl>(input.clone()) {
        Ok(impl_block) => return asset_process_impl_block(attr, &impl_block, false),
        _ => (),
    }

    match syn::parse2::<syn::ItemStruct>(input) {
        Ok(s) => asset_process_struct(attr, s),
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE)),
    }
}

fn generate_fields(strukt: &syn::ItemStruct, struct_name: impl ToTokens) -> TokenStream {
    let fields: Vec<TokenStream> = strukt
        .fields
        .iter()
        .filter_map(|field| {
            field
                .ident
                .as_ref()
                .map(|field_name| quote! {impl_prop!( #struct_name :: #field_name) })
        })
        .collect();
    let code = if fields.is_empty() {
        TokenStream::new()
    } else {
        let count = fields.len();
        quote! {
            impl ::rsciter::som::Fields for #struct_name {
                fn fields() -> &'static [::rsciter::Result<::rsciter::som::PropertyDef>] {
                    static FIELDS: std::sync::OnceLock<[::rsciter::Result<::rsciter::som::PropertyDef>; #count]> =
                        std::sync::OnceLock::new();

                    use ::rsciter::impl_prop;

                    FIELDS.get_or_init(|| [ #( #fields, )*  ])
                }
            }

        }
    };

    code
}

fn asset_process_struct(
    attr: TokenStream,
    strukt: syn::ItemStruct,
) -> Result<TokenStream, syn::Error> {
    let _ = attr;

    let struct_name = strukt.ident.clone();
    let passport = generate_passport(&struct_name);
    let code = generate_fields(&strukt, &struct_name);

    Ok(quote! {
        #strukt

        #passport

        #code
    })
}

fn asset_process_impl_block(
    attr: TokenStream,
    block: &syn::ItemImpl,
    skip_block: bool,
) -> Result<TokenStream, syn::Error> {
    let info = SciterMod::from_impl_block(block)?;
    let methods = generate_mod_methods(&info);
    let passport = if attr.to_string() == "HasPassport" {
        generate_passport(info.name_path())
    } else {
        TokenStream::new()
    };

    let block = if skip_block {
        TokenStream::new()
    } else {
        quote! {
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            #block
        }
    };

    Ok(quote! {
        #block

        #passport

        #methods
    })
}

fn asset_process_module(
    attr: TokenStream,
    mut module: syn::ItemMod,
    full: bool,
) -> Result<(TokenStream, syn::TypePath), syn::Error> {
    let (info, module) = SciterMod::prepare(&mut module, attr)?;
    let vis = info.visibility();
    let provider_struct_name = info.name_path();

    let methods = generate_mod_methods(&info);
    let passport = generate_passport(provider_struct_name);
    let (module, addons) = if full {
        (TokenStream::new(), generate_ns_items(&module)?)
    } else {
        (module.to_token_stream(), TokenStream::new())
    };
    Ok((
        quote!(
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            #module // TODO: remove attrs like #[transparent]

            #vis struct #provider_struct_name;

            #passport

            #methods

            #addons
        ),
        provider_struct_name.clone(),
    ))
}

fn generate_ns_items(module: &syn::ItemMod) -> syn::Result<TokenStream> {
    let mut result = TokenStream::new();
    if let Some(items) = module.content.as_ref().map(|content| &content.1) {
        for item in items.iter() {
            match item {
                syn::Item::Struct(s) => {
                    let passport = generate_passport(&s.ident);
                    let code = generate_fields(s, &s.ident);

                    result.extend(passport);
                    result.extend(code);
                }

                syn::Item::Impl(impl_block) if impl_block.trait_.is_none() => {
                    let methods = asset_process_impl_block(TokenStream::new(), impl_block, true);
                    result.extend(methods);
                }

                _ => (),
            }
        }
    }

    // hackish and inefficient. The better way is to reimplement ToTokens, skip last token and append result
    let mut code = quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #module
    }
    .to_token_stream()
    .to_string();

    code.pop(); // last '}'
    code.push_str(&result.to_string());
    code.push('}');

    let s = TokenStream::from_str(&code)?;

    Ok(s)
}

fn generate_passport(name: impl ToTokens) -> TokenStream {
    quote! {
        impl ::rsciter::som::HasPassport for #name {
            fn passport(&self) -> ::rsciter::Result<&'static ::rsciter::som::Passport> {
                use ::rsciter::som::*;
                let passport = impl_passport!(self, #name);
                passport
            }
        }
    }
}

fn generate_mod_methods(smod: &SciterMod) -> TokenStream {
    let provider_struct_name = smod.name_path();
    let (names, calls, implementations, arg_counts) = smod.methods(Some("asset_mut"));

    let mut method_defs = Vec::new();
    for ((name, call), arg_count) in names.iter().zip(calls).zip(arg_counts) {
        let thunk_name = quote::format_ident!("{name}_thunk");
        let cstr_name = to_cstr_lit(&name);
        method_defs.push(quote! {
            {
                unsafe extern "C" fn #thunk_name(
                    thing: *mut ::rsciter::bindings::som_asset_t,
                    argc: ::rsciter::bindings::UINT,
                    argv: *const ::rsciter::bindings::SCITER_VALUE,
                    p_result: *mut ::rsciter::bindings::SCITER_VALUE,
                ) -> ::rsciter::bindings::SBOOL {
                    let args = ::rsciter::args_from_raw_parts(argv, argc);
                    let mut asset_mut = ::rsciter::som::AssetRefMut::<#provider_struct_name>::new(thing);
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
        impl ::rsciter::som::Methods for #provider_struct_name {
            fn methods() -> &'static [::rsciter::Result<::rsciter::som::MethodDef>] {
                static METHODS: std::sync::OnceLock<[::rsciter::Result<::rsciter::som::MethodDef>; #count]> = std::sync::OnceLock::new();
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
    use expect_test::expect;

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

        expect![[r##"
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            mod Namespace_mod {
                pub fn open(path: &str, flags: usize) {
                    todo!()
                }
            }
            struct Namespace;
            impl ::rsciter::som::HasPassport for Namespace {
                fn passport(&self) -> ::rsciter::Result<&'static ::rsciter::som::Passport> {
                    use ::rsciter::som::*;
                    let passport = impl_passport!(self, Namespace);
                    passport
                }
            }
            impl ::rsciter::som::Methods for Namespace {
                fn methods() -> &'static [::rsciter::Result<::rsciter::som::MethodDef>] {
                    static METHODS: std::sync::OnceLock<
                        [::rsciter::Result<::rsciter::som::MethodDef>; 1usize],
                    > = std::sync::OnceLock::new();
                    METHODS
                        .get_or_init(|| {
                            [
                                {
                                    unsafe extern "C" fn open_thunk(
                                        thing: *mut ::rsciter::bindings::som_asset_t,
                                        argc: ::rsciter::bindings::UINT,
                                        argv: *const ::rsciter::bindings::SCITER_VALUE,
                                        p_result: *mut ::rsciter::bindings::SCITER_VALUE,
                                    ) -> ::rsciter::bindings::SBOOL {
                                        let args = ::rsciter::args_from_raw_parts(argv, argc);
                                        let mut asset_mut = ::rsciter::som::AssetRefMut::<
                                            Namespace,
                                        >::new(thing);
                                        match asset_mut.call_open(args) {
                                            Ok(Some(res)) => {
                                                *p_result = res.take();
                                                1
                                            }
                                            Ok(_) => 1,
                                            Err(_err) => 0,
                                        }
                                    }
                                    ::rsciter::som::Atom::new(c"open")
                                        .map(|name| ::rsciter::som::MethodDef {
                                            reserved: std::ptr::null_mut(),
                                            name: name.into(),
                                            params: 2usize,
                                            func: Some(open_thunk),
                                        })
                                },
                            ]
                        })
                }
            }
            #[allow(non_snake_case)]
            impl Namespace {
                fn call_open(
                    &mut self,
                    args: &[::rsciter::Value],
                ) -> ::rsciter::Result<Option<::rsciter::Value>> {
                    if args.len() != 2usize {
                        return Err(::rsciter::Error::ScriptingInvalidArgCount("open".to_string()));
                    }
                    let path = <String as ::rsciter::conv::FromValue>::from_value(&args[0usize])
                        .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                            "path",
                            Box::new(err),
                        ))?;
                    let flags = <usize as ::rsciter::conv::FromValue>::from_value(&args[1usize])
                        .map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(
                            "flags",
                            Box::new(err),
                        ))?;
                    Namespace_mod::open(&path, flags);
                    Ok(None)
                }
            }
        "##]]
        .assert_eq(&result);
    }

    #[test]
    fn test_asset_ns() {
        let result = expand(
            "",
            r#"
mod M {
    use super::*;

    pub struct NsObject {
        pub msg: String,
    }

    impl NsObject {
        pub fn test(&self) -> String {
            format!("Test: {}", self.msg)
        }
    }
}
"#,
            asset_ns,
        );

        expect![[r##"
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            struct M;
            impl ::rsciter::som::HasPassport for M {
                fn passport(&self) -> ::rsciter::Result<&'static ::rsciter::som::Passport> {
                    use ::rsciter::som::*;
                    let passport = impl_passport!(self, M);
                    passport
                }
            }
            impl ::rsciter::som::Methods for M {
                fn methods() -> &'static [::rsciter::Result<::rsciter::som::MethodDef>] {
                    static METHODS: std::sync::OnceLock<
                        [::rsciter::Result<::rsciter::som::MethodDef>; 0usize],
                    > = std::sync::OnceLock::new();
                    METHODS.get_or_init(|| { [] })
                }
            }
            #[allow(non_snake_case)]
            impl M {}
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            mod M_mod {
                use super::*;
                pub struct NsObject {
                    pub msg: String,
                }
                impl NsObject {
                    pub fn test(&self) -> String {
                        format!("Test: {}", self.msg)
                    }
                }
                impl ::rsciter::som::HasPassport for NsObject {
                    fn passport(&self) -> ::rsciter::Result<&'static ::rsciter::som::Passport> {
                        use ::rsciter::som::*;
                        let passport = impl_passport!(self, NsObject);
                        passport
                    }
                }
                impl ::rsciter::som::Fields for NsObject {
                    fn fields() -> &'static [::rsciter::Result<::rsciter::som::PropertyDef>] {
                        static FIELDS: std::sync::OnceLock<
                            [::rsciter::Result<::rsciter::som::PropertyDef>; 1usize],
                        > = std::sync::OnceLock::new();
                        use ::rsciter::impl_prop;
                        FIELDS.get_or_init(|| [impl_prop!(NsObject::msg)])
                    }
                }
                impl ::rsciter::som::Methods for NsObject {
                    fn methods() -> &'static [::rsciter::Result<::rsciter::som::MethodDef>] {
                        static METHODS: std::sync::OnceLock<
                            [::rsciter::Result<::rsciter::som::MethodDef>; 1usize],
                        > = std::sync::OnceLock::new();
                        METHODS
                            .get_or_init(|| {
                                [
                                    {
                                        unsafe extern "C" fn test_thunk(
                                            thing: *mut ::rsciter::bindings::som_asset_t,
                                            argc: ::rsciter::bindings::UINT,
                                            argv: *const ::rsciter::bindings::SCITER_VALUE,
                                            p_result: *mut ::rsciter::bindings::SCITER_VALUE,
                                        ) -> ::rsciter::bindings::SBOOL {
                                            let args = ::rsciter::args_from_raw_parts(argv, argc);
                                            let mut asset_mut = ::rsciter::som::AssetRefMut::<
                                                NsObject,
                                            >::new(thing);
                                            match asset_mut.call_test(args) {
                                                Ok(Some(res)) => {
                                                    *p_result = res.take();
                                                    1
                                                }
                                                Ok(_) => 1,
                                                Err(_err) => 0,
                                            }
                                        }
                                        ::rsciter::som::Atom::new(c"test")
                                            .map(|name| ::rsciter::som::MethodDef {
                                                reserved: std::ptr::null_mut(),
                                                name: name.into(),
                                                params: 0usize,
                                                func: Some(test_thunk),
                                            })
                                    },
                                ]
                            })
                    }
                }
                #[allow(non_snake_case)]
                impl NsObject {
                    fn call_test(
                        &mut self,
                        args: &[::rsciter::Value],
                    ) -> ::rsciter::Result<Option<::rsciter::Value>> {
                        let _ = args;
                        let result = self.test();
                        ::rsciter::conv::ToValue::to_value(result).map(|res| Some(res))
                    }
                }
            }
            impl M {
                pub fn new() -> ::rsciter::Result<::rsciter::som::GlobalAsset<M>> {
                    ::rsciter::som::GlobalAsset::new(M)
                }
            }
        "##]]
        .assert_eq(&result);
    }
}
