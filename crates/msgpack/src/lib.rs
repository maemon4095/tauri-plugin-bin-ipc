pub use msgpack_macro::command;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime> {
    const NAME: &'static str;

    fn handle(
        &self,
        app: ::tauri::AppHandle<R>,
        payload: ::std::vec::Vec<u8>,
    ) -> impl ::std::future::Future<Output = Vec<u8>>;
}

#[command]
fn a(x: usize) -> usize {
    x
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
