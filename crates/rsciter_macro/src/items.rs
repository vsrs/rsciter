use crate::TokenStream2;
use quote::quote;
use syn::{self, spanned::Spanned, FnArg, Ident, ItemFn, PatIdent, ReturnType, Type, TypePath};

#[allow(dead_code)]
pub struct MethodInfo<'m> {
    method: &'m ItemFn,
    call_ident: Ident,
    args: Vec<ArgInfo<'m>>,
}

impl<'m> MethodInfo<'m> {
    pub fn new(method: &'m syn::ItemFn, call_ident: Ident) -> syn::Result<Self> {
        let args = Self::get_args(method)?;
        Ok(Self {
            method,
            call_ident,
            args,
        })
    }

    pub fn name(&self) -> String {
        self.method.sig.ident.to_string()
    }

    pub fn call_ident(&self) -> &Ident {
        &self.call_ident
    }

    #[allow(unreachable_code)]
    pub fn body(&self, prefix: &TokenStream2) -> TokenStream2 {
        let _ = prefix;
        let arg_count = self.args.len();
        let (prelude, args) = if arg_count == 0 {
            (quote! { let _ = args; }, quote! {})
        } else {
            let method_name = self.name();

            let mut arg_names = Vec::new();
            let mut convertions = Vec::new();
            let mut calls = Vec::new();
            for (idx, arg) in self.args.iter().enumerate() {
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

        let method = &self.method.sig.ident;
        if matches!(self.method.sig.output, ReturnType::Default) {
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

    fn get_args(method: &ItemFn) -> syn::Result<Vec<ArgInfo>> {
        let mut res = Vec::new();
        for arg in method.sig.inputs.iter() {
            match arg {
                FnArg::Receiver(s) => {
                    return Err(syn::Error::new(
                        s.span(),
                        "#[rsciter::xmod] self parameter is not supported",
                    ));
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
                        pat_type: &*typed.ty,
                    };
                    res.push(arg_info)
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

        match path {
            Some(path) if Self::last_segment_is(path, "str") => quote! {
                let #ident = <String as ::rsciter::conv::FromValue>::from_value(&args[#idx])?;
            },

            Some(path) if !Self::last_segment_is(path, "Value") => quote! {
                let #ident = <#path as ::rsciter::conv::FromValue> :: from_value(&args[#idx])?;
            },

            _ => quote! {
                let #ident = ::rsciter::conv::FromValue::from_value(&args[#idx])?;
            },
        }
    }

    fn last_segment_is(path: &TypePath, val: &str) -> bool {
        if let Some(last) = path.path.segments.last() {
            if last.ident.to_string() == val {
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
