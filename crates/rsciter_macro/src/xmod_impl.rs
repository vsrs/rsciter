use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::sciter_mod::SciterMod;

pub fn xmod(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
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
    attr: TokenStream,
    mut module: syn::ItemMod,
) -> Result<TokenStream, syn::Error> {
    let (info, module) = SciterMod::prepare(&mut module, attr)?;
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
    _attr: TokenStream,
    block: syn::ItemImpl,
) -> Result<TokenStream, syn::Error> {
    let info = SciterMod::from_impl_block(&block)?;
    let code = generate_xfunction_provider(&info);

    Ok(quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #block

        #code
    })
}

fn generate_xfunction_provider(info: &SciterMod) -> TokenStream {
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
    use crate::tests::expand;
    use expect_test::expect;

    use super::*;

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
            xmod,
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
            xmod,
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
}
