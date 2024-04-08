use crate::channel::{Receiver, Sender};
use std::sync::Arc;
use tauri::{async_runtime::JoinHandle, AppHandle};

pub trait Listener<R: tauri::Runtime>: 'static + Send + Sync {
    type Err: Send;
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
    F: 'static
        + Send
        + Sync
        + for<'a> Fn(&'a AppHandle<R>, Sender, Receiver) -> JoinHandle<Result<(), E>>,
    E: Send,
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
    listener: Box<dyn Send + Sync + Fn(&AppHandle<R>, usize, Sender, Receiver) -> JoinHandle<()>>,
}

impl<R: tauri::Runtime> ListenerBox<R> {
    pub fn new<L: Listener<R>>(
        on_close: impl 'static
            + Send
            + Sync
            + Fn(&AppHandle<R>, usize, Result<(), ListenerError<L::Err>>),
        listener: L,
    ) -> Self {
        let listener = Arc::new(listener);
        let on_close = Arc::new(on_close);
        Self {
            listener: Box::new(move |app, id, tx, rx| {
                let app = app.clone();
                let on_close = Arc::clone(&on_close);
                let listener = Arc::clone(&listener);
                tauri::async_runtime::spawn(async move {
                    let mut tx_clone = tx.clone();
                    let result = listener.listen(&app, tx, rx).await;
                    tx_clone.close_channel();
                    let result = match result {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => Err(ListenerError::Custom(e)),
                        Err(e) => Err(ListenerError::Tauri(e)),
                    };
                    on_close(&app, id, result);
                })
            }),
        }
    }

    pub fn listen(
        &self,
        app_handle: &AppHandle<R>,
        id: usize,
        tx: Sender,
        rx: Receiver,
    ) -> JoinHandle<()> {
        (self.listener)(app_handle, id, tx, rx)
    }
}

#[derive(Debug)]
pub enum ListenerError<E> {
    Tauri(tauri::Error),
    Custom(E),
}
