mod impl_msgpack_command;
mod invoke_fn;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn bin_command(_attr: TokenStream, body: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let ctx = match CommandGenerationContext::new(item_fn) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let vis = &ctx.item_fn.vis;
    let command_name = &ctx.command_name;
    let invoke_fn = invoke_fn::gen(&ctx);
    let impl_msgpack_command = impl_msgpack_command::gen(&ctx);

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #command_name;

        impl #command_name  {
            #invoke_fn
        }

        #impl_msgpack_command
    }
}

struct CommandGenerationContext {
    pub item_fn: syn::ItemFn,
    pub ident_suffix: &'static str,
    pub runtime_generic_param: syn::Ident,
    pub command_name: syn::Ident,
    pub deps_path: syn::Path,
    pub return_type: syn::Type,
}

impl CommandGenerationContext {
    fn new(item_fn: syn::ItemFn) -> Result<Self, TokenStream> {
        let ident_suffix = crate::ident_suffix();
        let deps_path = crate::deps_path();
        let generics = &item_fn.sig.generics;
        let command_name = item_fn.sig.ident.clone();
        let runtime_generic_param = match generics.params.first() {
            Some(syn::GenericParam::Type(e)) => e.ident.clone(),
            None => format_ident!("__R_{}", ident_suffix),
            _ => {
                return Err(quote!(compile_error!(
                    "bin ipc command only have type parameter."
                )))
            }
        };
        let return_type = match &item_fn.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, t) => t.as_ref().clone(),
        };
        Ok(Self {
            ident_suffix,
            item_fn,
            command_name,
            deps_path,
            runtime_generic_param,
            return_type,
        })
    }
}
