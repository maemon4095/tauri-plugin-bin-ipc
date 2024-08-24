macro_rules! declare_error {
    ($id: ident; $msg: literal) => {
        #[derive(Debug)]
        pub struct $id;
        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($msg)
            }
        }
        impl std::error::Error for $id {}
    };
}

pub(crate) use declare_error;

use tauri::Manager;

pub trait AppHandleExt<R: tauri::Runtime> {
    fn lazy_state<T: Send + Sync>(
        &self,
        init: impl FnOnce(&tauri::AppHandle<R>) -> T,
    ) -> tauri::State<'_, T>;
}

impl<R: tauri::Runtime> AppHandleExt<R> for tauri::AppHandle<R> {
    fn lazy_state<T: Send + Sync>(
        &self,
        init: impl FnOnce(&tauri::AppHandle<R>) -> T,
    ) -> tauri::State<'_, T> {
        match self.try_state::<T>() {
            Some(v) => v,
            None => {
                self.manage(init(self));
                self.state::<T>()
            }
        }
    }
}
