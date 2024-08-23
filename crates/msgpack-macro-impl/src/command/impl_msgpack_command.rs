use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let generic_arg = item_fn
        .sig
        .generics
        .type_params()
        .next()
        .map(|e| e.ident.clone())
        .unwrap_or_else(|| format_ident!("__R"));

    let return_ty = match &item_fn.sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    let command_name = &fn_name;
    let command_name_str = command_name.to_string();
    let command_args_def = gen_command_args(item_fn);
    let command_args = match get_command_args(item_fn) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let command_arg_names: Vec<_> = command_args.iter().map(|p| p.0.as_str()).collect();
    let command_arg_fields: Vec<_> = command_args
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("arg{}", i))
        .collect();

    let deps = quote!(::tauri_plugin_bin_ipc_msgpack::__deps);

    quote! {
        impl<#generic_arg: #deps::tauri::Runtime> #deps::TauriPluginBinIpcMessagePackCommand<#generic_arg> for #command_name {
            const NAME: &'static #deps::str = #command_name_str;

            async fn handle(
                &self,
                app: &#deps::tauri::AppHandle<#generic_arg>,
                payload: #deps::std::vec::Vec<u8>,
            ) -> #deps::HandleResult {
                use ::tauri_plugin_bin_ipc_msgpack::__deps::*; //todo

                #command_args_def

                let mut command_args = CommandArgs::deserialize(&app, &payload)?;

                let result = Self::invoke(
                    #(
                        command_args.#command_arg_fields.take().ok_or(MissingArgumentError {
                            command_name: #command_name_str,
                            arg_name: #command_arg_names
                        })?
                    ),*
                ).await?;

                let response = WrapResult::<#return_ty>(PhantomData).wrap(result)?;

                Ok(rmp_serde::to_vec(&response)?)
            }
        }
    }
}

fn gen_command_args(item_fn: &syn::ItemFn) -> TokenStream {
    let fn_name = &item_fn.sig.ident;
    let command_name = &fn_name;
    let command_name_str = command_name.to_string();
    let generic_arg = item_fn
        .sig
        .generics
        .type_params()
        .next()
        .map(|e| e.ident.clone())
        .unwrap_or_else(|| format_ident!("__R"));

    let visitor_expecting = format!("arguments of command `{}`", command_name_str);
    let command_args = match get_command_args(item_fn) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let command_arg_names: Vec<_> = command_args.iter().map(|p| p.0.as_str()).collect();
    let command_arg_fields: Vec<_> = command_args
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("arg{}", i))
        .collect();
    let command_arg_types: Vec<_> = command_args.iter().map(|p| &p.1).collect();

    quote! {
        struct CommandArgs<#generic_arg: tauri::Runtime> {
            __marker: PhantomData<#generic_arg>,
            #(
                #command_arg_fields : Option<#command_arg_types>
            ),*
        }

        impl<#generic_arg: tauri::Runtime> CommandArgs<#generic_arg> {
            fn deserialize(
                app: &tauri::AppHandle<#generic_arg>,
                payload: &[u8],
            ) -> Result<Self, rmp_serde::decode::Error> {
                struct Visitor<'a, #generic_arg: tauri::Runtime>(&'a tauri::AppHandle<#generic_arg>);

                impl<'de, #generic_arg: tauri::Runtime> serde::de::Visitor<'de> for Visitor<'de, #generic_arg> {
                    type Value = CommandArgs<#generic_arg>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(#visitor_expecting)
                    }

                    fn visit_map<__A>(self, mut map: __A) -> Result<Self::Value, __A::Error>
                    where
                        __A: serde::de::MapAccess<'de>,
                    {
                        let mut command_args = CommandArgs {
                            __marker: PhantomData,
                            #(#command_arg_fields: None),*
                        };

                        while let Some(k) = map.next_key::<&str>()? {
                            match k {
                                #(
                                    #command_arg_names => {
                                        struct Proxy<'a, #generic_arg: tauri::Runtime>(&'a tauri::AppHandle<#generic_arg>);
                                        impl<'de, #generic_arg: tauri::Runtime> serde::de::DeserializeSeed<'de> for Proxy<'de, #generic_arg> {
                                            type Value = #command_arg_types;

                                            fn deserialize<__D>(self, de: __D) -> Result<Self::Value, __D::Error>
                                            where
                                                __D: serde::Deserializer<'de>,
                                            {
                                                let de = CommandArgDeserializer {
                                                    app_handle: self.0,
                                                    de,
                                                };

                                                DeserializeProxy::<#generic_arg, #command_arg_types>(PhantomData).deserialize(de)
                                            }
                                        }

                                        let arg = map.next_value_seed(Proxy(&self.0))?;
                                        command_args.#command_arg_fields = Some(arg);
                                    }
                                )*
                                _ => {
                                    return Err(<__A::Error as serde::de::Error>::unknown_field(
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

fn get_command_args(item_fn: &syn::ItemFn) -> Result<Vec<(String, syn::Type)>, TokenStream> {
    item_fn
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
        .collect()
}
