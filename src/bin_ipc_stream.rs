use std::pin::Pin;

use futures::{Sink, Stream};

use crate::{Body, Receiver, Sender};

pub struct BinIpcStream {
    pub(crate) id: usize,
    pub(crate) sender: Sender,
    pub(crate) receiver: Receiver,
}
impl BinIpcStream {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn into_split(self) -> (Sender, Receiver) {
        (self.sender, self.receiver)
    }
}

impl Sink<Body> for BinIpcStream {
    type Error = <Sender as Sink<Body>>::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { self.map_unchecked_mut(|me| &mut me.sender) };
        sender.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Body) -> Result<(), Self::Error> {
        let sender = unsafe { self.map_unchecked_mut(|me| &mut me.sender) };
        sender.start_send(item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { self.map_unchecked_mut(|me| &mut me.sender) };
        sender.poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let sender = unsafe { self.map_unchecked_mut(|me| &mut me.sender) };
        sender.poll_close(cx)
    }
}

impl Stream for BinIpcStream {
    type Item = Body;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let receiver = unsafe { self.map_unchecked_mut(|me| &mut me.receiver) };
        receiver.poll_next(cx)
    }
}
