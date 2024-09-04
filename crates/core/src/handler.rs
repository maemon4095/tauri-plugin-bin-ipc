use std::future::Future;

use tauri::AppHandle;

use crate::error::BinIpcError;

pub trait BinIpcHandler<R: tauri::Runtime>: 'static + Send + Sync {
    type Future: Future<Output = Result<Vec<u8>, BinIpcError>> + 'static + Send;

    fn handle(
        &self,
        app: &AppHandle<R>,
        name: &str,
        payload: &[u8],
    ) -> Result<Self::Future, BinIpcError>;
}
