use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn generate_bin_handler(args: TokenStream) -> TokenStream {
    let args: GenerateHandlerArgs = match syn::parse2(args) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let commands = args.0.iter();
    let deps =
        quote!(::tauri_plugin_bin_ipc_msgpack::__bin_ipc_deps_0cc84921_b5dc_4044_86a1_58ee53f2643a);
    let generated_handler = quote!(__GeneratedHandler_91d5ee51_f364_4349_b2f7_fcded5349e2e);

    quote! {
        {
            #[allow(non_camel_case_types)]
            struct #generated_handler;

            impl<R: #deps::Runtime> #deps::BinIpcHandler<R> for #generated_handler {
                type Future = #deps::FlattenJoinHandle<
                    #deps::Vec<#deps::u8>
                >;

                fn handle(&self, app: &#deps::AppHandle<R>, name: &#deps::str, payload: &[#deps::u8]) -> #deps::Result<Self::Future, #deps::BoxError> {
                    match name {
                        #(
                            <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::NAME => #deps::Ok(#deps::spawn(
                                <#commands as #deps::TauriPluginBinIpcMessagePackCommand<R>>::handle(
                                    &#commands,
                                    app,
                                    payload
                                )
                            ).into()),
                        )*
                        _ => #deps::Err(#deps::Box::new(#deps::NoSuchCommandError(name.to_string())) as #deps::BoxError)
                    }
                }
            }

            #generated_handler
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
