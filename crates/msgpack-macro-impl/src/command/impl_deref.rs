use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let params_list = &item_fn.sig.inputs;
    let return_type = match &item_fn.sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, t) => quote!(#t),
    };

    let param_type_list: Punctuated<&Box<syn::Type>, syn::Token![,]> = params_list
        .iter()
        .map(|p| match p {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(p) => &p.ty,
        })
        .collect();

    let invoke_return_ty = quote!(::tauri::async_runtime::JoinHandle<#return_type>);

    quote! {
        impl ::std::ops::Deref for #command_name {
            type Target = fn(#param_type_list) -> #invoke_return_ty;

            fn deref(&self) -> &Self::Target {
                return &(Self::invoke as Self::Target);
            }
        }
    }
}
