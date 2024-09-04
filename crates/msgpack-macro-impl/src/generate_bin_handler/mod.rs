use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

pub fn generate_bin_handler(args: TokenStream) -> TokenStream {
    let args: GenerateHandlerArgs = match syn::parse2(args) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let commands = args.0.iter();
    let deps = crate::deps_path();
    let generated_handler = format_ident!("__GeneratedHandler_{}", crate::ident_suffix());

    quote! {
        {
            #[allow(non_camel_case_types)]
            struct #generated_handler;

            impl<R: #deps::Runtime> #deps::BinIpcHandler<R> for #generated_handler {
                type Future = #deps::FlattenJoinHandle<
                    #deps::Vec<#deps::u8>
                >;

                fn handle(&self, app: &#deps::AppHandle<R>, name: &#deps::str, payload: &[#deps::u8]) -> #deps::Result<Self::Future, #deps::BinIpcError> {
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
                        _ => #deps::Err(#deps::BinIpcError::new_reportable(#deps::NoSuchCommandError(name.to_string())))
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
