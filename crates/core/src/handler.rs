use std::future::Future;

use tauri::AppHandle;

use crate::BoxError;

pub trait BinIpcHandler<R: tauri::Runtime>: 'static + Send + Sync {
    type Future: Future<Output = Result<Vec<u8>, BoxError>> + 'static + Send;

    fn handle(
        &self,
        app: &AppHandle<R>,
        name: &str,
        payload: &[u8],
    ) -> Result<Self::Future, BoxError>;
}
