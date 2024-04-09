// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::{SinkExt, StreamExt};
use tauri::AppHandle;
use tauri_plugin_bin_ipc::BinIpcStream;

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_bin_ipc::Builder::new()
                .register_bin_ipc_protocol(
                    "bin-ipc",
                    |_app: &AppHandle, mut stream: BinIpcStream| async move {
                        let reason = loop {
                            let Some(buf) = stream.next().await else {
                                break "close upstream".into();
                            };

                            if let Err(e) = stream.send(buf).await {
                                break format!("close downstream: {}", e);
                            }
                        };

                        println!("server close {:?}", reason)
                    },
                )
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
