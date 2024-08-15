use std::time::Duration;

use chrono::Local;
use tauri::plugin::{Builder, TauriPlugin};
use tauri_plugin_bin_ipc::PluginBuilderBinIpcExtension;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    Builder::new("simple-plugin")
        .bin_ipc_handler("simple-plugin", |_, name, payload| async move {
            let now = Local::now();
            tokio::time::sleep(Duration::from_secs(1)).await;
            let v = format!(
                "[{}]: command ({}) with {:?}",
                now.format("%Y-%d-%d %H:%M:%S"),
                name,
                payload
            );
            Ok(v.into_bytes())
        })
        .build()
}
