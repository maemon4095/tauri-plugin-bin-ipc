use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::CommandGenerationContext;

struct CommandArgGenerationContext<'a> {
    command_name: &'a syn::Ident,
    runtime_ty: &'a syn::Ident,
    command_arg_name: syn::Ident,
    return_type: &'a syn::Type,
    deps_path: &'a syn::Path,
    ident_suffix: &'static str,
    arg_names: Vec<String>,
    fields: Vec<syn::Ident>,
    types: Vec<syn::Type>,
}

pub fn gen(ctx: &CommandGenerationContext) -> TokenStream {
    let ctx = match CommandArgGenerationContext::new(ctx) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let command_name = &ctx.command_name;
    let command_name_str = ctx.command_name.to_string();
    let command_arg_names = &ctx.arg_names;
    let command_arg_fields = &ctx.fields;
    let deps = &ctx.deps_path;
    let runtime_ty = &ctx.runtime_ty;
    let return_ty = &ctx.return_type;

    let command_arg_name = &ctx.command_arg_name;
    let command_args_def = gen_command_args(&ctx);

    quote! {
        impl<#runtime_ty: #deps::Runtime> #deps::TauriPluginBinIpcMessagePackCommand<#runtime_ty> for #command_name {
            const NAME: &'static #deps::str = #command_name_str;

            fn handle(
                &self,
                app: &#deps::AppHandle<#runtime_ty>,
                payload: &[#deps::u8],
            ) -> impl 'static + #deps::Future<Output = #deps::HandleResult> + #deps::Send {
                #command_args_def

                let mut command_args = match #command_arg_name::deserialize(&app, &payload) {
                    #deps::Ok(v) => v,
                    #deps::Err(e) => return #deps::OrFuture::F0(#deps::ready_future(#deps::Err(#deps::BinIpcError::new_reportable(e))))
                };

                #deps::OrFuture::F1((move || async move {
                    let result = Self::invoke(
                        #(
                            command_args.#command_arg_fields.take().ok_or(#deps::MissingArgumentError {
                                command_name: #command_name_str,
                                arg_name: #command_arg_names
                            }).map_err(#deps::BinIpcError::new_reportable)?
                        ),*
                    ).await?;

                    let response = #deps::wrap_result::<#return_ty>().wrap(result)?;

                    #deps::Ok(#deps::encode_to_vec(&response)?)
                })())
            }
        }
    }
}

fn gen_command_args(ctx: &CommandArgGenerationContext) -> TokenStream {
    let command_name_str = ctx.command_name.to_string();
    let visitor_expecting = format!("arguments of command `{}`", command_name_str);
    let command_arg_names = &ctx.arg_names;
    let command_arg_fields = &ctx.fields;
    let command_arg_types = &ctx.types;
    let deps = &ctx.deps_path;
    let runtime_ty = &ctx.runtime_ty;
    let command_arg_name = &ctx.command_arg_name;
    let deserializer_ty = format_ident!("__D_{}", ctx.ident_suffix);
    let map_access_ty = format_ident!("__A_{}", ctx.ident_suffix);

    quote! {
        struct #command_arg_name<#runtime_ty: #deps::Runtime> {
            __marker: #deps::PhantomData<#runtime_ty>,
            #(
                #command_arg_fields : #deps::Option<#command_arg_types>
            ),*
        }

        impl<#runtime_ty: #deps::Runtime> #command_arg_name<#runtime_ty> {
            fn deserialize(
                app: &#deps::AppHandle<#runtime_ty>,
                payload: &[#deps::u8],
            ) -> Result<Self, #deps::MsgpackDecodeError> {
                struct Visitor<'a, #runtime_ty: #deps::Runtime>(&'a #deps::AppHandle<#runtime_ty>);

                impl<'de, #runtime_ty: #deps::Runtime> #deps::Visitor<'de> for Visitor<'de, #runtime_ty> {
                    type Value = #command_arg_name<#runtime_ty>;

                    fn expecting(&self, formatter: &mut #deps::StdFormatter) -> #deps::StdFmtResult {
                        formatter.write_str(#visitor_expecting)
                    }

                    fn visit_map<#map_access_ty>(self, mut map: #map_access_ty) -> #deps::Result<Self::Value, #map_access_ty::Error>
                    where
                        #map_access_ty: #deps::MapAccess<'de>,
                    {
                        let mut command_args = #command_arg_name {
                            __marker: #deps::PhantomData,
                            #(#command_arg_fields: #deps::from_app_handle_proxy::<#runtime_ty, #command_arg_types>().from_app_handle(self.0)),*
                        };

                        while let #deps::Some(k) = map.next_key::<&#deps::str>()? {
                            match k {
                                #(
                                    #command_arg_names => {
                                        struct Proxy<'a, #runtime_ty: #deps::Runtime>(&'a #deps::AppHandle<#runtime_ty>);
                                        impl<'de, #runtime_ty: #deps::Runtime> #deps::DeserializeSeed<'de> for Proxy<'de, #runtime_ty> {
                                            type Value = #command_arg_types;

                                            fn deserialize<#deserializer_ty>(self, de: #deserializer_ty) -> #deps::Result<Self::Value, #deserializer_ty::Error>
                                            where
                                                #deserializer_ty: #deps::Deserializer<'de>,
                                            {
                                                #deps::deserialize_proxy::<#runtime_ty, #command_arg_types>().deserialize(de)
                                            }
                                        }

                                        let arg = map.next_value_seed(Proxy(&self.0))?;
                                        command_args.#command_arg_fields = #deps::Some(arg);
                                    }
                                )*
                                _ => {
                                    return #deps::Err(<#map_access_ty::Error as #deps::SerdeError>::unknown_field(
                                        k,
                                        &[#(#command_arg_names),*],
                                    ))
                                }
                            }
                        }

                        #deps::Ok(command_args)
                    }
                }

                <_ as #deps::Deserializer>::deserialize_map(&mut #deps::MsgpackDeserializer::from_read_ref(&payload), Visitor(app))
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

impl<'a> CommandArgGenerationContext<'a> {
    pub fn new(ctx: &'a CommandGenerationContext) -> Result<Self, TokenStream> {
        let command_args = match get_command_args(&ctx.item_fn) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let CommandGenerationContext {
            runtime_ty,
            command_name,
            deps_path,
            return_type,
            ident_suffix,
            ..
        } = ctx;
        let command_arg_name = format_ident!("__CommandArgs_{}", ctx.ident_suffix);
        let mut arg_names = Vec::new();
        let mut fields = Vec::new();
        let mut types = Vec::new();
        for (i, (arg_name, arg_ty)) in command_args.into_iter().enumerate() {
            arg_names.push(arg_name);
            fields.push(format_ident!("arg{}", i));
            types.push(arg_ty);
        }

        Ok(Self {
            ident_suffix,
            runtime_ty,
            command_name,
            deps_path,
            return_type,
            command_arg_name,
            arg_names,
            fields,
            types,
        })
    }
}
