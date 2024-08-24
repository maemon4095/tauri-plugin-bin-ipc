use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::CommandGenerationContext;

struct CommandArgGenerationContext<'a> {
    command_name: &'a syn::Ident,
    runtime_generic_param: &'a syn::Ident,
    command_arg_name: syn::Ident,
    return_type: &'a syn::Type,
    deps_path: &'a TokenStream,
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
    let generic_arg = &ctx.runtime_generic_param;
    let return_ty = &ctx.return_type;

    let command_arg_name = &ctx.command_arg_name;
    let command_args_def = gen_command_args(&ctx);

    let map_err = quote!(map_err(|e| #deps::Box::new(e) as #deps::BoxError));

    quote! {
        impl<#generic_arg: #deps::Runtime> #deps::TauriPluginBinIpcMessagePackCommand<#generic_arg> for #command_name {
            const NAME: &'static #deps::str = #command_name_str;

            fn handle(
                &self,
                app: &#deps::AppHandle<#generic_arg>,
                payload: &[#deps::u8],
            ) -> impl 'static + #deps::Future<Output = #deps::HandleResult> + #deps::Send {
                #command_args_def

                let mut command_args = match #command_arg_name::deserialize(&app, &payload) {
                    #deps::Ok(v) => v,
                    #deps::Err(e) => return #deps::OrFuture::F0(#deps::ready_future(#deps::Err(#deps::Box::new(e) as #deps::BoxError)))
                };

                #deps::OrFuture::F1((move || async move {
                    let result = Self::invoke(
                        #(
                            command_args.#command_arg_fields.take().ok_or(#deps::MissingArgumentError {
                                command_name: #command_name_str,
                                arg_name: #command_arg_names
                            }).#map_err?
                        ),*
                    ).await.#map_err?;

                    let response = #deps::WrapResult::<#return_ty>(#deps::PhantomData).wrap(result).#map_err?;

                    #deps::Ok(#deps::encode_to_vec(&response).#map_err?)
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
    let generic_arg = &ctx.runtime_generic_param;
    let command_arg_name = &ctx.command_arg_name;

    quote! {
        struct #command_arg_name<#generic_arg: #deps::Runtime> {
            __marker: #deps::PhantomData<#generic_arg>,
            #(
                #command_arg_fields : #deps::Option<#command_arg_types>
            ),*
        }

        impl<#generic_arg: #deps::Runtime> #command_arg_name<#generic_arg> {
            fn deserialize(
                app: &#deps::AppHandle<#generic_arg>,
                payload: &[#deps::u8],
            ) -> Result<Self, #deps::MsgpackDecodeError> {
                struct Visitor<'a, #generic_arg: #deps::Runtime>(&'a #deps::AppHandle<#generic_arg>);

                impl<'de, #generic_arg: #deps::Runtime> #deps::Visitor<'de> for Visitor<'de, #generic_arg> {
                    type Value = #command_arg_name<#generic_arg>;

                    fn expecting(&self, formatter: &mut #deps::StdFormatter) -> #deps::StdFmtResult {
                        formatter.write_str(#visitor_expecting)
                    }

                    fn visit_map<__A>(self, mut map: __A) -> #deps::Result<Self::Value, __A::Error>
                    where
                        __A: #deps::MapAccess<'de>,
                    {
                        let mut command_args = #command_arg_name {
                            __marker: #deps::PhantomData,
                            #(#command_arg_fields: #deps::None),*
                        };

                        while let #deps::Some(k) = map.next_key::<&#deps::str>()? {
                            match k {
                                #(
                                    #command_arg_names => {
                                        struct Proxy<'a, #generic_arg: #deps::Runtime>(&'a #deps::AppHandle<#generic_arg>);
                                        impl<'de, #generic_arg: #deps::Runtime> #deps::DeserializeSeed<'de> for Proxy<'de, #generic_arg> {
                                            type Value = #command_arg_types;

                                            fn deserialize<__D>(self, de: __D) -> #deps::Result<Self::Value, __D::Error>
                                            where
                                                __D: #deps::Deserializer<'de>,
                                            {
                                                let de = #deps::CommandArgDeserializer {
                                                    app_handle: self.0,
                                                    de,
                                                };

                                                #deps::DeserializeProxy::<#generic_arg, #command_arg_types>(#deps::PhantomData).deserialize(de)
                                            }
                                        }

                                        let arg = map.next_value_seed(Proxy(&self.0))?;
                                        command_args.#command_arg_fields = #deps::Some(arg);
                                    }
                                )*
                                _ => {
                                    return #deps::Err(<__A::Error as #deps::SerdeError>::unknown_field(
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
            runtime_generic_param,
            command_name,
            deps_path,
            return_type,
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
            runtime_generic_param,
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
