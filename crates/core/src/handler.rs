use std::future::Future;

use tauri::AppHandle;

use crate::{util::ThreadSafe, BoxError};

pub trait BinIpcHandler<R: tauri::Runtime>: ThreadSafe {
    type Future: Future<Output = Result<Vec<u8>, BoxError>> + ThreadSafe;

    fn handle(&self, app: &AppHandle<R>, name: String, payload: Vec<u8>) -> Self::Future;
}
