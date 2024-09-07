use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use crate::sciter_mod;

pub fn passport_impl(attr: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
    const MESSAGE: &str =
        "the #[rsciter::passport] attribute can only be applied to impl block!";

    match syn::parse2::<syn::ItemImpl>(input) {
        Err(e) => Err(syn::Error::new(e.span(), MESSAGE.to_string())),
        Ok(impl_block) => process_passport_impl_block(attr, impl_block),
    }
}

fn process_passport_impl_block(
    attr: TokenStream2,
    block: syn::ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    let info = sciter_mod::SciterMod::from_impl_block(&block)?;
    let code = generate_passport_provider(&info, attr);

    Ok(quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #block

        #code
    })
}

fn generate_passport_provider(info: &sciter_mod::SciterMod, attr: TokenStream2) -> TokenStream2 {
    let provider_struct_name = info.name_path();
    let (names, calls, implementations) = info.passport_methods();
    let n_methods = names.len();
    let method_index: std::iter::Map<std::ops::Range<usize>, fn(usize) -> syn::Index> = (0..n_methods).map(syn::Index::from);
    let mut interface_name = attr.to_string();
    interface_name = if interface_name.is_empty() {
        provider_struct_name.to_token_stream().to_string()
    } else {
        interface_name
    };
    quote! {
        impl ::rsciter::som::Passport for &mut #provider_struct_name {
          	fn get_passport(&self) -> &'static bindings::som_passport_t {
                #(#implementations )*
                let sapi = ::rsciter::api::sapi().unwrap();
                type ObjectMethods = [bindings::som_method_def_t; #n_methods];
                let mut methods = Box::new(ObjectMethods::default());
                #(
                    let method = &mut methods[#method_index];
                    #calls
                )*
                let mut passport = Box::new(bindings::som_passport_t::default());
                passport.name = sapi.atom(#interface_name).unwrap();
        
                passport.n_methods = #n_methods;
                passport.methods = Box::into_raw(methods) as *const _;
                Box::leak(passport)
            }
        }
    }
}