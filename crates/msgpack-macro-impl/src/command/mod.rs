mod impl_deref;
mod impl_msgpack_command;
mod invoke_fn;

use proc_macro2::TokenStream;
use quote::quote;

pub fn command(_attr: TokenStream, body: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let vis = &item_fn.vis;
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let invoke_fn = invoke_fn::gen(&item_fn);
    let impl_deref = impl_deref::gen(&item_fn);

    quote! {
        #[doc(hidden)]
        #[allow(unused, non_camel_case_types, dead_code)]
        #vis struct #command_name;

        impl #command_name {
            #invoke_fn
        }

        #impl_deref
    }
}
