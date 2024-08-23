#[doc(hidden)]
pub mod __deps;
mod de;
mod error;
mod flatten_join_handle;
mod wrap_result;

use error::BoxError;
use std::future::Future;

pub use msgpack_macro::bin_command;
pub use msgpack_macro::generate_handler;
pub type HandleResult = Result<Vec<u8>, BoxError>;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime> {
    const NAME: &'static str;

    fn handle(
        &self,
        app: &tauri::AppHandle<R>,
        payload: Vec<u8>,
    ) -> impl Future<Output = HandleResult>;
}
