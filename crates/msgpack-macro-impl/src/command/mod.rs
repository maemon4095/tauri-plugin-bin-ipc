mod impl_msgpack_command;
mod invoke_fn;

use proc_macro2::TokenStream;
use quote::quote;

pub fn bin_command(_attr: TokenStream, body: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let vis = &item_fn.vis;
    let fn_name = &item_fn.sig.ident;
    let generics = &item_fn.sig.generics;
    let command_name = &fn_name;
    let invoke_fn = invoke_fn::gen(&item_fn);
    let impl_msgpack_command = impl_msgpack_command::gen(&item_fn);

    if generics.params.len() > 1 {
        return quote!(compile_error!(
            "bin ipc command could have at most one generic parameter."
        ));
    }
    if generics.lifetimes().count() > 0 {
        return quote!(compile_error!(
            "bin ipc command cannot have lifetime parameter."
        ));
    }

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #command_name;

        impl #command_name  {
            #invoke_fn
        }

        #impl_msgpack_command
    }
}
