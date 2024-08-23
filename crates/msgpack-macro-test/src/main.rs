use tauri_plugin_bin_ipc_msgpack::{bin_command, generate_handler};

fn main() {}

#[bin_command]
fn returns_result(x: usize, y: i32) -> Result<usize, std::num::TryFromIntError> {
    let y: usize = y.try_into()?;
    Ok(x + y)
}

#[bin_command]
fn simple(x: usize) -> usize {
    x
}

#[bin_command]
fn no_args() -> usize {
    0
}

#[bin_command]
fn no_args_no_return() {}

#[bin_command]
fn take_app_handle<R: tauri::Runtime>(_app: tauri::AppHandle<R>) {}

#[bin_command]
async fn async_command(x: usize, y: i32) -> Result<usize, std::num::TryFromIntError> {
    let y: usize = y.try_into()?;
    Ok(x + y)
}

fn gen_handle() {
    {
        struct GeneratedHandler;

        impl<R: ::tauri_plugin_bin_ipc_msgpack::__deps::tauri::Runtime>
            ::tauri_plugin_bin_ipc_msgpack::__deps::BinIpcHandler<R> for GeneratedHandler
        {
            type Future = ::tauri_plugin_bin_ipc_msgpack::__deps::FlattenJoinHandle<
                ::tauri_plugin_bin_ipc_msgpack::__deps::Vec<
                    ::tauri_plugin_bin_ipc_msgpack::__deps::u8,
                >,
            >;
            fn handle(
                &self,
                app: &::tauri_plugin_bin_ipc_msgpack::__deps::tauri::AppHandle<R>,
                name: String,
                payload: Vec<u8>,
            ) -> Self::Future {
                match name.as_str() {
                    _ => todo!(),
                }
            }
        }
        GeneratedHandler
    }
}
