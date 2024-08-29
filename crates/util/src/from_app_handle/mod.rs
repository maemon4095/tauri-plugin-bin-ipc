mod proxy;

pub use proxy::from_app_handle_proxy;
use tauri::AppHandle;

pub trait FromAppHandle<R: tauri::Runtime> {
    fn from_app_handle(app: &AppHandle<R>) -> Self;
}

impl<R: tauri::Runtime> FromAppHandle<R> for AppHandle<R> {
    fn from_app_handle(app: &AppHandle<R>) -> Self {
        app.clone()
    }
}
