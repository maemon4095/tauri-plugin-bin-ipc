use tauri_plugin_bin_ipc::{
    bin_command, generate_bin_handler, BinIpcError, PluginBuilderBinIpcExtension,
};
fn main() {}

#[bin_command]
fn simple(x: usize) -> usize {
    x
}

#[bin_command]
fn returns_result(x: usize, y: i32) -> Result<usize, std::num::TryFromIntError> {
    let y: usize = y.try_into()?;
    Ok(x + y)
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

#[bin_command]
fn return_ipc_error(x: usize, y: i32) -> Result<usize, BinIpcError> {
    let y: usize = match y.try_into() {
        Ok(v) => v,
        Err(e) => return Err(BinIpcError::new_reportable(e)),
    };
    Ok(x + y)
}

#[allow(unused)]
fn gen_handle<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("test")
        .bin_ipc_handler(
            "scheme",
            generate_bin_handler![
                simple,
                returns_result,
                no_args,
                no_args_no_return,
                take_app_handle,
                async_command,
            ],
        )
        .build()
}
