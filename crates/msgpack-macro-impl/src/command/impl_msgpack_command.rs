use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let command_name_str = command_name.to_string();
    let payload_ty = gen_payload_ty(item_fn);

    quote! {
        impl<R: ::tauri::Runtime> TauriPluginBinIpcMessagePackCommand<R> for #command_name {
            const NAME: &'static str = #command_name_str;

            async fn handle(
                &self,
                app: ::tauri::AppHandle<R>,
                payload: ::std::vec::Vec<u8>,
            ) -> Vec<u8> {
                #payload_ty

                let payload: Payload = ::rmp_serde::from_slice(&payload).unwrap();


            }
        }
    }
}

fn gen_payload_ty(item_fn: &syn::ItemFn) -> TokenStream {
    let params_list = &item_fn.sig.inputs;

    let fields: Punctuated<&syn::PatType, syn::Token![,]> = params_list
        .iter()
        .map(|p| match p {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(p) => p,
        })
        .collect();

    quote! {
        #[derive(::serde::Deserialize)]
        struct Payload {
            #fields
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let item_fn: syn::ItemFn = syn::parse_quote! {
            fn a(arg0: Option<usize>, arg1: usize)  {}
        };

        let token = gen_payload_ty(&item_fn);

        let file: syn::File = syn::parse_quote! {
            #token
        };
        let pretty = prettyplease::unparse(&file);

        println!("{}", pretty)
    }
}
