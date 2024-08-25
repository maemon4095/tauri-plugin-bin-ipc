// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri_plugin_bin_ipc::{bin_command, generate_bin_handler, BuilderBinIpcExtension};

#[bin_command]
fn greet(name: String) -> String {
    format!("Hello {}!", name)
}

fn main() {
    tauri::Builder::default()
        .bin_ipc_handler("bin-ipc", generate_bin_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
