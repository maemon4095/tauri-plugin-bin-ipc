use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn generate_bin_handler(args: TokenStream) -> TokenStream {
    let args: GenerateHandlerArgs = match syn::parse2(args) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let commands = args.0.iter();
    let deps = quote!(::tauri_plugin_bin_ipc_msgpack::__deps);

    quote! {
        {
            struct GeneratedHandler;

            #[derive(#deps::Debug)]
            struct NoSuchCommandError(String);

            impl #deps::std::fmt::Display for NoSuchCommandError {
                fn fmt(&self, f: &mut #deps::std::fmt::Formatter) -> #deps::std::fmt::Result {
                    f.write_str("Command `")?;
                    f.write_str(&self.0)?;
                    f.write_str("` does not exists.")
                }
            }

            impl #deps::StdError for NoSuchCommandError {}

            impl<R: #deps::tauri::Runtime> #deps::BinIpcHandler<R> for GeneratedHandler {
                type Future = #deps::FlattenJoinHandle<
                    #deps::Vec<#deps::u8>
                >;

                fn handle(&self, app: &#deps::tauri::AppHandle<R>, name: &#deps::str, payload: &[#deps::u8]) -> #deps::Result<Self::Future, #deps::BoxError> {
                    match name {
                        #(
                            <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::NAME => #deps::Ok(#deps::tauri::async_runtime::spawn(
                                <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::handle(
                                    &#commands,
                                    app,
                                    payload
                                )
                            ).into()),
                        )*
                        _ => #deps::Err(#deps::Box::new(NoSuchCommandError(name.to_string())) as #deps::BoxError)
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
