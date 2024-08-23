use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let deps = quote!(::tauri_plugin_bin_ipc_msgpack::__deps);
    let fn_name = &item_fn.sig.ident;
    let params_list = &item_fn.sig.inputs;
    let return_type = match &item_fn.sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, t) => quote!(#t),
    };
    let generics = &item_fn.sig.generics;
    let where_clause = &item_fn.sig.generics.where_clause;
    let args_list: Punctuated<&Box<syn::Pat>, syn::Token![,]> = params_list
        .iter()
        .map(|p| match p {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(p) => &p.pat,
        })
        .collect();

    let invoke_return_ty = quote!(#deps::tauri::async_runtime::JoinHandle<#return_type>);

    if item_fn.sig.asyncness.is_some() {
        quote! {
            pub fn invoke #generics (#params_list) -> #invoke_return_ty #where_clause {
                return #deps::tauri::async_runtime::spawn(#fn_name(#args_list));

                #item_fn
            }
        }
    } else {
        quote! {
            pub fn invoke #generics (#params_list) -> #invoke_return_ty #where_clause {
                return #deps::tauri::async_runtime::spawn_blocking(move || { #fn_name(#args_list) });

                #item_fn
            }
        }
    }
}
