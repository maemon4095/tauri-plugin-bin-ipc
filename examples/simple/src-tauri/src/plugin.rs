use std::{future::Future, pin::Pin, time::Duration};

use chrono::Local;
use tauri::plugin::{Builder, TauriPlugin};
use tauri_plugin_bin_ipc::{BinIpcHandler, BoxError, PluginBuilderBinIpcExtension};

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    Builder::new("simple-plugin")
        .bin_ipc_handler("simple-plugin", Handler)
        .build()
}

struct Handler;

impl<R: tauri::Runtime> BinIpcHandler<R> for Handler {
    type Future = Pin<Box<dyn Future<Output = Result<Vec<u8>, BoxError>> + Send>>;

    fn handle(
        &self,
        _app: &tauri::AppHandle<R>,
        name: &str,
        payload: &[u8],
    ) -> Result<Self::Future, BoxError> {
        let name = name.to_string();
        let payload = payload.to_vec();
        Ok(Box::pin(async move {
            let now = Local::now();
            tokio::time::sleep(Duration::from_secs(1)).await;
            let v = format!(
                "[{}]: command ({}) with {:?}",
                now.format("%Y-%d-%d %H:%M:%S"),
                name,
                payload
            );
            Ok(v.into_bytes())
        }))
    }
}
