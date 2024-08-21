use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let command_name_str = command_name.to_string();
    let command_args_def = gen_command_args(item_fn);

    quote! {
        impl<R: ::tauri::Runtime> TauriPluginBinIpcMessagePackCommand<R> for #command_name {
            const NAME: &'static str = #command_name_str;

            async fn handle(
                &self,
                app: ::tauri::AppHandle<R>,
                payload: ::std::vec::Vec<u8>,
            ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                use ::tauri_plugin_bin_ipc::deps::*;
                #command_args_def

                let command_args = CommandArgs::deserialize(app, &payload)?;

            }
        }
    }
}

fn gen_command_args(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let command_name_str = command_name.to_string();

    let visitor_expecting = format!("{} arguments", command_name_str);
    let command_args: Result<Vec<(String, syn::Type)>, TokenStream> = item_fn
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => Err(quote!(compile_error!(
                "command must be standalone function."
            ))),
            syn::FnArg::Typed(arg) => match &*arg.pat {
                syn::Pat::Ident(i) => Ok((i.ident.to_string(), syn::Type::clone(&arg.ty))),
                _ => Err(quote!(compile_error!(
                    "command parameter pattern must be identifier pattern."
                ))),
            },
        })
        .collect();

    let command_args = match command_args {
        Ok(v) => v,
        Err(e) => return e,
    };

    let command_arg_names: Vec<_> = command_args.iter().map(|p| p.0.as_str()).collect();
    let command_arg_fields: Vec<_> = command_args
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("args{}", i))
        .collect();
    let command_arg_types: Vec<_> = command_args.iter().map(|p| &p.1).collect();

    quote! {
        struct CommandArgs<R: tauri::Runtime> {
            __marker: PhantomData<R>,
            #(
                #command_arg_fields : #command_arg_types
            ),*
        }

        impl<R: tauri::Runtime> CommandArgs<R> {
            fn deserialize(
                app: &::tauri::AppHandle<R>,
                payload: &[u8],
            ) -> Result<CommandArgs<R>, rmp_serde::decode::Error> {
                struct Visitor<'a, R: tauri::Runtime>(&'a tauri::AppHandle<R>);

                impl<'de, R: tauri::Runtime> serde::de::Visitor<'de> for Visitor<'de, R> {
                    type Value = CommandArgs<R>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(#visitor_expecting)
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>,
                    {
                        let mut command_args = CommandArgs::<R> {
                            __marker: PhantomData,
                            #(#command_arg_fields: None),*
                        };

                        while let Some(k) = map.next_key::<&str>()? {
                            match k {
                                #(
                                    #command_arg_names => {
                                        struct Proxy<'a, R: tauri::Runtime>(&'a tauri::AppHandle<R>);
                                        impl<'de, R: tauri::Runtime> serde::de::DeserializeSeed<'de> for Proxy<'de, R> {
                                            type Value = usize;

                                            fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
                                            where
                                                D: serde::Deserializer<'de>,
                                            {
                                                let de = CommandArgDeserializer {
                                                    app_handle: self.0,
                                                    de,
                                                };

                                                DeserializeProxy::<R, usize>(PhantomData).deserialize(de)
                                            }
                                        }

                                        let arg = map.next_value_seed(Proxy(&self.0))?;
                                        command_args.#command_arg_fields = Some(arg);
                                    }
                                )*
                                _ => {
                                    return Err(<A::Error as serde::de::Error>::unknown_field(
                                        k,
                                        &[#(#command_arg_names),*],
                                    ))
                                }
                            }
                        }

                        Ok(command_args)
                    }
                }

                rmp_serde::Deserializer::from_read_ref(&payload).deserialize_map(Visitor(app))
            }
        }
    }
}
