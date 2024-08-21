mod de;
#[doc(hidden)]
pub mod deps;

pub use de::{CommandArgDeserializer, DeserializeProxy};
pub use msgpack_macro::command;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime> {
    const NAME: &'static str;

    fn handle(
        &self,
        app: ::tauri::AppHandle<R>,
        payload: ::std::vec::Vec<u8>,
    ) -> impl ::std::future::Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>>;
}

#[command]
fn a(x: usize) -> usize {
    x
}

fn b<R: ::tauri::Runtime>(
    app: &::tauri::AppHandle<R>,
    payload: Vec<::std::primitive::u8>,
) -> Result<(), Box<dyn crate::deps::std::error::Error>> {
    use crate::deps::*;
    // 引数の番号のみで管理する。 AppHandleを受け取らないコマンドに対応するため。
    struct CommandArgs<R: tauri::Runtime> {
        __marker: PhantomData<R>,
        arg0: Option<usize>,
        arg1: Option<tauri::AppHandle<R>>,
    }

    let command_args = CommandArgs::deserialize(app, &payload)?;

    return Ok(());

    impl<R: tauri::Runtime> CommandArgs<R> {
        fn deserialize(
            app: &::tauri::AppHandle<R>,
            payload: &[u8],
        ) -> Result<CommandArgs<R>, rmp_serde::decode::Error> {
            struct Visitor<'a, R: tauri::Runtime>(&'a tauri::AppHandle<R>);

            impl<'de, R: tauri::Runtime> serde::de::Visitor<'de> for Visitor<'de, R> {
                type Value = CommandArgs<R>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("struct")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut command_args = CommandArgs::<R> {
                        __marker: PhantomData,
                        arg0: None,
                        arg1: None,
                    };

                    const COMMAND_ARG_NAME_0: &str = "arg0";

                    while let Some(k) = map.next_key::<&str>()? {
                        match k {
                            COMMAND_ARG_NAME_0 => {
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

                                let arg0 = map.next_value_seed(Proxy(&self.0))?;
                                command_args.arg0 = Some(arg0);
                            }
                            _ => {
                                return Err(<A::Error as serde::de::Error>::unknown_field(
                                    k,
                                    &[COMMAND_ARG_NAME_0],
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

#[cfg(test)]
mod test {
    use super::*;
    use pollster::block_on;

    #[test]
    fn test() {
        block_on(async {
            println!("{:?}", a(32).await);
        });
    }
}
