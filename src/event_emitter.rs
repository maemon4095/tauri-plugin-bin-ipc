use tauri::{AppHandle, Manager};

pub static BIN_IPC_EVENT_NAME: &str = "bin-ipc-signal";
pub struct EventEmitter<'a> {
    scheme: &'a str,
    id: usize,
}

impl<'a> EventEmitter<'a> {
    pub fn new(scheme: &'a str, id: usize) -> Self {
        Self { scheme, id }
    }

    fn emit<R: tauri::Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        ty: IpcEventType,
    ) -> Result<(), tauri::Error> {
        app_handle.emit_all(
            BIN_IPC_EVENT_NAME,
            IpcEvent {
                ty,
                scheme: &self.scheme,
                id: self.id,
            },
        )
    }

    pub fn emit_ready<R: tauri::Runtime>(
        &self,
        app_handle: &AppHandle<R>,
    ) -> Result<(), tauri::Error> {
        self.emit(app_handle, IpcEventType::ReadyToPop)
    }

    pub fn emit_cleanup<R: tauri::Runtime>(
        &self,
        app_handle: &AppHandle<R>,
    ) -> Result<(), tauri::Error> {
        self.emit(app_handle, IpcEventType::CleanUp)
    }
}

#[derive(serde::Serialize, Debug, Clone)]
enum IpcEventType {
    #[serde(rename = "ready-to-pop")]
    ReadyToPop,
    #[serde(rename = "cleanup")]
    CleanUp,
}

#[derive(serde::Serialize, Debug, Clone)]
struct IpcEvent<'a> {
    #[serde(rename = "type")]
    ty: IpcEventType,
    scheme: &'a str,
    id: usize,
}
