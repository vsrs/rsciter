use crate::TokenStream2;
use quote::quote;
use syn::{
    self, spanned::Spanned, FnArg, Ident, PatIdent, Receiver, ReturnType, Signature, Type, TypePath,
};

#[allow(dead_code)]
pub struct MethodInfo<'m> {
    sig: &'m Signature,
    call_ident: Ident,
    args: Vec<Arg<'m>>,
}

enum Arg<'m> {
    Reciever(#[allow(dead_code)] &'m Receiver),
    Arg(ArgInfo<'m>),
}

impl<'m> MethodInfo<'m> {
    pub fn new(signature: &'m syn::Signature) -> syn::Result<Self> {
        let call_ident = quote::format_ident!("call_{}", &signature.ident);

        if signature.generics.lt_token.is_some() {
            return Err(syn::Error::new(
                signature.generics.span(),
                "#[rsciter::xmod] Generic functions are not supported!",
            ));
        }
        if signature.variadic.is_some() {
            return Err(syn::Error::new(
                signature.generics.span(),
                "#[rsciter::xmod] Variadic functions are not supported!",
            ));
        }

        let args = Self::get_args(signature)?;
        Ok(Self {
            sig: signature,
            call_ident,
            args,
        })
    }

    pub fn name(&self) -> String {
        self.sig.ident.to_string()
    }

    pub fn call_ident(&self) -> &Ident {
        &self.call_ident
    }

    pub fn args_count(&self) -> usize {
        self.args.len()
    }

    #[allow(unreachable_code)]
    pub fn body(&self, prefix: &TokenStream2) -> TokenStream2 {
        let mut arg_count = self.args.len();
        if matches!(self.args.first(), Some(Arg::Reciever(_))) {
            arg_count -= 1;
        }

        let (prelude, args) = if arg_count == 0 {
            (quote! { let _ = args; }, quote! {})
        } else {
            let method_name = self.name();

            let mut arg_names = Vec::new();
            let mut convertions = Vec::new();
            let mut calls = Vec::new();
            for (idx, arg) in self
                .args
                .iter()
                .filter_map(|it| match it {
                    Arg::Arg(a) => Some(a),
                    _ => None,
                })
                .enumerate()
            {
                arg_names.push(arg.ident());
                convertions.push(arg.convertion(idx));
                calls.push(arg.call());
            }

            (
                {
                    quote! {
                        if args.len() != #arg_count {
                            return Err(::rsciter::Error::ScriptingInvalidArgCount(#method_name .to_string()))
                        }

                        #(#convertions)*
                    }
                },
                {
                    quote! { #(#calls),* }
                },
            )
        };

        let method = &self.sig.ident;
        if matches!(self.sig.output, ReturnType::Default) {
            quote! {
                #prelude
                #prefix #method ( #args );
                Ok(None)
            }
        } else {
            quote! {
                #prelude
                let result = #prefix #method ( #args );
                ::rsciter::conv::ToValue::to_value(result).map(|res| Some(res))
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn passport_body(&self, prefix: &TokenStream2) -> TokenStream2 {
        let mut arg_count = self.args.len();
        if matches!(self.args.first(), Some(Arg::Reciever(_))) {
            arg_count -= 1;
        }

        let (prelude, args) = if arg_count == 0 {
            (quote! { let _ = argc; }, quote! {})
        } else {
            let method_name = self.name();

            let mut arg_names = Vec::new();
            let mut convertions = Vec::new();
            let mut calls = Vec::new();
            for (idx, arg) in self
                .args
                .iter()
                .filter_map(|it| match it {
                    Arg::Arg(a) => Some(a),
                    _ => None,
                })
                .enumerate()
            {
                arg_names.push(arg.ident());
                convertions.push(arg.convertion(idx));
                calls.push(arg.call());
            }

            (
                {
                    quote! {
                        if args.len() != #arg_count {
                            *p_result = Value::error_string(::rsciter::Error::ScriptingInvalidArgCount(#method_name .to_string()).to_string())
                                .unwrap()
                                .take();
                            return true as crate::bindings::SBOOL;
                        }

                        #(#convertions)*
                    }
                },
                {
                    quote! { #(#calls),* }
                },
            )
        };

        let method = &self.sig.ident;
        if matches!(self.sig.output, ReturnType::Default) {
            quote! {
                #prelude
                #prefix #method ( #args );
                Ok(None)
            }
        } else {
            quote! {
                #prelude
                let result = me.#method ( #args );
                let result = ::rsciter::conv::ToValue::to_value(result).map(|res| res);
                *p_result = result.unwrap().take();
                return true as crate::bindings::SBOOL;
            }
        }
    }

    fn get_args(sig: &Signature) -> syn::Result<Vec<Arg>> {
        let mut res = Vec::new();
        for arg in sig.inputs.iter() {
            match arg {
                FnArg::Receiver(s) => {
                    res.push(Arg::Reciever(s));
                }
                FnArg::Typed(typed) => {
                    match typed.ty.as_ref() {
                        Type::Reference(reference) if reference.mutability.is_some() => {
                            return Err(syn::Error::new(
                                reference.span(),
                                "#[rsciter::xmod] mutable references in function parameters are not supported!",
                            ))
                        },
                        _ => (),
                    }

                    let ident = match typed.pat.as_ref() {
                        syn::Pat::Ident(pat) => pat,
                        it => {
                            return Err(syn::Error::new(
                                it.span(),
                                "#[rsciter::xmod] Only simple parameter variable bindings are supported!",
                            ))
                        }
                    };

                    let arg_info = ArgInfo {
                        arg,
                        ident,
                        pat_type: &typed.ty,
                    };
                    res.push(Arg::Arg(arg_info))
                }
            }
        }

        Ok(res)
    }
}

#[allow(dead_code)]
struct ArgInfo<'m> {
    arg: &'m FnArg,
    ident: &'m PatIdent,
    pat_type: &'m Type,
}

impl ArgInfo<'_> {
    pub fn ident(&self) -> &Ident {
        &self.ident.ident
    }

    pub fn call(&self) -> TokenStream2 {
        let ident = self.ident();
        if matches!(self.pat_type, Type::Reference(_)) {
            quote! { & #ident }
        } else {
            quote! { #ident }
        }
    }

    pub fn convertion(&self, idx: usize) -> TokenStream2 {
        let ident = self.ident();
        let path = Self::get_type_path(self.pat_type);
        let arg_name = ident.to_string();

        match path {
            Some(path) if Self::last_segment_is(path, "str") => quote! {
                let #ident = <String as ::rsciter::conv::FromValue>::from_value(&args[#idx]).map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(#arg_name, Box::new(err)) ).unwrap();
            },

            Some(path) if !Self::last_segment_is(path, "Value") => quote! {
                let #ident = <#path as ::rsciter::conv::FromValue>::from_value(&args[#idx]).map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(#arg_name, Box::new(err)) ).unwrap();
            },

            _ => quote! {
                let #ident = ::rsciter::conv::FromValue::from_value(&args[#idx]).map_err(|err| ::rsciter::Error::ScriptingInvalidArgument(#arg_name, Box::new(err)) ).unwrap();
            },
        }
    }

    fn last_segment_is(path: &TypePath, val: &str) -> bool {
        if let Some(last) = path.path.segments.last() {
            if last.ident == val {
                return true;
            }
        }

        false
    }

    fn get_type_path(ty: &Type) -> Option<&TypePath> {
        match ty {
            Type::Path(path) => Some(path),
            Type::Reference(reference) => Self::get_type_path(reference.elem.as_ref()),
            _ => None,
        }
    }
}
