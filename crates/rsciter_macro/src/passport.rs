use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
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
    _attr: TokenStream2,
    block: syn::ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    let info = sciter_mod::SciterMod::from_impl_block(&block)?;
    let code = generate_passport_provider(&info);

    Ok(quote! {
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #block

        #code
    })
}

fn generate_passport_provider(info: &sciter_mod::SciterMod) -> TokenStream2 {
    let provider_struct_name = info.name_path();
    let (names, calls, implementations) = info.passport_methods();
    let n_methods = names.len();
    let method_index: std::iter::Map<std::ops::Range<usize>, fn(usize) -> syn::Index> = (0..n_methods).map(syn::Index::from);

    quote! {
        impl ::rsciter::som::Passport for #provider_struct_name {
          	fn get_passport(&self) -> &'static som_passport_t {
                #(#implementations )*
                let sapi = ::rsciter::api::sapi().unwrap();
                type ObjectMethods = [som_method_def_t; #n_methods];
                let mut methods = Box::new(ObjectMethods::default());
                #(
                    let method = &mut methods[#method_index];
                    #calls
                )*
                let mut passport = Box::new(som_passport_t::default());
                passport.name = sapi.atom(stringify!(#provider_struct_name)).unwrap();
        
                passport.n_methods = #n_methods;
                passport.methods = Box::into_raw(methods) as *const _;
                Box::leak(passport)
            }
        }
    }
}