#[doc(hidden)]
pub mod __deps;
mod de;
mod error;
mod flatten_join_handle;
mod or_future;
mod wrap_result;

use core::BoxError;
pub use msgpack_macro::bin_command;
pub use msgpack_macro::generate_handler;
use std::future::Future;
pub type HandleResult = Result<Vec<u8>, BoxError>;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime>: 'static + Send + Sync {
    const NAME: &'static str;

    fn handle(
        &self,
        app: &tauri::AppHandle<R>,
        payload: &[u8],
    ) -> impl 'static + Future<Output = HandleResult> + Send;
}
