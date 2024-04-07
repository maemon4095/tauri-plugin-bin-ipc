use tauri::{AppHandle, Manager};

pub static BIN_IPC_EVENT_NAME: &str = "bin-ipc-signal";

pub struct EventEmitter<R: tauri::Runtime> {
    scheme: String,
    app_handle: AppHandle<R>,
}

impl<R: tauri::Runtime> EventEmitter<R> {
    pub fn new(scheme: String, app_handle: AppHandle<R>) -> Self {
        Self { scheme, app_handle }
    }

    pub fn emit_ready(&self) -> Result<(), tauri::Error> {
        self.app_handle.emit_all(
            BIN_IPC_EVENT_NAME,
            IpcEvent::ReadyToPop {
                scheme: &self.scheme,
            },
        )
    }
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum IpcEvent<'a> {
    ReadyToPop { scheme: &'a str },
    Disconnect,
}
