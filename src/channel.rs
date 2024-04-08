use futures::channel::mpsc;
use std::pin::Pin;
use std::sync::Arc;

use crate::{error::BoxError, Body};

pub fn channel(
    on_send: impl 'static + Send + Sync + Fn() -> Result<(), BoxError>,
    upbuf: usize,
    downbuf: usize,
) -> (
    (Sender, Receiver),
    (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>),
) {
    let (tx, client_rx) = mpsc::channel(downbuf);
    let (client_tx, rx) = mpsc::channel(upbuf);

    let tx = Sender {
        on_send: Arc::new(on_send),
        sender: tx,
    };
    let rx = Receiver { receiver: rx };

    ((tx, rx), (client_tx, client_rx))
}

#[derive(Clone)]
pub struct Sender {
    on_send: Arc<dyn 'static + Send + Sync + Fn() -> Result<(), BoxError>>,
    sender: mpsc::Sender<Body>,
}

impl Sender {
    pub fn close_channel(&mut self) {
        self.sender.close_channel()
    }
}

impl futures::Sink<Body> for Sender {
    type Error = BoxError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { Pin::map_unchecked_mut(self, |s| &mut s.sender) };
        sender.poll_ready(cx).map_err(Into::into)
    }

    fn start_send(self: Pin<&mut Self>, item: Body) -> Result<(), Self::Error> {
        let (sender, on_send) = unsafe {
            let me = Pin::get_unchecked_mut(self);
            let sender = Pin::new_unchecked(&mut me.sender);
            let on_send = Pin::new_unchecked(&mut me.on_send);
            (sender, on_send)
        };
        match sender.start_send(item) {
            Ok(()) => {
                on_send()?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { Pin::map_unchecked_mut(self, |s| &mut s.sender) };
        sender.poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { Pin::map_unchecked_mut(self, |s| &mut s.sender) };
        sender.poll_close(cx).map_err(Into::into)
    }
}

pub struct Receiver {
    receiver: mpsc::Receiver<Body>,
}

impl futures::Stream for Receiver {
    type Item = Body;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let receiver = unsafe { Pin::map_unchecked_mut(self, |me| &mut me.receiver) };
        receiver.poll_next(cx)
    }
}
