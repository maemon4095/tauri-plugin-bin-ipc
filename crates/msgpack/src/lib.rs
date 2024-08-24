#[doc(hidden)]
pub mod command_deps;
mod de;
mod error;
mod flatten_join_handle;
mod or_future;
mod wrap_result;

use bin_ipc_core::BoxError;
use std::future::Future;

pub use msgpack_macro::{bin_command, generate_bin_handler};
pub type HandleResult = Result<Vec<u8>, BoxError>;
#[doc(hidden)]
pub use command_deps as __bin_ipc_deps_0cc84921_b5dc_4044_86a1_58ee53f2643a;

pub trait TauriPluginBinIpcMessagePackCommand<R: tauri::Runtime>: 'static + Send + Sync {
    const NAME: &'static str;

    fn handle(
        &self,
        app: &tauri::AppHandle<R>,
        payload: &[u8],
    ) -> impl 'static + Future<Output = HandleResult> + Send;
}
