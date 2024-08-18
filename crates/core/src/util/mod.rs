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

macro_rules! trait_alias {
    ($id: ident = $($bounds: tt)+) => {
        pub trait $id: $($bounds)* {}
        impl<T: $($bounds)*> $id for T {}
    };
}

pub(crate) use trait_alias;

use tauri::Manager;

trait_alias!(ThreadSafe = Send + Sync + 'static);

pub trait AppHandleExt<R: tauri::Runtime> {
    fn lazy_state<T: ThreadSafe>(
        &self,
        init: impl FnOnce(&tauri::AppHandle<R>) -> T,
    ) -> tauri::State<'_, T>;
}

impl<R: tauri::Runtime> AppHandleExt<R> for tauri::AppHandle<R> {
    fn lazy_state<T: ThreadSafe>(
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
