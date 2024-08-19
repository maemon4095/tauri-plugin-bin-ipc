use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let command_name_str = command_name.to_string();

    quote! {
        impl<R: ::tauri::Runtime> TauriPluginBinIpcMessagePackCommand<R> for #command_name {
            const NAME: &'static str = #command_name_str;

            async fn handle(
                &self,
                app: ::tauri::AppHandle<R>,
                payload: ::std::vec::Vec<u8>,
            ) -> Vec<u8> {

                // todo: Deserializerを実装して各引数をデシリアライズする形で実装する。
                // デシリアライザは、rmp_serdeのでシリアライザを内部に持つ。
            }
        }
    }
}
