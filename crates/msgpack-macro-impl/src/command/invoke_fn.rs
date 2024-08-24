use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use super::CommandGenerationContext;

pub fn gen(ctx: &CommandGenerationContext) -> TokenStream {
    let deps = &ctx.deps_path;
    let item_fn = &ctx.item_fn;
    let fn_name = &item_fn.sig.ident;
    let params_list = &item_fn.sig.inputs;
    let return_type = &ctx.return_type;
    let generics = &item_fn.sig.generics;
    let where_clause = &item_fn.sig.generics.where_clause;
    let args_list: Punctuated<&Box<syn::Pat>, syn::Token![,]> = params_list
        .iter()
        .map(|p| match p {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(p) => &p.pat,
        })
        .collect();

    if item_fn.sig.asyncness.is_some() {
        quote! {
            pub fn invoke #generics (#params_list) -> #deps::TauriJoinHandle<#return_type> #where_clause {
                return #deps::spawn(#fn_name(#args_list));

                #item_fn
            }
        }
    } else {
        quote! {
            pub fn invoke #generics (#params_list) -> #deps::TauriJoinHandle<#return_type> #where_clause {
                return #deps::spawn_blocking(move || { #fn_name(#args_list) });

                #item_fn
            }
        }
    }
}
