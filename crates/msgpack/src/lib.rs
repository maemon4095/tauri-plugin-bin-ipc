pub use msgpack_macro::command;
use rmp_serde::decode::ReadRefReader;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime> {
    const NAME: &'static str;

    fn handle(
        &self,
        app: ::tauri::AppHandle<R>,
        payload: ::std::vec::Vec<u8>,
    ) -> impl ::std::future::Future<Output = Vec<u8>>;
}

// 外部クレートの型なので、実装のコンフリクトが起きる。
// traitを使った方法は使えない。型を見て関数ポインタを返すような仕様にするか？
impl<R: tauri::Runtime> CommandArg<R> for tauri::AppHandle<R> {
    fn from_command<'de>(
        de: CommandDeserializer<'de, R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(de.app_handle)
    }
}

impl<T: serde::de::DeserializeOwned, R: tauri::Runtime> CommandArg<R> for T {
    fn from_command<'de>(
        de: CommandDeserializer<'de, R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        todo!()
    }
}

trait CommandArg<R: tauri::Runtime>: Sized {
    fn from_command<'de>(
        de: CommandDeserializer<'de, R>,
    ) -> Result<Self, Box<dyn std::error::Error>>;
}

struct CommandDeserializer<'de, R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
    de: rmp_serde::Deserializer<ReadRefReader<'de, [u8]>>,
}

struct Payload<R: tauri::Runtime> {
    app: tauri::AppHandle<R>,
    arg0: usize,
}

impl<R: tauri::Runtime> Payload<R> {
    fn de<'de>(
        app_handle: tauri::AppHandle<R>,
        de: rmp_serde::Deserializer<ReadRefReader<'de, [u8]>>,
    ) {
    }
}
#[command]
fn a(x: usize) -> usize {
    x
}

fn b<'de>(mut de: rmp_serde::Deserializer<ReadRefReader<'de, [u8]>>) {
    struct X {}

    struct V {}

    impl<'de> serde::de::Visitor<'de> for V {
        type Value = X;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("struct")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            while let Some(k) = map.next_key::<&str>()? {
                match k {
                    "a" => {
                        map.next_value::<usize>()?;
                    }
                    _ => panic!(),
                }
            }

            todo!()
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
