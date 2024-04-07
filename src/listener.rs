use crate::channel::{Receiver, Sender};

use tauri::{async_runtime::JoinHandle, AppHandle};

pub trait Listener<R: tauri::Runtime>: Send + Sync {
    type Err;
    fn listen(
        &self,
        app_handle: &AppHandle<R>,
        tx: Sender,
        rx: Receiver,
    ) -> JoinHandle<Result<(), Self::Err>>;
}

impl<R, E, F> Listener<R> for F
where
    R: tauri::Runtime,
    F: Send + Sync + for<'a> Fn(&'a AppHandle<R>, Sender, Receiver) -> JoinHandle<Result<(), E>>,
{
    type Err = E;

    fn listen(
        &self,
        app_handle: &AppHandle<R>,
        tx: Sender,
        rx: Receiver,
    ) -> JoinHandle<Result<(), Self::Err>> {
        self(app_handle, tx, rx)
    }
}

pub struct ListenerBox<R: tauri::Runtime> {
    listener: Box<dyn Send + Sync + Fn(&AppHandle<R>, Sender, Receiver) -> JoinHandle<()>>,
}

impl<R: tauri::Runtime> ListenerBox<R> {
    pub fn new<L: Listener<R>>(listener: L) -> Self {
        todo!()
    }

    pub fn listen(&self, app_handle: &AppHandle<R>, tx: Sender, rx: Receiver) -> JoinHandle<()> {
        (self.listener)(app_handle, tx, rx)
    }
}
