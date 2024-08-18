use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let params_list = &item_fn.sig.inputs;
    let return_type = match &item_fn.sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, t) => quote!(#t),
    };

    let args_list: Punctuated<&Box<syn::Pat>, syn::Token![,]> = params_list
        .iter()
        .map(|p| match p {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(p) => &p.pat,
        })
        .collect();

    let invoke_return_ty = quote!(::tauri::async_runtime::JoinHandle<#return_type>);

    if item_fn.sig.asyncness.is_some() {
        quote! {
            pub fn invoke(#params_list) -> #invoke_return_ty {
                return ::tauri::async_runtime::spawn(#fn_name(#args_list));

                #item_fn
            }
        }
    } else {
        quote! {
            pub fn invoke(#params_list) -> #invoke_return_ty {
                return ::tauri::async_runtime::spawn_blocking(move || { #fn_name(#args_list) });

                #item_fn
            }
        }
    }
}
