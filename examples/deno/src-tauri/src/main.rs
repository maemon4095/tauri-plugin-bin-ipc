// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::{SinkExt, StreamExt};
use tauri::AppHandle;
use tauri_plugin_bin_ipc::{Receiver, Sender};

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_bin_ipc::Builder::new()
                .register_bin_ipc_protocol(
                    "bin-ipc",
                    |_app: &AppHandle, mut tx: Sender, mut rx: Receiver| async move {
                        let reason = loop {
                            let Some(buf) = rx.next().await else {
                                break "closeup";
                            };
                            match tx.send(buf).await {
                                Ok(()) => (),
                                Err(_) => break "closedown",
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
