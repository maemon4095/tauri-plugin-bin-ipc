use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn generate_handler(args: TokenStream) -> TokenStream {
    let args: GenerateHandlerArgs = match syn::parse2(args) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let commands = args.0.iter();
    let deps = quote!(::tauri_plugin_bin_ipc_msgpack::__deps);

    quote! {
        {
            struct GeneratedHandler;

            impl<R: #deps::tauri::Runtime> #deps::BinIpcHandler<R> for GeneratedHandler {
                type Future = #deps::FlattenJoinHandle<
                    #deps::Vec<#deps::u8>
                >;

                fn handle(&self, app: &#deps::tauri::AppHandle<R>, name: String, payload: Vec<u8>) -> Self::Future {
                    match name.as_str() {
                        #(
                            <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::NAME => {
                                let handle = #deps::tauri::async_runtime::spawn(
                                    <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::handle(
                                        &#commands,
                                        app,
                                        payload
                                    )
                                );

                                todo!()
                            }
                        )*

                        _ => {todo!()}
                    }
                }
            }

            GeneratedHandler
        }
    }
}

struct GenerateHandlerArgs(Punctuated<syn::Ident, syn::Token![,]>);

impl syn::parse::Parse for GenerateHandlerArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let punctuated = Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(input)?;
        Ok(Self(punctuated))
    }
}
